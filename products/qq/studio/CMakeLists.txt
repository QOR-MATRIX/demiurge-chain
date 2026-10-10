# QQ Studio, the native editor (ADR-083 decision 2). Its own process, started by the launcher; it never holds a key.
#
# The Studio's QML is a library (module "QQ.Studio") so tests can load the real window; the executable is main.cpp.

qt_add_library(qqstudio STATIC)

qt_add_qml_module(qqstudio
    URI QQ.Studio
    VERSION 1.0
    QML_FILES
        Main.qml
        Inspector.qml
        Gizmo.qml
        AgentPanel.qml
)

# The model a new scene opens with: the same fixture the tests render, so what is checked is what is shown.
qt_add_resources(qqstudio "samples"
    PREFIX /samples
    BASE "${PROJECT_SOURCE_DIR}/tests/fixtures"
    FILES "${PROJECT_SOURCE_DIR}/tests/fixtures/orb.gltf"
)

target_link_libraries(qqstudio PUBLIC
    Qt6::Gui Qt6::Quick Qt6::Quick3D Qt6::QuickControls2
    qqruntime qqruntimeplugin
)

qt_add_executable(qq-studio WIN32 main.cpp)
target_link_libraries(qq-studio PRIVATE qqstudio qqstudioplugin qqagent)
