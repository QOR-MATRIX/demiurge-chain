# The QQ Runtime: the QML module "QQ" (ADR-083 decision 2). QQ Studio and the QQ Player both run on it, so what a
# creator sees while editing is what a player gets. Static, and linked into each executable with its plugin.

qt_add_library(qqruntime STATIC)

qt_add_qml_module(qqruntime
    URI QQ
    VERSION 1.0
    QML_FILES
        World.qml
        Scene.qml
        Sun.qml
        Lamp.qml
        Ground.qml
        Shape.qml
        Prop.qml
    SOURCES
        sceneio.h sceneio.cpp
)

target_include_directories(qqruntime PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qqruntime PUBLIC Qt6::Qml Qt6::Quick Qt6::Quick3D)
