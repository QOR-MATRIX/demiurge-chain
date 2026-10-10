# The QQ Player (DIRECTION P3.2): plays one QQ scene as a game, natively and, built with Qt for WebAssembly, in a
# browser. Its window is the QML module "QQ.Player"; the playground it plays by default is bundled inside it.

qt_add_executable(qq-player WIN32 main.cpp)

qt_add_qml_module(qq-player
    URI QQ.Player
    VERSION 1.0
    QML_FILES
        Main.qml
)

# The playground: a small game with everything P3.2 plays (bodies, a player, particles, sound and logic), kept as a
# project would keep it, so its relative paths mean what they mean on disk.
qt_add_resources(qq-player "playground"
    PREFIX /playground
    BASE "${CMAKE_CURRENT_SOURCE_DIR}/playground"
    FILES
        playground/scenes/playground.qml
        playground/logic/hover.qml
        playground/assets/orb.gltf
        playground/assets/chime.wav
)

target_link_libraries(qq-player PRIVATE Qt6::Gui Qt6::Quick Qt6::Quick3D qqruntime qqruntimeplugin)

if(EMSCRIPTEN)
    # The browser build: a page that fills its window, and the QQ-MEASURE line on the console.
    set_target_properties(qq-player PROPERTIES QT_WASM_INITIAL_MEMORY "256MB")
endif()
