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
        Player.qml
        Emitter.qml
        Sound.qml
        Behaviour.qml
        Logic.qml
        Game.qml
    SOURCES
        sceneio.h sceneio.cpp
        input.h input.cpp
        logicfile.h logicfile.cpp
        softdot.h softdot.cpp
        gpu.h gpu.cpp
        confinement.h confinement.cpp
        webvoice.h webvoice.cpp
)

target_include_directories(qqruntime PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qqruntime PUBLIC Qt6::Qml Qt6::Quick Qt6::Quick3D PRIVATE Qt6::Network Qt6::Multimedia)
if(WIN32)
    # Gamepads, through XInput (xinput1_4.dll, part of Windows since 8); the choice of GPU, through DXGI.
    target_link_libraries(qqruntime PRIVATE xinput dxgi)
endif()
