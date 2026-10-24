// Values the reader hands over from QML, checked before anything is written: plain data whatever wraps it, numbers that
// are finite and near enough to be a place on a page or a size, whole numbers, and lists. Shared by PdfMarks and
// PdfForms.

#pragma once

#include <QJSValue>
#include <QMetaType>
#include <QString>
#include <QVariant>
#include <QVariantList>

#include <cmath>
#include <limits>

namespace qormedia::values {

// Every number is a place on a page, a size or a count: none is near this.
inline constexpr double kFarthest = 1e6;

// A value from QML as plain data, whatever wraps it.
inline QVariant plain(const QVariant &value)
{
    return value.metaType() == QMetaType::fromType<QJSValue>() ? value.value<QJSValue>().toVariant() : value;
}

// The same all the way down: maps and lists of plain data, which may be kept and read on another thread, as a script's
// value may not.
inline QVariant plainAll(const QVariant &value)
{
    const QVariant v = plain(value);
    if (v.metaType() == QMetaType::fromType<QVariantMap>()) {
        QVariantMap map = v.toMap();
        for (auto it = map.begin(); it != map.end(); ++it)
            *it = plainAll(*it);
        return map;
    }
    if (v.metaType() == QMetaType::fromType<QVariantList>()) {
        QVariantList items = v.toList();
        for (QVariant &item : items)
            item = plainAll(item);
        return items;
    }
    return v;
}

inline bool within(double v)
{
    return std::isfinite(v) && std::abs(v) <= kFarthest;
}

inline bool number(const QVariant &value, double *out)
{
    const QVariant v = plain(value);
    bool ok = false;
    const double n = v.isValid() ? v.toDouble(&ok) : 0;
    if (!ok || !within(n))
        return false;
    *out = n;
    return true;
}

inline bool whole(const QVariant &value, int *out)
{
    double n = 0;
    // In brackets: Windows' max macro arrives with PDFium's header wherever this follows it.
    if (!number(value, &n) || n != std::floor(n) || n < 0 || n > (std::numeric_limits<int>::max)())
        return false;
    *out = int(n);
    return true;
}

inline bool list(const QVariant &value, QVariantList *out)
{
    const QVariant v = plain(value);
    if (v.metaType() == QMetaType::fromType<QString>() || !v.canConvert<QVariantList>())
        return false;
    *out = v.toList();
    return true;
}

}  // namespace qormedia::values
