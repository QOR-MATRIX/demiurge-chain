# QQ Studio, the native editor (ADR-083 decision 2). Its own process, started by the launcher; it never holds a key.
#
# The Studio's QML is a library (module "QQ.Studio") so tests can load the real window; the executable is main.cpp.

qt_add_library(qqstudio STATIC
    # The Studio's side of the QOR host protocol: publishing asks the launcher that started it (ADR-088).
    launcherlink.h launcherlink.cpp
    # The logic editor's colours (P3.15).
    logichighlighter.h logichighlighter.cpp
)

qt_add_qml_module(qqstudio
    URI QQ.Studio
    VERSION 1.0
    QML_FILES
        Main.qml
        Inspector.qml
        Gizmo.qml
        AgentPanel.qml
        History.qml
        ProjectPanel.qml
        LogicEditor.qml
        StatsOverlay.qml
        StudioButton.qml
)

# The Studio's reading and numeric faces (Look's body and mono, ADR-108), copied here from docs/design/fonts/ by
# docs/design/tokens.mjs: only the Studio carries them, so the Player and its browser build do not grow.
qt_add_resources(qqstudio "studiofonts"
    PREFIX "/qq/studio-fonts"
    BASE fonts
    FILES
        fonts/Barlow-Regular.ttf
        fonts/Barlow-Medium.ttf
        fonts/IBMPlexMono-Regular.ttf
        fonts/IBMPlexMono-Medium.ttf
)

# The model a new scene opens with: the same fixture the tests render, so what is checked is what is shown.
qt_add_resources(qqstudio "samples"
    PREFIX /samples
    BASE "${PROJECT_SOURCE_DIR}/tests/fixtures"
    FILES "${PROJECT_SOURCE_DIR}/tests/fixtures/orb.gltf"
)

target_include_directories(qqstudio PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")

target_link_libraries(qqstudio PUBLIC
    Qt6::Gui Qt6::Quick Qt6::Quick3D Qt6::QuickControls2 Qt6::Network
    qqruntime qqruntimeplugin
)

qt_add_executable(qq-studio WIN32 main.cpp)
target_link_libraries(qq-studio PRIVATE qqstudio qqstudioplugin qqagent)
