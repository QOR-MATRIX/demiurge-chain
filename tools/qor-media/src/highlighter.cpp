#include "highlighter.h"

#include <QColor>
#include <QFont>

namespace qormedia {

namespace {

QColor color(const QVariantMap &colors, const char *name, const char *fallback)
{
    const QColor c(colors.value(QString::fromLatin1(name)).toString());
    return c.isValid() ? c : QColor(QString::fromLatin1(fallback));
}

QTextCharFormat styled(const QColor &c, bool bold = false, bool italic = false)
{
    QTextCharFormat f;
    f.setForeground(c);
    if (bold)
        f.setFontWeight(QFont::DemiBold);
    f.setFontItalic(italic);
    return f;
}

}  // namespace

Highlighter::Highlighter(QObject *parent) : QSyntaxHighlighter(parent)
{
    build();
}

void Highlighter::setLanguage(const QString &language)
{
    m_language = language;
    build();
}

void Highlighter::setColors(const QVariantMap &colors)
{
    m_colors = colors;
    build();
}

void Highlighter::build()
{
    m_rules.clear();
    const QColor accent = color(m_colors, "accent-bright", "#ff8a33");
    const QColor strong = color(m_colors, "ink", "#ffffff");
    const QColor muted = color(m_colors, "ink-muted", "#9aa3b2");
    const QColor quiet = color(m_colors, "ink-faint", "#6b7385");
    const QColor warm = color(m_colors, "accent", "#ff6a00");
    m_comment = styled(quiet, false, true);

    const auto add = [this](const char *pattern, const QTextCharFormat &f) {
        m_rules.append({QRegularExpression(QString::fromLatin1(pattern)), f});
    };
    if (m_language == QLatin1String("markdown")) {
        add(R"(^#{1,6}\s.*$)", styled(accent, true));
        add(R"(\*\*[^*]+\*\*|__[^_]+__)", styled(strong, true));
        add(R"((?<![*\w])\*[^*\s][^*]*\*(?!\*)|(?<![_\w])_[^_\s][^_]*_(?!_))", styled(strong, false, true));
        add(R"(`[^`]+`)", styled(warm));
        add(R"(\[[^\]]+\]\([^)]+\))", styled(accent));
        add(R"(^\s*([-*+]|\d+\.)\s)", styled(warm, true));
        add(R"(^>.*$)", styled(muted, false, true));
    } else if (m_language == QLatin1String("code")) {
        add(R"(\b(fn|let|mut|pub|struct|enum|impl|trait|use|mod|match|if|else|for|while|loop|return|async|await|)"
            R"(const|static|class|def|import|from|as|in|function|var|new|this|self|Self|true|false|null|None|nil|)"
            R"(void|int|bool|char|auto|namespace|template|typename|public|private|protected|try|catch|throw|)"
            R"(switch|case|break|continue|yield|lambda|with|property|signal|readonly|export|default|interface|type)\b)",
            styled(accent, true));
        add(R"(\b[0-9][0-9_]*(\.[0-9_]+)?([eE][+-]?[0-9]+)?\b|\b0x[0-9a-fA-F_]+\b)", styled(warm));
        add(R"("(\\.|[^"\\])*"|'(\\.|[^'\\])*')", styled(muted));
        add(R"((//|#(?![!\[])).*$|--\s.*$)", m_comment);
    }
    rehighlight();
}

void Highlighter::highlightBlock(const QString &text)
{
    for (const Rule &rule : std::as_const(m_rules)) {
        auto it = rule.pattern.globalMatch(text);
        while (it.hasNext()) {
            const auto m = it.next();
            setFormat(int(m.capturedStart()), int(m.capturedLength()), rule.format);
        }
    }
    if (m_language != QLatin1String("code"))
        return;
    // Block comments, followed across lines: state 1 means a block comment is open at the end of this line.
    setCurrentBlockState(0);
    qsizetype start = previousBlockState() == 1 ? 0 : text.indexOf(QStringLiteral("/*"));
    while (start >= 0) {
        const qsizetype end = text.indexOf(QStringLiteral("*/"), start + (previousBlockState() == 1 && start == 0 ? 0 : 2));
        qsizetype length;
        if (end < 0) {
            setCurrentBlockState(1);
            length = text.size() - start;
        } else {
            length = end - start + 2;
        }
        setFormat(int(start), int(length), m_comment);
        start = text.indexOf(QStringLiteral("/*"), start + length);
    }
}

}  // namespace qormedia
