#include "files.h"

#include <QCollator>
#include <QDesktopServices>
#include <QDir>
#include <QFileInfo>
#include <QSet>

#include <algorithm>

namespace qormedia {

namespace {

const QSet<QString> &video()
{
    static const QSet<QString> s{"mkv", "mp4", "m4v", "mov", "avi", "webm", "wmv", "flv", "ts", "m2ts", "mts",
                                 "mpg", "mpeg", "3gp", "ogv", "vob", "divx", "f4v", "asf"};
    return s;
}
const QSet<QString> &audio()
{
    static const QSet<QString> s{"mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "wma", "aiff", "aif",
                                 "alac", "ape", "dsf", "mka", "ac3", "dts"};
    return s;
}
const QSet<QString> &picture()
{
    static const QSet<QString> s{"jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff", "svg", "ico", "tga",
                                 "heic", "heif", "avif", "jxl", "raw", "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2"};
    return s;
}
const QSet<QString> &code()
{
    static const QSet<QString> s{"rs", "tsx", "js", "py", "cpp", "c", "h", "hpp", "cs", "java", "go", "toml",
                                 "yaml", "yml", "ini", "sql", "qml", "sh", "ps1", "css", "json", "xml", "html", "htm"};
    return s;
}
const QSet<QString> &document()
{
    static const QSet<QString> s{"pdf", "txt", "md", "markdown", "rtf", "doc", "docx", "odt", "xls", "xlsx", "ods",
                                 "ppt", "pptx", "odp", "csv", "log", "epub"};
    return s;
}

QString extension(const QString &path)
{
    return QFileInfo(path).suffix().toLower();
}

}  // namespace

QString Files::kindOf(const QString &path)
{
    const QString ext = extension(path);
    if (video().contains(ext))
        return QStringLiteral("video");
    if (audio().contains(ext))
        return QStringLiteral("audio");
    if (picture().contains(ext))
        return QStringLiteral("picture");
    if (document().contains(ext) || code().contains(ext))
        return QStringLiteral("document");
    return QStringLiteral("other");
}

bool Files::isText(const QString &path)
{
    const QString ext = extension(path);
    return ext == QLatin1String("txt") || ext == QLatin1String("md") || ext == QLatin1String("markdown")
        || ext == QLatin1String("csv") || ext == QLatin1String("log") || code().contains(ext);
}

QString Files::textLanguage(const QString &path)
{
    const QString ext = extension(path);
    if (ext == QLatin1String("md") || ext == QLatin1String("markdown"))
        return QStringLiteral("markdown");
    if (code().contains(ext))
        return QStringLiteral("code");
    return QStringLiteral("plain");
}

QStringList Files::siblings(const QString &path)
{
    const QFileInfo info(path);
    const QString kind = kindOf(path);
    QStringList found;
    const QFileInfoList entries = info.dir().entryInfoList(QDir::Files | QDir::Readable);
    for (const QFileInfo &e : entries) {
        if (e.fileName().startsWith(QLatin1Char('.')) || kindOf(e.filePath()) != kind)
            continue;
        // The file asked about keeps the spelling it was asked with, so what is reported about it matches.
        found << (e == info ? path : QDir::toNativeSeparators(e.filePath()));
    }
    QCollator collator;
    collator.setNumericMode(true);
    collator.setCaseSensitivity(Qt::CaseInsensitive);
    std::sort(found.begin(), found.end(), [&](const QString &a, const QString &b) {
        return collator.compare(QFileInfo(a).fileName(), QFileInfo(b).fileName()) < 0;
    });
    return found;
}

QUrl Files::urlOf(const QString &path)
{
    return QUrl::fromLocalFile(path);
}

QString Files::nameOf(const QString &path)
{
    return QFileInfo(path).fileName();
}

bool Files::openElsewhere(const QString &path)
{
    return QDesktopServices::openUrl(QUrl::fromLocalFile(path));
}

}  // namespace qormedia
