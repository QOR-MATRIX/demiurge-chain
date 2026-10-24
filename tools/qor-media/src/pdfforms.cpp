#include "pdfforms.h"

#include <QColor>
#include <QRectF>
#include <QTransform>

#include <algorithm>

#include <fpdf_annot.h>

#include "pdfium.h"
#include "qmlvalues.h"

namespace qormedia {

namespace {

using namespace values;

// The most one save may fill, and the most words a field may take: far past any form filled by hand.
constexpr qsizetype kMostFills = 10000;
constexpr qsizetype kMostText = 100000;
// A text field's words spread one to each of its MaxLen boxes (PDF 1.7, table 228), a flag PDFium's header leaves unnamed.
constexpr int kComb = 1 << 24;

// A string PDFium writes into a buffer it is given, in UTF-16 with its end, once asked how long it is.
template<typename Get>
QString wide(Get get)
{
    const unsigned long bytes = get(nullptr, 0);
    if (bytes <= 2)
        return {};
    QList<FPDF_WCHAR> buffer(qsizetype(bytes / 2), 0);
    get(buffer.data(), bytes);
    return QString::fromUtf16(reinterpret_cast<const char16_t *>(buffer.constData()), buffer.size() - 1);
}

// Line breaks one way: a field keeps a carriage return, QML a line feed.
QString lines(QString text)
{
    return text.replace(QStringLiteral("\r\n"), QStringLiteral("\n")).replace(QLatin1Char('\r'), QLatin1Char('\n'));
}

QString valueOf(FPDF_FORMHANDLE form, FPDF_ANNOTATION annot)
{
    return lines(wide([&](FPDF_WCHAR *buffer, unsigned long size) { return FPDFAnnot_GetFormFieldValue(form, annot, buffer, size); }));
}

QString kindOf(int type)
{
    switch (type) {
    case FPDF_FORMFIELD_TEXTFIELD: return QStringLiteral("text");
    case FPDF_FORMFIELD_CHECKBOX: return QStringLiteral("check");
    case FPDF_FORMFIELD_RADIOBUTTON: return QStringLiteral("radio");
    case FPDF_FORMFIELD_COMBOBOX: return QStringLiteral("combo");
    case FPDF_FORMFIELD_LISTBOX: return QStringLiteral("list");
    case FPDF_FORMFIELD_PUSHBUTTON: return QStringLiteral("button");
    case FPDF_FORMFIELD_SIGNATURE: return QStringLiteral("signature");
    default: return {};
    }
}

bool isXfa(FPDF_DOCUMENT document)
{
    const int type = FPDF_GetFormType(document);
    return type == FORMTYPE_XFA_FULL || type == FORMTYPE_XFA_FOREGROUND;
}

bool shown(FPDF_ANNOTATION annot)
{
    return !(FPDFAnnot_GetFlags(annot) & (FPDF_ANNOT_FLAG_HIDDEN | FPDF_ANNOT_FLAG_NOVIEW));
}

// Acrobat shows an XFA form's answers from its XFA, which this PDFium cannot write: filled only in the fields, it would
// show the old ones.
QString xfaWhy()
{
    return PdfForms::tr("This form keeps its answers in XFA, which QOR Media cannot write yet: open it in another program to fill it.");
}

// Why QOR Media does not fill a widget, or nothing when it does.
QString whyNot(int type, int flags, bool xfa)
{
    if (xfa)
        return xfaWhy();
    switch (type) {
    case FPDF_FORMFIELD_TEXTFIELD:
        // PDFium would write a password into the file, which the standard says a reader never does (PDF 1.7, table 228).
        if (flags & FPDF_FORMFLAG_TEXT_PASSWORD)
            return PdfForms::tr("A password is never saved into a PDF, so this field is not filled here.");
        [[fallthrough]];
    case FPDF_FORMFIELD_CHECKBOX:
    case FPDF_FORMFIELD_RADIOBUTTON:
    case FPDF_FORMFIELD_COMBOBOX:
    case FPDF_FORMFIELD_LISTBOX: return flags & FPDF_FORMFLAG_READONLY ? PdfForms::tr("This field is read only.") : QString();
    case FPDF_FORMFIELD_PUSHBUTTON: return PdfForms::tr("A form's buttons do nothing here: QOR Media runs none of a form's actions.");
    case FPDF_FORMFIELD_SIGNATURE:
        return PdfForms::tr("QOR Media does not make certified signatures yet. A picture of yours can be placed here, which certifies nothing.");
    default: return PdfForms::tr("QOR Media does not fill this kind of field.");
    }
}

QVariantMap widgetOf(FPDF_FORMHANDLE form, FPDF_ANNOTATION annot, int page, int index, const QTransform &toView, bool xfa)
{
    const int type = FPDFAnnot_GetFormFieldType(form, annot);
    const QString kind = kindOf(type);
    FS_RECTF r{};
    if (kind.isEmpty() || !FPDFAnnot_GetRect(annot, &r))
        return {};
    const int flags = FPDFAnnot_GetFormFieldFlags(form, annot);
    const QString why = whyNot(type, flags, xfa);
    QVariantList options, chosen;
    for (int i = 0; i < FPDFAnnot_GetOptionCount(form, annot); ++i) {
        options << wide([&](FPDF_WCHAR *buffer, unsigned long size) { return FPDFAnnot_GetOptionLabel(form, annot, i, buffer, size); });
        if (FPDFAnnot_IsOptionSelected(form, annot, i))
            chosen << i;
    }
    float size = 0, most = 0, align = 0;
    FPDFAnnot_GetFontSize(form, annot, &size);
    // A field's own limit and alignment, where its widget is the field, as it is for most.
    FPDFAnnot_GetNumberValue(annot, "MaxLen", &most);
    FPDFAnnot_GetNumberValue(annot, "Q", &align);
    unsigned int red = 0, green = 0, blue = 0;
    const QColor ink = FPDFAnnot_GetFontColor(form, annot, &red, &green, &blue) ? QColor(int(red), int(green), int(blue)) : QColor(Qt::black);
    return {{QStringLiteral("page"), page},
            {QStringLiteral("index"), index},
            {QStringLiteral("name"), wide([&](FPDF_WCHAR *buffer, unsigned long n) { return FPDFAnnot_GetFormFieldName(form, annot, buffer, n); })},
            {QStringLiteral("label"), wide([&](FPDF_WCHAR *buffer, unsigned long n) { return FPDFAnnot_GetFormFieldAlternateName(form, annot, buffer, n); })},
            {QStringLiteral("kind"), kind},
            {QStringLiteral("rect"), toView.mapRect(QRectF(QPointF(r.left, r.bottom), QPointF(r.right, r.top)).normalized())},
            {QStringLiteral("value"), valueOf(form, annot)},
            {QStringLiteral("checked"), bool(FPDFAnnot_IsChecked(form, annot))},
            {QStringLiteral("state"), wide([&](FPDF_WCHAR *buffer, unsigned long n) { return FPDFAnnot_GetFormFieldExportValue(form, annot, buffer, n); })},
            {QStringLiteral("options"), options},
            {QStringLiteral("chosen"), chosen},
            {QStringLiteral("fillable"), why.isEmpty()},
            {QStringLiteral("why"), why},
            {QStringLiteral("xfa"), xfa},
            {QStringLiteral("required"), bool(flags & FPDF_FORMFLAG_REQUIRED)},
            {QStringLiteral("multiline"), type == FPDF_FORMFIELD_TEXTFIELD && (flags & FPDF_FORMFLAG_TEXT_MULTILINE)},
            {QStringLiteral("secret"), type == FPDF_FORMFIELD_TEXTFIELD && (flags & FPDF_FORMFLAG_TEXT_PASSWORD)},
            {QStringLiteral("comb"), type == FPDF_FORMFIELD_TEXTFIELD && (flags & kComb)},
            {QStringLiteral("maxLength"), type == FPDF_FORMFIELD_TEXTFIELD && most > 0 ? int(most) : 0},
            {QStringLiteral("editable"), type == FPDF_FORMFIELD_COMBOBOX && (flags & FPDF_FORMFLAG_CHOICE_EDIT)},
            {QStringLiteral("multiple"), type == FPDF_FORMFIELD_LISTBOX && (flags & FPDF_FORMFLAG_CHOICE_MULTI_SELECT)},
            {QStringLiteral("fontSize"), size > 0 ? double(size) : 0.0},
            {QStringLiteral("colour"), ink.name()},
            {QStringLiteral("align"), std::clamp(int(align), 0, 2)}};
}

// An entry of what the reader filled, checked.
struct Fill
{
    enum class Way { Text, Checked, Chosen };
    int page = -1;
    int index = -1;
    Way way = Way::Text;
    QString text;
    bool checked = false;
    QList<int> chosen;
};

bool parse(const QVariant &entry, int pages, Fill *f)
{
    const QVariantMap map = plain(entry).toMap();
    if (!whole(map.value(QStringLiteral("page")), &f->page) || f->page >= pages || !whole(map.value(QStringLiteral("index")), &f->index))
        return false;
    const bool text = map.contains(QStringLiteral("text"));
    const bool checked = map.contains(QStringLiteral("checked"));
    const bool chosen = map.contains(QStringLiteral("chosen"));
    // One way of filling it, and only one.
    if (int(text) + int(checked) + int(chosen) != 1)
        return false;
    if (text) {
        const QVariant v = plain(map.value(QStringLiteral("text")));
        f->way = Fill::Way::Text;
        f->text = lines(v.toString());
        return v.metaType() == QMetaType::fromType<QString>() && f->text.size() <= kMostText;
    }
    if (checked) {
        const QVariant v = plain(map.value(QStringLiteral("checked")));
        f->way = Fill::Way::Checked;
        f->checked = v.toBool();
        return v.metaType() == QMetaType::fromType<bool>();
    }
    QVariantList places;
    if (!list(map.value(QStringLiteral("chosen")), &places))
        return false;
    f->way = Fill::Way::Chosen;
    for (const QVariant &place : std::as_const(places)) {
        int i = -1;
        if (!whole(place, &i) || f->chosen.contains(i))
            return false;
        f->chosen << i;
    }
    return true;
}

// Whether the widget an entry names is one this fills, in the way the entry asks.
bool fits(FPDF_FORMHANDLE form, FPDF_ANNOTATION annot, const Fill &f)
{
    if (FPDFAnnot_GetSubtype(annot) != FPDF_ANNOT_WIDGET || !shown(annot))
        return false;
    const int type = FPDFAnnot_GetFormFieldType(form, annot);
    const int flags = FPDFAnnot_GetFormFieldFlags(form, annot);
    if (!whyNot(type, flags, false).isEmpty())
        return false;
    switch (f.way) {
    case Fill::Way::Text: {
        const bool editable = type == FPDF_FORMFIELD_COMBOBOX && (flags & FPDF_FORMFLAG_CHOICE_EDIT);
        if (type != FPDF_FORMFIELD_TEXTFIELD && !editable)
            return false;
        // One line takes no line breaks; and words are no more than the field's own limit, when it has one.
        if (!(type == FPDF_FORMFIELD_TEXTFIELD && (flags & FPDF_FORMFLAG_TEXT_MULTILINE)) && f.text.contains(QLatin1Char('\n')))
            return false;
        float most = 0;
        return !(type == FPDF_FORMFIELD_TEXTFIELD && FPDFAnnot_GetNumberValue(annot, "MaxLen", &most) && most > 0 && f.text.size() > most);
    }
    case Fill::Way::Checked:
        return type == FPDF_FORMFIELD_CHECKBOX || (type == FPDF_FORMFIELD_RADIOBUTTON && f.checked);
    case Fill::Way::Chosen: {
        const int count = FPDFAnnot_GetOptionCount(form, annot);
        const bool several = type == FPDF_FORMFIELD_LISTBOX && (flags & FPDF_FORMFLAG_CHOICE_MULTI_SELECT);
        if ((type != FPDF_FORMFIELD_COMBOBOX && type != FPDF_FORMFIELD_LISTBOX) || (!several && f.chosen.size() != 1))
            return false;
        return std::all_of(f.chosen.cbegin(), f.chosen.cend(), [&](int i) { return i < count; });
    }
    }
    return false;
}

// Whether the field now holds what the entry filled it with.
bool holds(FPDF_FORMHANDLE form, FPDF_ANNOTATION annot, const Fill &f)
{
    switch (f.way) {
    case Fill::Way::Text: return valueOf(form, annot) == f.text;
    case Fill::Way::Checked: return bool(FPDFAnnot_IsChecked(form, annot)) == f.checked;
    case Fill::Way::Chosen:
        for (int i = 0; i < FPDFAnnot_GetOptionCount(form, annot); ++i)
            if (bool(FPDFAnnot_IsOptionSelected(form, annot, i)) != f.chosen.contains(i))
                return false;
        return true;
    }
    return false;
}

// Fill it as a person would: focused, its words replaced, ticked by Space, or its options chosen; then let go of, which
// is when PDFium writes the field's value and draws its widgets again.
bool fill(FPDF_FORMHANDLE form, FPDF_PAGE page, FPDF_ANNOTATION annot, const Fill &f)
{
    if (!FORM_SetFocusedAnnot(form, annot))
        return false;
    bool done = true;
    switch (f.way) {
    case Fill::Way::Text:
        // Nothing to select in an empty field is no failure: the words go where the caret is.
        FORM_SelectAllText(form, page);
        FORM_ReplaceSelection(form, page, reinterpret_cast<FPDF_WIDESTRING>(f.text.utf16()));
        break;
    case Fill::Way::Checked:
        if (bool(FPDFAnnot_IsChecked(form, annot)) != f.checked)
            done = FORM_OnChar(form, page, ' ', 0);
        break;
    case Fill::Way::Chosen:
        if (FPDFAnnot_GetFormFieldType(form, annot) == FPDF_FORMFIELD_COMBOBOX || !(FPDFAnnot_GetFormFieldFlags(form, annot) & FPDF_FORMFLAG_CHOICE_MULTI_SELECT)) {
            done = FORM_SetIndexSelected(form, page, f.chosen.first(), true);
        } else {
            for (int i = 0; i < FPDFAnnot_GetOptionCount(form, annot); ++i)
                if (bool(FPDFAnnot_IsOptionSelected(form, annot, i)) != f.chosen.contains(i))
                    done = FORM_SetIndexSelected(form, page, i, f.chosen.contains(i)) && done;
        }
        break;
    }
    return FORM_ForceToKillFocus(form) && done;
}

}  // namespace

QVariantList PdfForms::read(const QString &path, const QString &password, int only)
{
    QVariantList widgets;
    PdfiumLock locked;
    PdfSource source(path);
    if (!source.open())
        return widgets;
    const QByteArray secret = password.toUtf8();
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, password.isEmpty() ? nullptr : secret.constData())};
    if (!document.handle)
        return widgets;
    PdfiumForm form(document.handle);
    if (!form.handle())
        return widgets;
    const bool xfa = isXfa(document.handle);
    const int pages = FPDF_GetPageCount(document.handle);
    for (int i = only < 0 ? 0 : only; i < (only < 0 ? pages : qMin(only + 1, pages)); ++i) {
        PdfiumPage page{FPDF_LoadPage(document.handle, i), form.handle()};
        if (!page.handle)
            continue;
        const QTransform toView = pageSpace(page.handle).inverted();
        for (int a = 0; a < FPDFPage_GetAnnotCount(page.handle); ++a) {
            PdfiumAnnot annot{FPDFPage_GetAnnot(page.handle, a)};
            if (!annot.handle || FPDFAnnot_GetSubtype(annot.handle) != FPDF_ANNOT_WIDGET || !shown(annot.handle))
                continue;
            if (const QVariantMap widget = widgetOf(form.handle(), annot.handle, i, a, toView, xfa); !widget.isEmpty())
                widgets << widget;
        }
    }
    return widgets;
}

