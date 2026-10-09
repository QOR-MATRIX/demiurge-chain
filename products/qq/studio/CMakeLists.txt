# QQ Studio, the native editor (ADR-083 decision 2). Its own process, started by the launcher; it never holds a key.

find_package(Qt6 REQUIRED COMPONENTS QuickControls2)

qt_add_executable(qq-studio WIN32 main.cpp)

qt_add_qml_module(qq-studio
    URI QQ.Studio
    VERSION 1.0
    QML_FILES Main.qml
)

# The model a new Studio opens with: the same fixture the tests render, so what is checked is what is shown.
qt_add_resources(qq-studio "samples"
    PREFIX /samples
    BASE "${PROJECT_SOURCE_DIR}/tests/fixtures"
    FILES "${PROJECT_SOURCE_DIR}/tests/fixtures/orb.gltf"
)

target_link_libraries(qq-studio PRIVATE
    Qt6::Gui Qt6::Quick Qt6::Quick3D Qt6::QuickControls2
    qqruntime qqruntimeplugin
)
