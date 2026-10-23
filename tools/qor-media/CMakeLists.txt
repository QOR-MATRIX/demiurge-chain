# QOR Media (ADR-094): the Library's player, viewer and editor, in Qt 6, C++20 and QML, docked inside the QOR Launcher.
#
#   src/    the launcher link (media.next), docking into the launcher's window, files, text and highlighting, a PDF's
#           pages drawn with their marks, and changed through PDFium
#   qml/    the module "QOR.Media": the window, the player, the viewer, the editor, the reader (Qt PDF); Tokens.qml, which
#           docs/design/tokens.mjs generates from docs/design/tokens.json, and Theme, which binds to it (ADR-108)
#   fonts/  Barlow, Barlow Semi Condensed and IBM Plex Mono, under the SIL Open Font License, copied by tokens.mjs
#   tests/  Qt Test suites
#
# Configure with the Qt the owner installed (commercial licence), MSVC 2022 and Ninja; build.ps1 does all of it:
#
#   pwsh tools/qor-media/build.ps1            # build and test
#   pwsh tools/qor-media/build.ps1 -Deploy    # and make the self-contained copy the launcher opens

cmake_minimum_required(VERSION 3.22)

project(QorMedia VERSION 0.1.0 DESCRIPTION "QOR Media" LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.12 REQUIRED COMPONENTS Core Gui Multimedia Network Pdf PdfQuick Qml Quick QuickControls2 QuickShapes Svg Test)
# PDFium's own interface, for changing a PDF (ADR-094 decision 3). build.ps1 fetches the pinned release and points
# PDFium_DIR at it; its DLL goes beside every program built here.
find_package(PDFium REQUIRED)
file(COPY "${PDFium_LIBRARY}" DESTINATION "${CMAKE_BINARY_DIR}")

qt_standard_project_setup(REQUIRES 6.12)

set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml")

if(MSVC)
    add_compile_options(/W4 /permissive- /utf-8 /external:anglebrackets /external:W0 /wd4702)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Everything but main(), so the tests use exactly what the application does.
qt_add_library(qormedia_core STATIC
    src/medialink.h src/medialink.cpp
    src/files.h src/files.cpp
    src/textfile.h src/textfile.cpp
    src/highlighter.h src/highlighter.cpp
    src/dock.h src/dock.cpp
    src/look.h src/look.cpp
    src/pdfpages.h src/pdfpages.cpp
    src/pdfdrawing.h src/pdfdrawing.cpp
    src/tokens.h
)
target_include_directories(qormedia_core PUBLIC src)
target_link_libraries(qormedia_core PUBLIC Qt6::Core Qt6::Gui Qt6::Network Qt6::Pdf Qt6::Qml Qt6::Quick pdfium)

# Before the module, or Theme is registered as an ordinary type.
set_source_files_properties(qml/Theme.qml qml/Tokens.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_qml_module(qormedia_core
    URI QOR.Media
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/Tokens.qml
        qml/Theme.qml
        qml/FocusRing.qml
        qml/TextButton.qml
        qml/Player.qml
        qml/Viewer.qml
        qml/Editor.qml
        qml/Reader.qml
        qml/Elsewhere.qml
        qml/TopBar.qml
        qml/IconButton.qml
    RESOURCES
        qml/icons/back.svg
        qml/icons/play.svg
        qml/icons/pause.svg
        qml/icons/volume.svg
        qml/icons/mute.svg
        qml/icons/full.svg
        qml/icons/subtitles.svg
        qml/icons/tracks.svg
        qml/icons/prev.svg
        qml/icons/next.svg
        qml/icons/rotate.svg
        qml/icons/save.svg
        qml/icons/preview.svg
        qml/icons/open.svg
        qml/icons/find.svg
        qml/icons/zoomin.svg
        qml/icons/zoomout.svg
        qml/icons/fit.svg
        qml/icons/pages.svg
        qml/icons/up.svg
        qml/icons/down.svg
        qml/icons/delete.svg
        qml/icons/undo.svg
        fonts/Barlow-Regular.ttf
        fonts/Barlow-Medium.ttf
        fonts/BarlowSemiCondensed-Regular.ttf
        fonts/BarlowSemiCondensed-Medium.ttf
        fonts/BarlowSemiCondensed-SemiBold.ttf
        fonts/IBMPlexMono-Regular.ttf
        fonts/IBMPlexMono-Medium.ttf
)

qt_add_executable(qor-media WIN32 src/main.cpp)
target_link_libraries(qor-media PRIVATE qormedia_core qormedia_coreplugin Qt6::Multimedia Qt6::PdfQuick Qt6::QuickControls2 Qt6::QuickShapes Qt6::Svg)

enable_testing()
foreach(suite medialink files textfile playback look reader pdfpages)
    qt_add_executable(tst_${suite} tests/tst_${suite}.cpp)
    target_link_libraries(tst_${suite} PRIVATE qormedia_core qormedia_coreplugin Qt6::Multimedia Qt6::PdfQuick Qt6::QuickControls2 Qt6::QuickShapes Qt6::Test)
    add_test(NAME ${suite} COMMAND tst_${suite})
endforeach()
# The look's test reads the source itself, not the file generated from it.
target_compile_definitions(tst_look PRIVATE
    QORMEDIA_TOKENS="${CMAKE_CURRENT_SOURCE_DIR}/../../docs/design/tokens.json")
