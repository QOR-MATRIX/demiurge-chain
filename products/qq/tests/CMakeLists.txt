# QQ's tests: Qt Test, rendering for real on the machine's GPU and judging what was drawn by its pixels.

qt_add_executable(tst_world tst_world.cpp)

target_compile_definitions(tst_world PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_world PRIVATE
    Qt6::Test Qt6::Gui Qt6::Quick Qt6::Quick3D
    qqruntime qqruntimeplugin
)

add_test(NAME tst_world COMMAND tst_world)

qt_add_executable(tst_scene tst_scene.cpp)

target_compile_definitions(tst_scene PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_scene PRIVATE
    Qt6::Test Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Quick3D
    qqruntime qqruntimeplugin
)

# A model inside the executable, as the Studio bundles its sample: adopting one into a project copies it out.
qt_add_resources(tst_scene "fixtures" PREFIX /fixtures BASE fixtures FILES fixtures/orb.gltf)

add_test(NAME tst_scene COMMAND tst_scene)

qt_add_executable(tst_studio tst_studio.cpp)

target_compile_definitions(tst_studio PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_studio PRIVATE
    Qt6::Test Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Quick3D Qt6::QuickControls2
    qqstudio qqstudioplugin qqruntime qqruntimeplugin
)

add_test(NAME tst_studio COMMAND tst_studio)

qt_add_executable(tst_play tst_play.cpp)

target_compile_definitions(tst_play PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_play PRIVATE
    Qt6::Test Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Quick3D
    qqruntime qqruntimeplugin
)

add_test(NAME tst_play COMMAND tst_play)

qt_add_executable(tst_player tst_player.cpp)

# The real player, started as the launcher starts it.
target_compile_definitions(tst_player PRIVATE
    QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures"
    QQ_PLAYER="$<TARGET_FILE:qq-player>")

target_link_libraries(tst_player PRIVATE Qt6::Test Qt6::Gui)
add_dependencies(tst_player qq-player)

add_test(NAME tst_player COMMAND tst_player)

qt_add_executable(tst_agent tst_agent.cpp)

# The real programs, started as an MCP client and the launcher start them.
target_compile_definitions(tst_agent PRIVATE
    QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures"
    QQ_STUDIO="$<TARGET_FILE:qq-studio>"
    QQ_MCP="$<TARGET_FILE:qq-mcp>")

target_link_libraries(tst_agent PRIVATE
    Qt6::Test Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Quick3D Qt6::QuickControls2 Qt6::Network
    qqagent qqstudio qqstudioplugin qqruntime qqruntimeplugin
)
add_dependencies(tst_agent qq-studio qq-mcp)

add_test(NAME tst_agent COMMAND tst_agent)

qt_add_executable(tst_lockdown tst_lockdown.cpp)

target_compile_definitions(tst_lockdown PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_lockdown PRIVATE
    Qt6::Test Qt6::Gui Qt6::Qml Qt6::Quick Qt6::Quick3D Qt6::Network
    qqruntime qqruntimeplugin
)

add_test(NAME tst_lockdown COMMAND tst_lockdown)
