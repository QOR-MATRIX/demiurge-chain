#include "textfile.h"

#include "files.h"
#include "highlighter.h"

#include <QFile>
#include <QFileInfo>
#include <QSaveFile>
#include <QStringDecoder>
#include <QStringEncoder>

namespace qormedia {

TextFile::TextFile(QObject *parent) : QObject(parent) {}

QString TextFile::load(const QString &path)
{
    m_path = path;
    m_error.clear();
    m_language = Files::textLanguage(path);
    m_loaded = m_crlf = m_bom = m_utf16 = false;
    QFile file(path);
    if (QFileInfo(path).size() > Limit) {
        m_error = tr("This file is larger than the editor opens (16 MB).");
        emit changed();
        return {};
    }
    if (!file.open(QIODevice::ReadOnly)) {
        m_error = tr("This file could not be opened: %1").arg(file.errorString());
        emit changed();
        return {};
    }
    QByteArray bytes = file.readAll();
    QString text;
    if (bytes.startsWith("\xFF\xFE") || bytes.startsWith("\xFE\xFF")) {
        m_utf16 = m_bom = true;
        text = QStringDecoder(QStringDecoder::Utf16)(bytes);
    } else {
        if (bytes.startsWith("\xEF\xBB\xBF")) {
            m_bom = true;
            bytes.remove(0, 3);
        }
        // A NUL byte in a file not marked UTF-16 means it is not text.
        if (bytes.left(8192).contains('\0')) {
            m_error = tr("This file is not text, so the editor does not open it.");
            emit changed();
            return {};
        }
        text = QStringDecoder(QStringDecoder::Utf8)(bytes);
    }
    const qsizetype firstBreak = text.indexOf(QLatin1Char('\n'));
    m_crlf = firstBreak > 0 && text.at(firstBreak - 1) == QLatin1Char('\r');
    text.replace(QStringLiteral("\r\n"), QStringLiteral("\n"));
    m_loaded = true;
    emit changed();
    return text;
}

bool TextFile::save(const QString &text)
{
    if (!m_loaded)
        return false;
    QString out = text;
    if (m_crlf)
        out.replace(QLatin1Char('\n'), QStringLiteral("\r\n"));
    QByteArray bytes;
    if (m_utf16) {
        QStringEncoder encode(QStringEncoder::Utf16, QStringEncoder::Flag::WriteBom);
        bytes = encode(out);
    } else {
        bytes = out.toUtf8();
        if (m_bom)
            bytes.prepend("\xEF\xBB\xBF");
    }
    QSaveFile file(m_path);
    if (!file.open(QIODevice::WriteOnly) || file.write(bytes) != bytes.size() || !file.commit()) {
        m_error = tr("This file could not be saved: %1").arg(file.errorString());
        emit changed();
        return false;
    }
    m_error.clear();
    emit changed();
    return true;
}

void TextFile::highlight(QQuickTextDocument *document, const QVariantMap &colors)
{
    if (!document || !document->textDocument())
        return;
    if (!m_highlighter)
        m_highlighter = new Highlighter(this);
    m_highlighter->setLanguage(m_language);
    m_highlighter->setColors(colors);
    m_highlighter->setDocument(document->textDocument());
    m_highlighter->rehighlight();
}

}  // namespace qormedia
