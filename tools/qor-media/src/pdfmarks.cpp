#include "pdfmarks.h"

#include <QColor>
#include <QDateTime>
#include <QFont>
#include <QFontDatabase>
#include <QJSValue>
#include <QMap>
#include <QPainterPath>
#include <QPolygonF>
#include <QRectF>
#include <QSet>
#include <QTransform>
#include <QUuid>

#include <algorithm>
#include <cmath>
#include <limits>

#include <fpdf_annot.h>
#include <fpdf_edit.h>

#include "pdfium.h"

namespace qormedia {

namespace {

// The most one save may hold: far past anything made by hand, short of what would stall it.
constexpr qsizetype kMostPoints = 200000;
constexpr qsizetype kMostText = 100000;
// Every number is a place on a page, or a size: none is near this.
constexpr double kFarthest = 1e6;

enum class Kind { Highlight, Underline, StrikeOut, Note, Ink, Signature };

// A mark as the reader made it, checked.
struct Mark
{
    Kind kind = Kind::Highlight;
    int page = -1;
    QColor colour;
    QList<QRectF> rects;      // words
    QRectF box;               // a note's square; a signature's box
    QString text;             // a note's
    QList<QPolygonF> strokes; // ink's, in points; a drawn signature's, from 0 to 1 in its box
    QList<QPolygonF> shapes;  // a typed signature's, from 0 to 1 in its box
    double width = 0;         // ink's line in points; a drawn signature's as a share of its box's height
};

// A value from QML as plain data, whatever wraps it.
QVariant plain(const QVariant &value)
{
    return value.metaType() == QMetaType::fromType<QJSValue>() ? value.value<QJSValue>().toVariant() : value;
}

bool within(double v)
{
    return std::isfinite(v) && std::abs(v) <= kFarthest;
}

bool number(const QVariant &value, double *out)
{
    const QVariant v = plain(value);
    bool ok = false;
    const double n = v.isValid() ? v.toDouble(&ok) : 0;
    if (!ok || !within(n))
        return false;
    *out = n;
    return true;
}

bool whole(const QVariant &value, int *out)
{
    double n = 0;
    if (!number(value, &n) || n != std::floor(n) || n < 0 || n > std::numeric_limits<int>::max())
        return false;
    *out = int(n);
    return true;
}

bool point(const QVariant &value, QPointF *out)
{
    const QVariant v = plain(value);
    if (v.metaType() == QMetaType::fromType<QPointF>() || v.metaType() == QMetaType::fromType<QPoint>()) {
        *out = v.toPointF();
        return within(out->x()) && within(out->y());
    }
    const QVariantMap map = v.toMap();
    double x = 0, y = 0;
    if (!number(map.value(QStringLiteral("x")), &x) || !number(map.value(QStringLiteral("y")), &y))
        return false;
    *out = {x, y};
    return true;
}

bool rect(const QVariant &value, QRectF *out)
{
    const QVariant v = plain(value);
    QRectF r;
    if (v.metaType() == QMetaType::fromType<QRectF>() || v.metaType() == QMetaType::fromType<QRect>()) {
        r = v.toRectF();
    } else {
        const QVariantMap map = v.toMap();
        double x = 0, y = 0, w = 0, h = 0;
        if (!number(map.value(QStringLiteral("x")), &x) || !number(map.value(QStringLiteral("y")), &y)
            || !number(map.value(QStringLiteral("width")), &w) || !number(map.value(QStringLiteral("height")), &h))
            return false;
        r = QRectF(x, y, w, h);
    }
    if (!within(r.left()) || !within(r.top()) || !within(r.right()) || !within(r.bottom()) || r.width() <= 0 || r.height() <= 0)
        return false;
    *out = r;
    return true;
}

bool list(const QVariant &value, QVariantList *out)
{
    const QVariant v = plain(value);
    if (v.metaType() == QMetaType::fromType<QString>() || !v.canConvert<QVariantList>())
        return false;
    *out = v.toList();
    return true;
}

bool colour(const QVariant &value, QColor *out)
{
    const QVariant v = plain(value);
    const QColor c = v.metaType() == QMetaType::fromType<QColor>() ? v.value<QColor>() : QColor::fromString(v.toString());
    if (!c.isValid())
        return false;
    *out = c;
    return true;
}

// Lists of points, `budget` counting down every point in the save.
bool lines(const QVariant &value, QList<QPolygonF> *out, qsizetype *budget)
{
    QVariantList all;
    if (!list(value, &all) || all.isEmpty())
        return false;
    for (const QVariant &line : std::as_const(all)) {
        QVariantList at;
        if (!list(line, &at) || at.isEmpty() || (*budget -= at.size()) < 0)
            return false;
        QPolygonF polygon;
        for (const QVariant &p : std::as_const(at)) {
            QPointF q;
            if (!point(p, &q))
                return false;
            polygon << q;
        }
        out->append(polygon);
    }
    return true;
}

bool parse(const QVariant &entry, int pages, qsizetype *budget, Mark *m)
{
    static const QMap<QString, Kind> kinds{{QStringLiteral("highlight"), Kind::Highlight},
                                           {QStringLiteral("underline"), Kind::Underline},
                                           {QStringLiteral("strikeout"), Kind::StrikeOut},
                                           {QStringLiteral("note"), Kind::Note},
                                           {QStringLiteral("ink"), Kind::Ink},
                                           {QStringLiteral("signature"), Kind::Signature}};
    const QVariantMap map = plain(entry).toMap();
    const QString kind = plain(map.value(QStringLiteral("kind"))).toString();
    if (!kinds.contains(kind) || !whole(map.value(QStringLiteral("page")), &m->page) || m->page >= pages
        || !colour(map.value(QStringLiteral("colour")), &m->colour))
        return false;
    m->kind = kinds.value(kind);
    switch (m->kind) {
    case Kind::Highlight:
    case Kind::Underline:
    case Kind::StrikeOut: {
        QVariantList rects;
        if (!list(map.value(QStringLiteral("rects")), &rects) || rects.isEmpty() || (*budget -= 4 * rects.size()) < 0)
            return false;
        for (const QVariant &r : std::as_const(rects)) {
            QRectF run;
            if (!rect(r, &run))
                return false;
            m->rects << run;
        }
        return true;
    }
    case Kind::Note: {
        double x = 0, y = 0;
        m->text = plain(map.value(QStringLiteral("text"))).toString();
        if (!number(map.value(QStringLiteral("x")), &x) || !number(map.value(QStringLiteral("y")), &y)
            || m->text.size() > kMostText)
            return false;
        m->box = QRectF(x, y, PdfMarks::NoteSize, PdfMarks::NoteSize);
        return true;
    }
    case Kind::Ink:
        return number(map.value(QStringLiteral("width")), &m->width) && m->width > 0 && m->width <= 100
            && lines(map.value(QStringLiteral("strokes")), &m->strokes, budget);
    case Kind::Signature: {
        if (!rect(QVariantMap{{QStringLiteral("x"), map.value(QStringLiteral("x"))},
                              {QStringLiteral("y"), map.value(QStringLiteral("y"))},
                              {QStringLiteral("width"), map.value(QStringLiteral("width"))},
                              {QStringLiteral("height"), map.value(QStringLiteral("height"))}},
                  &m->box))
            return false;
        // Drawn, with its line, or typed: one or the other.
        if (map.contains(QStringLiteral("strokes")))
            return !map.contains(QStringLiteral("shapes")) && number(map.value(QStringLiteral("line")), &m->width)
                && m->width > 0 && m->width <= 1 && lines(map.value(QStringLiteral("strokes")), &m->strokes, budget);
        return lines(map.value(QStringLiteral("shapes")), &m->shapes, budget);
    }
    }
    return false;
}

// What the reader calls each of a PDF's own marks, by its annotation's subtype. Anything else (a link, a form's field,
// a note's pop-up, a sound, a film) is not a mark it shows or takes away.
QString kindOf(FPDF_ANNOTATION_SUBTYPE subtype)
{
    switch (subtype) {
    case FPDF_ANNOT_TEXT: return QStringLiteral("note");
    case FPDF_ANNOT_HIGHLIGHT: return QStringLiteral("highlight");
    case FPDF_ANNOT_UNDERLINE: return QStringLiteral("underline");
    case FPDF_ANNOT_SQUIGGLY: return QStringLiteral("squiggly");
    case FPDF_ANNOT_STRIKEOUT: return QStringLiteral("strikeout");
    case FPDF_ANNOT_INK: return QStringLiteral("ink");
    case FPDF_ANNOT_STAMP: return QStringLiteral("stamp");
    case FPDF_ANNOT_FREETEXT: return QStringLiteral("words");
    case FPDF_ANNOT_CARET: return QStringLiteral("caret");
    case FPDF_ANNOT_LINE:
    case FPDF_ANNOT_SQUARE:
    case FPDF_ANNOT_CIRCLE:
    case FPDF_ANNOT_POLYGON:
    case FPDF_ANNOT_POLYLINE: return QStringLiteral("shape");
    default: return {};
    }
}

struct Annot
{
    FPDF_ANNOTATION handle = nullptr;
    ~Annot()
    {
        if (handle)
            FPDFPage_CloseAnnot(handle);
    }
};

QString stringOf(FPDF_ANNOTATION annot, const char *key)
{
    const unsigned long bytes = FPDFAnnot_GetStringValue(annot, key, nullptr, 0);
    if (bytes <= 2)
        return {};
    QList<FPDF_WCHAR> buffer(qsizetype(bytes / 2), 0);
    FPDFAnnot_GetStringValue(annot, key, buffer.data(), bytes);
    return QString::fromUtf16(reinterpret_cast<const char16_t *>(buffer.constData()), buffer.size() - 1);
}

bool setString(FPDF_ANNOTATION annot, const char *key, const QString &value)
{
    return FPDFAnnot_SetStringValue(annot, key, reinterpret_cast<FPDF_WIDESTRING>(value.utf16()));
}

// A number as a PDF writes it: no more than three places, no trailing zeros.
QByteArray num(double v)
{
    QByteArray s = QByteArray::number(v, 'f', 3);
    while (s.contains('.') && (s.endsWith('0') || s.endsWith('.')))
        s.chop(1);
    return s == "-0" ? QByteArray("0") : s;
}

// An appearance's drawing, given in the page's space as the reader lays it out and written in the PDF's, with the
// bounds of everything drawn, which are the annotation's rectangle and its appearance's box.
class Appearance
{
public:
    explicit Appearance(const QTransform &toPdf) : m_toPdf(toPdf) {}