bool fillFields(FPDF_DOCUMENT document, FPDF_FORMHANDLE form, const QVariantList &filled, QString *why)
{
    const auto refuse = [&](const QString &text) {
        *why = text;
        return false;
    };
    if (filled.isEmpty())
        return true;
    if (!form)
        return refuse(PdfForms::tr("This PDF's form could not be read."));
    if (isXfa(document))
        return refuse(xfaWhy());

    // Everything checked first: each entry, and the widget it names.
    const QString cannot = PdfForms::tr("What was filled in was not a list QOR Media can write.");
    if (filled.size() > kMostFills)
        return refuse(cannot);
    const int pages = FPDF_GetPageCount(document);
    QList<Fill> fills;
    for (const QVariant &entry : filled) {
        Fill f;
        if (!parse(entry, pages, &f))
            return refuse(cannot);
        PdfiumPage page{FPDF_LoadPage(document, f.page), form};
        if (!page.handle)
            return refuse(PdfForms::tr("A page of this PDF could not be read."));
        PdfiumAnnot annot{f.index < FPDFPage_GetAnnotCount(page.handle) ? FPDFPage_GetAnnot(page.handle, f.index) : nullptr};
        if (!annot.handle || !fits(form, annot.handle, f))
            return refuse(cannot);
        fills << f;
    }

    for (const Fill &f : std::as_const(fills)) {
        PdfiumPage page{FPDF_LoadPage(document, f.page), form};
        PdfiumAnnot annot{page.handle ? FPDFPage_GetAnnot(page.handle, f.index) : nullptr};
        if (!annot.handle || !fill(form, page.handle, annot.handle, f))
            return refuse(PdfForms::tr("A field could not be filled."));
        if (!holds(form, annot.handle, f)) {
            // Words cut short by a limit kept where this cannot read it, on a field above its widget.
            const QString got = f.way == Fill::Way::Text ? valueOf(form, annot.handle) : QString();
            const bool cut = !got.isEmpty() && got.size() < f.text.size() && f.text.startsWith(got);
            return refuse(cut ? PdfForms::tr("Some words are longer than their field takes. Shorten them, and save again.")
                              : PdfForms::tr("A field could not be filled."));
        }
    }
    return true;
}

}  // namespace qormedia
