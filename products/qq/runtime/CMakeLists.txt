# The QQ Runtime: the QML module "QQ" (ADR-083 decision 2). QQ Studio and the QQ Player both run on it, so what a
# creator sees while editing is what a player gets. Static, and linked into each executable with its plugin.

qt_add_library(qqruntime STATIC)

qt_add_qml_module(qqruntime
    URI QQ
    VERSION 1.0
    QML_FILES
        World.qml
        Sun.qml
        Ground.qml
        Prop.qml
)

target_link_libraries(qqruntime PRIVATE Qt6::Quick Qt6::Quick3D)