    void op(const char *text)
    {
        m_ops += text;
        m_ops += '\n';
    }
    void colour(const QColor &c, const char *which)
    {
        m_ops += num(c.redF()) + ' ' + num(c.greenF()) + ' ' + num(c.blueF()) + ' ' + which + '\n';
    }
    void width(double w) { m_ops += num(w) + " w\n"; }
    void move(QPointF p)
    {
        at(p);
        m_ops += " m\n";
    }
    void line(QPointF p)
    {
        at(p);
        m_ops += " l\n";
    }
    void polygon(const QPolygonF &points)
    {
        move(points.first());
        for (qsizetype i = 1; i < points.size(); ++i)
            line(points[i]);
        op("h");
    }
    void polyline(const QPolygonF &points)
    {
        move(points.first());
        // A single point is a dot, which a round cap draws.
        line(points.size() == 1 ? points.first() : points[1]);
        for (qsizetype i = 2; i < points.size(); ++i)
            line(points[i]);
        op("S");
    }
    // A point the rectangle must hold, though nothing is drawn there.
    void include(QPointF p) { grow(m_toPdf.map(p)); }

    FS_RECTF bounds(double margin) const
    {
        return {float(m_left - margin), float(m_top + margin), float(m_right + margin), float(m_bottom - margin)};
    }
    QString text() const { return QString::fromLatin1(m_ops); }

private:
    void at(QPointF view)
    {
        const QPointF p = m_toPdf.map(view);
        grow(p);
        m_ops += num(p.x()) + ' ' + num(p.y());
    }
    void grow(QPointF p)
    {
        m_left = std::min(m_left, p.x());
        m_right = std::max(m_right, p.x());
        m_bottom = std::min(m_bottom, p.y());
        m_top = std::max(m_top, p.y());
    }

