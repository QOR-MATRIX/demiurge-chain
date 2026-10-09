# QQ, the QOR Engine (ADR-081, ADR-083): Qt 6, C++20 and QML.
#
#   runtime/  the QML module "QQ": worlds, light, props
#   studio/   QQ Studio, the native editor
#   tests/    Qt Test suites, rendering for real and reading back pixels
#
# Configure with the Qt the owner installed (commercial licence, ADR-083), MSVC 2022 and Ninja:
#
#   cmake -S products/qq -B <build> -G Ninja -DCMAKE_PREFIX_PATH=C:/Qt/6.12.0/msvc2022_64 -DCMAKE_BUILD_TYPE=Release
#   cmake --build <build>
#   ctest --test-dir <build> --output-on-failure
#
# products/qq/README.md has the whole procedure. Build outside the repository's drive if it is short of space.

cmake_minimum_required(VERSION 3.22)

project(QQ VERSION 0.1.0 DESCRIPTION "QQ, the QOR Engine" LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.12 REQUIRED COMPONENTS Core Gui Qml Quick Quick3D QuickControls2 Test)

qt_standard_project_setup(REQUIRES 6.12)

# Every QML module lands in one place, so a test can import them from the build tree.
set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml")

if(MSVC)
    # /W4 for QQ's own code; headers included with angle brackets (Qt's) are external and judged by Qt, not here.
    add_compile_options(/W4 /permissive- /utf-8 /external:anglebrackets /external:W0
        # C4702 (unreachable code) is raised by the optimiser inside Qt's templates, where /external cannot reach.
        /wd4702)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

add_subdirectory(runtime)
add_subdirectory(studio)

enable_testing()
add_subdirectory(tests)
