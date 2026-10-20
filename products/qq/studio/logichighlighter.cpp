#include "logichighlighter.h"

#include <QColor>
#include <QTextDocument>

namespace qq {

namespace {

class Highlighter : public QSyntaxHighlighter
{
public:
    explicit Highlighter(QTextDocument *document) : QSyntaxHighlighter(document)
    {
        auto format = [](const char *colour, bool bold = false, bool italic = false) {
            QTextCharFormat f;
            f.setForeground(QColor(QLatin1String(colour)));
            if (bold)
                f.setFontWeight(QFont::DemiBold);
            f.setFontItalic(italic);
            return f;
        };
        // The Studio's inks: gilt for the language's own words, ember for QQ's logic API, sage for strings, a pale
        // blue for numbers and types, and a quiet grey for comments.
        m_rules.append({QRegularExpression(QStringLiteral(
                            R"(\b(import|property|readonly|function|signal|const|let|var|if|else|for|of|in|while|)"
                            R"(return|break|continue|new|true|false|null|undefined|this|typeof|switch|case|default)\b)")),
                        format("#e3c27a", true)});
        m_rules.append({QRegularExpression(QStringLiteral(
                            R"(\b(on[A-Z]\w*|spawn|remove|removing|after|every|cancel|ray|tween|machine|play|)"
                            R"(setVolume|volumeOf|entity|random|randomInt|overlaps|outcome|showScreen|hideScreen|)"
                            R"(onlyScreen|pause|resume|held|rebind|bindingOf|pointing|part|push|applyForce|)"
                            R"(setVelocity|setSpin|target|scene|elapsed|steps)\b)")),
                        format("#ff8a3d")});
        m_rules.append({QRegularExpression(QStringLiteral(R"(\b[A-Z][A-Za-z0-9]*\b)")), format("#8ec5ff")});
        m_rules.append({QRegularExpression(QStringLiteral(R"(\b\d+(\.\d+)?\b)")), format("#8ec5ff")});
        m_rules.append({QRegularExpression(QStringLiteral(R"("([^"\\]|\\.)*"|'([^'\\]|\\.)*')")), format("#9fd49a")});
        m_comment = format("#6c7280", false, true);
    }

protected:
    void highlightBlock(const QString &text) override
    {
        for (const Rule &rule : std::as_const(m_rules)) {
            auto it = rule.pattern.globalMatch(text);
            while (it.hasNext()) {
                const auto m = it.next();
                setFormat(int(m.capturedStart()), int(m.capturedLength()), rule.format);
            }
        }
        // Comments last, over what is in them: // to the line's end, and /* */ across lines.
        setCurrentBlockState(0);
        qsizetype start = previousBlockState() == 1 ? 0 : text.indexOf(QStringLiteral("/*"));
        while (start >= 0) {
            const qsizetype end = text.indexOf(QStringLiteral("*/"), start + (previousBlockState() == 1 && start == 0 ? 0 : 2));
            const qsizetype length = end < 0 ? text.size() - start : end - start + 2;
            if (end < 0)
                setCurrentBlockState(1);
            setFormat(int(start), int(length), m_comment);
            start = end < 0 ? -1 : text.indexOf(QStringLiteral("/*"), start + length);
        }
        // A // outside a string starts a comment to the line's end.
        bool inString = false;
        QChar quote;
        for (qsizetype i = 0; i + 1 < text.size(); ++i) {
            const QChar c = text.at(i);
            if (inString) {
                if (c == QLatin1Char('\\'))
                    ++i;
                else if (c == quote)
                    inString = false;
            } else if (c == QLatin1Char('"') || c == QLatin1Char('\'')) {
                inString = true;
                quote = c;
            } else if (c == QLatin1Char('/') && text.at(i + 1) == QLatin1Char('/') && format(int(i)) != m_comment) {
                setFormat(int(i), int(text.size() - i), m_comment);
                break;
            }
        }
    }

private:
    struct Rule {
        QRegularExpression pattern;
        QTextCharFormat format;
    };
    QList<Rule> m_rules;
    QTextCharFormat m_comment;
};

}  // namespace

LogicHighlighter::LogicHighlighter(QObject *parent) : QObject(parent) {}

void LogicHighlighter::setDocument(QQuickTextDocument *document)
{
    if (document == m_document)
        return;
    delete m_highlighter;
    m_document = document;
    if (m_document && m_document->textDocument())
        m_highlighter = new Highlighter(m_document->textDocument());
    emit documentChanged();
}

}  // namespace qq