    QTransform m_toPdf;
    QByteArray m_ops;
    double m_left = std::numeric_limits<double>::max(), m_right = std::numeric_limits<double>::lowest();
    double m_bottom = std::numeric_limits<double>::max(), m_top = std::numeric_limits<double>::lowest();
};

QPolygonF inBox(const QPolygonF &unit, const QRectF &box)
{
    QPolygonF placed;
    for (const QPointF &u : unit)
        placed << QPointF(box.left() + u.x() * box.width(), box.top() + u.y() * box.height());
    return placed;
}

bool write(FPDF_PAGE page, const Mark &m)
{
    const QTransform toPdf = pageSpace(page);
    Appearance look(toPdf);
    FPDF_ANNOTATION_SUBTYPE subtype = FPDF_ANNOT_UNKNOWN;
    double margin = 0;
    switch (m.kind) {
    case Kind::Highlight:
        // Through the words, so they read: the appearance's own state, which PDFium adds for a colour less than whole.
        subtype = FPDF_ANNOT_HIGHLIGHT;
        look.op("/GS gs");
        look.colour(m.colour, "rg");
        for (const QRectF &r : m.rects)
            look.polygon({r.topLeft(), r.topRight(), r.bottomRight(), r.bottomLeft()});
        look.op("f");
        break;
    case Kind::Underline:
    case Kind::StrikeOut:
        subtype = m.kind == Kind::Underline ? FPDF_ANNOT_UNDERLINE : FPDF_ANNOT_STRIKEOUT;
        look.colour(m.colour, "RG");
        for (const QRectF &r : m.rects) {
            const double t = std::clamp(r.height() / 14, 0.5, 3.0);
            const double y = m.kind == Kind::Underline ? r.bottom() - t / 2 : r.center().y();
            look.width(t);
            look.move({r.left(), y});
            look.line({r.right(), y});
            look.op("S");
            for (const QPointF &corner : {r.topLeft(), r.bottomRight()})
                look.include(corner);
        }
        break;
    case Kind::Note: {
        // A sheet with its corner turned down and three lines on it, in the note's colour.
        subtype = FPDF_ANNOT_TEXT;
        const QRectF s = m.box;
        const double w = s.width();
        look.op("1 j");
        look.width(1);
        look.colour(m.colour, "rg");
        look.colour(m.colour.darker(170), "RG");
        look.polygon({s.topLeft(), {s.left() + 0.7 * w, s.top()}, {s.right(), s.top() + 0.3 * w}, s.bottomRight(), s.bottomLeft()});
        look.op("B");
        look.polyline({{s.left() + 0.7 * w, s.top()}, {s.left() + 0.7 * w, s.top() + 0.3 * w}, {s.right(), s.top() + 0.3 * w}});
        for (const double down : {0.5, 0.65, 0.8})
            look.polyline({{s.left() + 0.2 * w, s.top() + down * w}, {s.left() + 0.8 * w, s.top() + down * w}});
        margin = 1;
        break;
    }
    case Kind::Ink:
        subtype = FPDF_ANNOT_INK;
        look.op("1 J");
        look.op("1 j");
        look.width(m.width);
        look.colour(m.colour, "RG");
        for (const QPolygonF &stroke : m.strokes)
            look.polyline(stroke);
        margin = m.width / 2 + 0.5;
        break;
    case Kind::Signature:
        subtype = FPDF_ANNOT_STAMP;
        if (!m.strokes.isEmpty()) {
            const double line = m.width * m.box.height();
            look.op("1 J");
            look.op("1 j");
            look.width(line);
            look.colour(m.colour, "RG");
            for (const QPolygonF &stroke : m.strokes)
                look.polyline(inBox(stroke, m.box));
            margin = line / 2 + 0.5;
        } else {
            look.colour(m.colour, "rg");
            for (const QPolygonF &shape : m.shapes)
                look.polygon(inBox(shape, m.box));
            look.op("f");
            margin = 0.5;
        }
        look.include(m.box.topLeft());
        look.include(m.box.bottomRight());
        break;
    }

    Annot annot{FPDFPage_CreateAnnot(page, subtype)};
    if (!annot.handle)
        return false;
    const QColor c = m.colour.toRgb();
    bool done = FPDFAnnot_SetFlags(annot.handle, FPDF_ANNOT_FLAG_PRINT);
    // The colour, and a highlight's opacity, before the appearance, which PDFium will not colour after.
    if (m.kind != Kind::Signature)
        done = done && FPDFAnnot_SetColor(annot.handle, FPDFANNOT_COLORTYPE_Color, c.red(), c.green(), c.blue(),
                                          m.kind == Kind::Highlight ? qRound(255 * PdfMarks::HighlightOpacity) : 255);
    if (m.kind == Kind::Ink) {
        done = done && FPDFAnnot_SetBorder(annot.handle, 0, 0, float(m.width));
        for (const QPolygonF &stroke : m.strokes) {
            QList<FS_POINTF> points;
            for (const QPointF &p : stroke) {
                const QPointF q = toPdf.map(p);
                points.append({float(q.x()), float(q.y())});
            }
            done = done && FPDFAnnot_AddInkStroke(annot.handle, points.constData(), size_t(points.size())) >= 0;
        }
    }
    if (m.kind == Kind::Highlight || m.kind == Kind::Underline || m.kind == Kind::StrikeOut)
        for (const QRectF &r : m.rects) {
            // Upper left, upper right, lower left, lower right of the words as they read.
            const QPointF a = toPdf.map(r.topLeft()), b = toPdf.map(r.topRight());
            const QPointF d = toPdf.map(r.bottomLeft()), e = toPdf.map(r.bottomRight());
            const FS_QUADPOINTSF quad{float(a.x()), float(a.y()), float(b.x()), float(b.y()),
                                      float(d.x()), float(d.y()), float(e.x()), float(e.y())};
            done = done && FPDFAnnot_AppendAttachmentPoints(annot.handle, &quad);
        }
    const FS_RECTF where = look.bounds(margin);
    done = done && FPDFAnnot_SetRect(annot.handle, &where);
    done = done && setString(annot.handle, "NM", QUuid::createUuid().toString(QUuid::WithoutBraces));
    done = done && setString(annot.handle, "M", QDateTime::currentDateTimeUtc().toString(QStringLiteral("'D:'yyyyMMddHHmmss'Z'")));
    if (m.kind == Kind::Note)
        done = done && setString(annot.handle, "Contents", m.text);
    if (m.kind == Kind::Signature)
        done = done && setString(annot.handle, "Contents", PdfMarks::tr("Signature"));
    const QString content = look.text();
    return done && FPDFAnnot_SetAP(annot.handle, FPDF_ANNOT_APPEARANCEMODE_NORMAL, reinterpret_cast<FPDF_WIDESTRING>(content.utf16()));
}

}  // namespace

QVariantList PdfMarks::colours()
{
    // A highlighter's yellow, green, blue and pink; a pen's red and black.
    static const struct
    {
        const char *name;
        const char *colour;
    } kColours[] = {{QT_TR_NOOP("Yellow"), "#ffd60a"}, {QT_TR_NOOP("Green"), "#2dc653"}, {QT_TR_NOOP("Blue"), "#3a86ff"},
                    {QT_TR_NOOP("Pink"), "#ff5da2"},   {QT_TR_NOOP("Red"), "#e5383b"},   {QT_TR_NOOP("Black"), "#1a1a1a"}};
    QVariantList list;
    for (const auto &c : kColours)
        list << QVariantMap{{QStringLiteral("name"), tr(c.name)}, {QStringLiteral("colour"), QString::fromLatin1(c.colour)}};
    return list;
}

QString PdfMarks::signatureFace()
{
    // Great Vibes, under the SIL Open Font License, carried in the executable: a face for the page, not the look's.
    static const QString family = [] {
        const int id = QFontDatabase::addApplicationFont(QStringLiteral(":/qt/qml/QOR/Media/fonts/GreatVibes-Regular.ttf"));
        const QStringList families = id >= 0 ? QFontDatabase::applicationFontFamilies(id) : QStringList();
        return families.isEmpty() ? QString() : families.first();
    }();
    return family;
}

QVariantList PdfMarks::read(const QString &path, const QString &password, int only)
{
    QVariantList marks;
    PdfiumLock locked;
    PdfSource source(path);
    if (!source.open())
        return marks;
    const QByteArray secret = password.toUtf8();
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, password.isEmpty() ? nullptr : secret.constData())};
    if (!document.handle)
        return marks;
    const int pages = FPDF_GetPageCount(document.handle);
    for (int i = only < 0 ? 0 : only; i < (only < 0 ? pages : qMin(only + 1, pages)); ++i) {
        PdfiumPage page{FPDF_LoadPage(document.handle, i)};
        if (!page.handle)
            continue;
        const QTransform toView = pageSpace(page.handle).inverted();
        for (int a = 0; a < FPDFPage_GetAnnotCount(page.handle); ++a) {
            Annot annot{FPDFPage_GetAnnot(page.handle, a)};
            const QString kind = annot.handle ? kindOf(FPDFAnnot_GetSubtype(annot.handle)) : QString();
            FS_RECTF r{};
            if (kind.isEmpty() || (FPDFAnnot_GetFlags(annot.handle) & (FPDF_ANNOT_FLAG_HIDDEN | FPDF_ANNOT_FLAG_NOVIEW))
                || !FPDFAnnot_GetRect(annot.handle, &r))
                continue;
            const QRectF at = toView.mapRect(QRectF(QPointF(r.left, r.bottom), QPointF(r.right, r.top)).normalized());
            marks << QVariantMap{{QStringLiteral("page"), i},
                                 {QStringLiteral("index"), a},
                                 {QStringLiteral("kind"), kind},
                                 {QStringLiteral("rect"), at},
                                 {QStringLiteral("text"), stringOf(annot.handle, "Contents")},
                                 {QStringLiteral("author"), stringOf(annot.handle, "T")}};
        }
    }
    return marks;
}

