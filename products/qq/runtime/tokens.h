// Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,
// run `node docs/design/tokens.mjs`, and commit what it writes.

#pragma once

#include <QtGui/qrgb.h>

namespace qq::tokens {

// Colours, as QRgb (0xrrggbb).
inline constexpr QRgb colorVoid = 0x06070a;
inline constexpr QRgb colorBase = 0x0b0e14;
inline constexpr QRgb colorWell = 0x0e121a;
inline constexpr QRgb colorSurface = 0x131821;
inline constexpr QRgb colorRaised = 0x1b212d;
inline constexpr QRgb colorEdge = 0x2a3140;
inline constexpr QRgb colorEdgeSoft = 0x1f2530;
inline constexpr QRgb colorInk = 0xf3ede1;
inline constexpr QRgb colorInkBody = 0xd6d1c6;
inline constexpr QRgb colorInkMuted = 0x9ba2ad;
inline constexpr QRgb colorInkFaint = 0x868f9d;
inline constexpr QRgb colorAccent = 0xff6a00;
inline constexpr QRgb colorAccentBright = 0xff9142;
inline constexpr QRgb colorAccentDim = 0xb44a00;
inline constexpr QRgb colorOnAccent = 0x06070a;
inline constexpr QRgb colorGilt = 0xc9a45c;
inline constexpr QRgb colorGiltBright = 0xead39a;
inline constexpr QRgb colorGiltDim = 0x7a6236;
inline constexpr QRgb colorOk = 0x22c55e;
inline constexpr QRgb colorWarn = 0xe6a817;
inline constexpr QRgb colorBad = 0xd57889;

// Faces: family names, as each is registered once its file is loaded.
inline constexpr const char *fontDisplay = "Cinzel";
inline constexpr const char *fontInterface = "Barlow Semi Condensed";
inline constexpr const char *fontBody = "Barlow";
inline constexpr const char *fontMono = "IBM Plex Mono";

// The type scale, in pixels; tracking, in ems.
inline constexpr int sizeMicro = 10;
inline constexpr int sizeCaption = 11;
inline constexpr int sizeUi = 12;
inline constexpr int sizeBody = 13;
inline constexpr int sizeTitle = 15;
inline constexpr int sizeHeading = 18;
inline constexpr int sizeDisplay = 24;
inline constexpr int sizeFigure = 38;
inline constexpr double trackingHeading = 0.06;
inline constexpr double trackingLabel = 0.1;
inline constexpr double trackingEyebrow = 0.18;

// Corners, in pixels.
inline constexpr int radiusControl = 8;
inline constexpr int radiusPanel = 12;

// Motion: durations in milliseconds, and the one easing as a cubic Bézier curve.
inline constexpr int motionQuick = 150;
inline constexpr int motionStandard = 260;
inline constexpr int motionEnter = 420;
inline constexpr int motionSheen = 760;
inline constexpr double easeBezier[4] = {0.16, 1, 0.3, 1};

// The focus ring.
inline constexpr int focusWidth = 2;
inline constexpr int focusHalo = 5;
inline constexpr QRgb focusColor = colorAccentBright;

}  // namespace qq::tokens
