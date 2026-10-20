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
        Trigger.qml
        ScreenPart.qml
        ScreenText.qml
        ScreenBar.qml
        ScreenButton.qml
        ScreenImage.qml
        ScreenPanel.qml
        InputAction.qml
        Sprite.qml
        TileMap.qml
        Player2D.qml
        Music.qml
        Voices.qml
        Spotlight.qml
        Group.qml
        Prefab.qml
        Logic.qml
        Game.qml
    RESOURCES
        # A Shape's pictured surface (P3.14), beside Shape.qml, which names them relative to itself.
        shaders/surface.vert
        shaders/surface.frag
    SOURCES
        sceneio.h sceneio.cpp
        input.h input.cpp
        logicfile.h logicfile.cpp
        softdot.h softdot.cpp
        gpu.h gpu.cpp
        confinement.h confinement.cpp
        webvoice.h webvoice.cpp
        look.h look.cpp
        tilemap.h tilemap.cpp
        kinematics.h kinematics.cpp
        run.h run.cpp
        clips.h clips.cpp
        mixer.h mixer.cpp
)

# The interface's faces (Look), the homepage's and ARQADE's, under the SIL Open Font License (fonts/*-OFL.txt).
qt_add_resources(qqruntime "qqfonts"
    PREFIX "/qq/fonts"
    BASE fonts
    FILES
        fonts/Cinzel.ttf
        fonts/BarlowSemiCondensed-Regular.ttf
        fonts/BarlowSemiCondensed-SemiBold.ttf
)

target_include_directories(qqruntime PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
# QQ's own kinematics and random numbers (P3.11) must give the same results on every machine: no multiply-add fused
# where the source has two operations. MSVC does not fuse them under its default /fp:precise; GCC and Clang may.
if(NOT MSVC)
    target_compile_options(qqruntime PRIVATE -ffp-contract=off)
endif()
target_link_libraries(qqruntime PUBLIC Qt6::Qml Qt6::Quick Qt6::Quick3D PRIVATE Qt6::Network Qt6::Multimedia)
if(WIN32)
    # Gamepads, through XInput (xinput1_4.dll, part of Windows since 8); the choice of GPU, through DXGI; the system's
    # animation setting (Look), through user32.
    target_link_libraries(qqruntime PRIVATE xinput dxgi user32)
endif()