QVariantList PdfMarks::rectsOf(const QVariant &geometry)
{
    QVariantList rects;
    const auto add = [&](const QPolygonF &polygon) {
        const QRectF r = polygon.boundingRect();
        if (r.width() > 0 && r.height() > 0)
            rects << r;
    };
    const QVariant v = plain(geometry);
    if (v.metaType() == QMetaType::fromType<QList<QPolygonF>>()) {
        for (const QPolygonF &polygon : v.value<QList<QPolygonF>>())
            add(polygon);
        return rects;
    }
    QVariantList polygons;
    if (!list(v, &polygons))
        return rects;
    for (const QVariant &item : std::as_const(polygons)) {
        const QVariant p = plain(item);
        if (p.metaType() == QMetaType::fromType<QPolygonF>()) {
            add(p.value<QPolygonF>());
            continue;
        }
        QVariantList corners;
        QPolygonF polygon;
        if (list(p, &corners))
            for (const QVariant &corner : std::as_const(corners))
                if (QPointF at; point(corner, &at))
                    polygon << at;
        add(polygon);
    }
    return rects;
}

QVariantMap PdfMarks::typed(const QString &name)
{
    const QString words = name.simplified().left(80);
    if (words.isEmpty())
        return {};
    QFont face(signatureFace());
    face.setPixelSize(96);
    QPainterPath path;
    path.addText(0, 0, face, words);
    const QRectF box = path.boundingRect();
    if (box.width() <= 0 || box.height() <= 0)
        return {};
    QVariantList shapes;
    for (const QPolygonF &outline : path.toSubpathPolygons()) {
        if (outline.size() < 3)
            continue;
        QVariantList points;
        for (const QPointF &p : outline)
            points << QPointF((p.x() - box.left()) / box.width(), (p.y() - box.top()) / box.height());
        shapes << QVariant(points);
    }
    return {{QStringLiteral("aspect"), box.width() / box.height()}, {QStringLiteral("shapes"), shapes}};
}

