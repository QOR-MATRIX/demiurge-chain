// Highlighter: the editor's colours for Markdown and code (ADR-094).
//
// Markdown: headings, emphasis, inline code, links, lists and quotes. Code: keywords common to the languages people
// keep in their folders (C-like, Rust, Python, JavaScript, shell, QML), strings, numbers and comments, with block
// comments followed across lines. Colours come from the launcher's theme.

#pragma once

#include <QRegularExpression>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QVariantMap>

namespace qormedia {

class Highlighter : public QSyntaxHighlighter
{
    Q_OBJECT

public:
    explicit Highlighter(QObject *parent = nullptr);

    void setLanguage(const QString &language);
    void setColors(const QVariantMap &colors);

protected:
    void highlightBlock(const QString &text) override;

private:
    struct Rule {
        QRegularExpression pattern;
        QTextCharFormat format;
    };
    void build();

    QString m_language = QStringLiteral("plain");
    QVariantMap m_colors;
    QList<Rule> m_rules;
    QTextCharFormat m_comment;
};

}  // namespace qormedia
