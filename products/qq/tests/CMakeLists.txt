# QQ's tests: Qt Test, rendering for real on the machine's GPU and judging what was drawn by its pixels.

qt_add_executable(tst_world tst_world.cpp)

target_compile_definitions(tst_world PRIVATE QQ_FIXTURES="${CMAKE_CURRENT_SOURCE_DIR}/fixtures")

target_link_libraries(tst_world PRIVATE
    Qt6::Test Qt6::Gui Qt6::Quick Qt6::Quick3D
    qqruntime qqruntimeplugin
)

add_test(NAME tst_world COMMAND tst_world)
