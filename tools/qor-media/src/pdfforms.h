// PdfForms: a PDF's form read and filled through PDFium's own interface (ADR-094 decision 3, L9.5).
//
// A form is fields, each with a name, a kind and a value, shown on the pages by widgets: a field on two pages has a widget
// on each, and a radio group one for each button. The reader reads a page's widgets as maps (`read`) and holds what the
// person fills until the PDF is saved, as entries that each name the widget it was filled through and one of:
//   text     a text field's words, or an editable list's own words
//   checked  a check box ticked (true) or not; a radio button chosen (true only: a group is not unchosen)
//   chosen   the places of a list's options chosen: one for a drop-down list, any for a list that allows several
// with `page` (the widget's page in the file as it is now) and `index` (its place among the page's annotations).
//
// Each is filled through PDFium's form as a person filling it in any reader would (FORM_ReplaceSelection, FORM_OnChar,
// FORM_SetIndexSelected), so PDFium writes the field's value and draws its widgets again, in the field's own font. The
// form's scripts are not run, since this PDFium has no JavaScript: a total a script works out stays as it was, and a
// button does nothing. A form that keeps its answers in XFA (the IRS's do, beside their fields) is shown and not filled:
// Acrobat shows the answers in its XFA, which this PDFium cannot write. Nor is a password field: PDFium would write the
// password into the file.

#pragma once

#include <QObject>
#include <QString>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

#include <fpdfview.h>
#include <fpdf_formfill.h>

namespace qormedia {

class PdfForms : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    explicit PdfForms(QObject *parent = nullptr) : QObject(parent) {}

    /// The widgets of the fields `path` holds on `page`, or on every page when it is -1, opened with `password` when it
    /// has one. Each is {page, index, name, label, kind, rect, value, checked, state, options, chosen, fillable, why, xfa,
    /// required, multiline, secret, comb, maxLength, editable, multiple, fontSize, colour, align}:
    ///   name      the field's full name; widgets of one field share it, and so share its value
    ///   label     what the form calls it for a person (its /TU), or nothing
    ///   kind      "text", "check", "radio", "combo" (a drop-down list), "list", "button" or "signature"
    ///   rect      where the widget is, in the page's points as Qt PDF lays it out (pageSpace, pdfium.h)
    ///   value     the field's value: its words, the option chosen, or a box's or group's state ("Off" when none)
    ///   checked   a check box or radio button on; `state` the state it sets its field to when it is
    ///   options   a list's options, as a person reads them; `chosen` the places of those chosen
    ///   fillable  whether QOR Media fills it; `why` says why not, when it is not; `xfa` whether its form keeps its
    ///             answers in XFA
    ///   fontSize  its words' size in points (0: sized to fit); `colour` their colour, "#rrggbb"; `align` 0 left,
    ///             1 centre, 2 right
    /// Hidden widgets are left out. Empty when it cannot be read. Reading a page loads all it draws, so the reader asks for a
    /// page as it shows it.
    Q_INVOKABLE static QVariantList read(const QString &path, const QString &password = QString(), int page = -1);
};

/// Fill `document`'s fields as `filled` says, entries as PdfForms describes them, through `form`, the document's own.
/// The caller has the document open under PdfiumLock. Every entry is checked before any field changes; false, with
/// `why`, when one is not a field this fills, or not one it fills that way.
bool fillFields(FPDF_DOCUMENT document, FPDF_FORMHANDLE form, const QVariantList &filled, QString *why);

}  // namespace qormedia