bool writeMarks(FPDF_DOCUMENT document, const QVariantList &added, const QVariantList &removed, QString *why)
{
    const int pages = FPDF_GetPageCount(document);
    const auto refuse = [&](const QString &text) {
        *why = text;
        return false;
    };

    // Everything checked first.
    QList<Mark> marks;
    qsizetype budget = kMostPoints;
    for (const QVariant &entry : added) {
        Mark m;
        if (!parse(entry, pages, &budget, &m))
            return refuse(PdfMarks::tr("The marks to save were not a list QOR Media can write."));
        marks << m;
    }
    QMap<int, QSet<int>> taking;
    for (const QVariant &entry : removed) {
        const QVariantMap map = plain(entry).toMap();
        int page = -1, index = -1;
        if (!whole(map.value(QStringLiteral("page")), &page) || page >= pages || !whole(map.value(QStringLiteral("index")), &index))
            return refuse(PdfMarks::tr("The marks to take away were not a list QOR Media can follow."));
        taking[page].insert(index);
    }
    // Each mark taken away is one of the page's own, with the pop-up that shows a note's text.
    QMap<int, QList<int>> going;
    for (auto it = taking.cbegin(); it != taking.cend(); ++it) {
        PdfiumPage page{FPDF_LoadPage(document, it.key())};
        if (!page.handle)
            return refuse(PdfMarks::tr("A page of this PDF could not be read."));
        QSet<int> all;
        for (const int index : it.value()) {
            Annot annot{index < FPDFPage_GetAnnotCount(page.handle) ? FPDFPage_GetAnnot(page.handle, index) : nullptr};
            if (!annot.handle || kindOf(FPDFAnnot_GetSubtype(annot.handle)).isEmpty())
                return refuse(PdfMarks::tr("The marks to take away were not a list QOR Media can follow."));
            all.insert(index);
            Annot popup{FPDFAnnot_GetLinkedAnnot(annot.handle, "Popup")};
            if (const int at = popup.handle ? FPDFPage_GetAnnotIndex(page.handle, popup.handle) : -1; at >= 0)
                all.insert(at);
        }
        QList<int> order(all.cbegin(), all.cend());
        std::sort(order.begin(), order.end(), std::greater<int>());
        going.insert(it.key(), order);
    }

    // Taken away from the end of each page's list, so the rest keep their places; then the new marks after them.
    for (auto it = going.cbegin(); it != going.cend(); ++it) {
        PdfiumPage page{FPDF_LoadPage(document, it.key())};
        for (const int index : it.value())
            if (!page.handle || !FPDFPage_RemoveAnnot(page.handle, index))
                return refuse(PdfMarks::tr("A mark could not be taken away."));
    }
    for (const Mark &m : std::as_const(marks)) {
        PdfiumPage page{FPDF_LoadPage(document, m.page)};
        if (!page.handle || !write(page.handle, m))
            return refuse(PdfMarks::tr("A mark could not be written."));
    }
    return true;
}

}  // namespace qormedia
