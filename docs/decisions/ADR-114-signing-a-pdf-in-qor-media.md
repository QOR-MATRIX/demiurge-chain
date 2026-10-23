# ADR-114: Signing a PDF in QOR Media: a drawn signature now, certified signatures decided later

**Status:** **Decision 1 accepted**, 23 October 2026, by the owner. Asked what ADR-094 meant by a PDF being "signed",
the owner chose **"Both, drawn first"**: build the drawn signature now, with the rest of marking up, and write a decision
record for certified signatures to decide later. **Decision 2 is proposed, not decided**: it sets out what certified
signatures need and the options, and nothing is built on it until the owner accepts, amends or rejects it
(`docs/decisions/README.md`). It decides no economic value.

**Relates to:** [ADR-094](ADR-094-the-library-and-qor-media.md) decision 3 (PDFs viewed, annotated, highlighted,
filled, **signed**, and their pages changed; Qt PDF to show, PDFium to change), which this makes precise without
changing it; [ADR-056](ADR-056-no-lock-screen.md) (keys in the operating system's keychain) and
[ADR-112](ADR-112-the-controllers-own-key-signs-every-agent-authorisation.md) (a person's own key signs what is theirs),
which decision 2 weighs.

## Context

"Signed" names two different things. Most PDF editors' *Fill & Sign* places a picture of a signature on the page, drawn
with the pointer or typed in a handwriting face. It is what most people mean, and it proves nothing about who placed it
or whether the PDF changed afterwards. A **certified digital signature** (PDF's own, PAdES) is a cryptographic seal: a
signature over the file's bytes made with a private key, whose X.509 certificate names the signer; PDF readers check it
and show it broken if the file changes. It needs a certificate for each signer, usually a timestamp from a timestamping
authority (RFC 3161), and, to stay checkable for years, the validation data stored with it.

What QOR Media has: PDFium, which adds annotations and can *read* a PDF's digital signatures
(`fpdf_signature.h`) but cannot create one; Qt, which has no PDF signing either. A person's Demiurge key is Sr25519,
which no PDF reader verifies.

## Decision

1. **Accepted: signing a PDF in QOR Media is a drawn or typed signature, placed on a page.** The person draws it with the
   pointer, a pen or a finger, or types their name in a handwriting face; places it on any page; moves and sizes it; and
   saves it into the PDF as a mark, through PDFium, like the reader's other marks. It may be kept in the person's
   profile to place again, never sent anywhere. Where it is placed, QOR Media says plainly that it is a picture of a
   signature and does not certify the PDF.
2. **Proposed: certified digital signatures, as their own later step.** The options:
   - **A. The person's own certificate, from Windows' certificate store** (an identity card, a smart card or a
     certificate their employer or a certificate authority issued them). QOR Media signs through Windows' own
     cryptography, so the private key never leaves where it lives, and adds a timestamp from a public timestamping
     authority. No certificate authority is run by Demiurge. Needs a PDF library that writes signatures (for example
     PoDoFo, LGPL), decided with its licence.
   - **B. A certificate authority run for QOR IDs.** Each verified QOR ID gets a certificate. Readers trust it only
     where people add Demiurge's root, so Acrobat shows it unknown; and running an authority is a standing security,
     audit and legal duty.
   - **C. Certificates bought for each person from a trusted authority** (Acrobat's trust list). Signatures verify
     everywhere; each costs money and an identity check outside Demiurge.
   - **D. Not at all:** QOR Media shows a PDF's signatures (read through PDFium) and leaves certifying to other tools.

   **Recommendation: A, with D's reading of signatures first.** It adds no authority to run and no cost, signs with
   keys people already hold, and verifies wherever their certificate is trusted.

## Consequences

- QOR Media's marking up includes the drawn signature (L9.5). Its tests hold that it is saved into the PDF and shown
  after opening it again.
- Nothing certifies a PDF until decision 2 is accepted. The roadmap's L9.5 is ticked on the drawn signature; certified
  signing, if accepted, becomes an item of its own.
- Whatever decision 2 becomes, a person's Demiurge key is not used to sign PDFs, since no PDF reader can check it.
