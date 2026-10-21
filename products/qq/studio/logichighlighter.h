// LogicHighlighter: colours a logic file as the Studio's logic editor shows it (DIRECTION P3.15): QML and JavaScript's
// words, QQ's logic API (onStep, spawn, tween…), strings, numbers and comments, in the Studio's inks.
//
//   TextArea { id: code }
//   LogicHighlighter { document: code.textDocument }

#pragma once

#include <QObject>
#include <QPointer>
#include <QQuickTextDocument>
#include <QRegularExpression>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QtQml/qqmlregistration.h>

namespace qq {

class LogicHighlighter : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// The text area's document to colour.
    Q_PROPERTY(QQuickTextDocument *document READ document WRITE setDocument NOTIFY documentChanged)

public:
    explicit LogicHighlighter(QObject *parent = nullptr);

    QQuickTextDocument *document() const { return m_document; }
    void setDocument(QQuickTextDocument *document);

signals:
    void documentChanged();

private:
    QPointer<QQuickTextDocument> m_document;
    QPointer<QSyntaxHighlighter> m_highlighter;
};

}  // namespace qq
