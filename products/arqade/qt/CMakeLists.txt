# ARQADE in the launcher (ADR-102; DIRECTION P7.19): the QOR Launcher's arcade, in Qt 6, C++20, Qt Quick and Qt Quick 3D,
# docked inside the launcher under its product host as QOR Media is.
#
#   src/    the launcher link (arqade.next, arqade.signin, arqade.web), ARQADE's server, docking into the launcher's window
#   qml/    the module "ARQADE": the window, the portal, the games, the player's Sparks, the rankings
#   assets/ the sky every 3D view's metal reflects and the core's halo (tools/make_light.py writes both)
#   fonts/  Cinzel and Barlow Semi Condensed, the homepage's faces, under the SIL Open Font License
#   tests/  Qt Test suites
#
# Configure with the Qt the owner installed (commercial licence), MSVC 2022 and Ninja; build.ps1 does all of it:
#
#   pwsh products/arqade/qt/build.ps1            # build and test
#   pwsh products/arqade/qt/build.ps1 -Deploy    # and make the self-contained copy the launcher opens

cmake_minimum_required(VERSION 3.22)

project(Arqade VERSION 0.1.0 DESCRIPTION "ARQADE" LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.12 REQUIRED COMPONENTS Core Gui GuiPrivate Network Qml Quick QuickControls2 Quick3D Test)

qt_standard_project_setup(REQUIRES 6.12)

set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml")

if(MSVC)
    add_compile_options(/W4 /permissive- /utf-8 /external:anglebrackets /external:W0 /wd4702)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Everything but main(), so the tests use exactly what the application does.
qt_add_library(arqade_core STATIC
    src/arqadelink.h src/arqadelink.cpp
    src/arqadeserver.h src/arqadeserver.cpp
    src/dock.h src/dock.cpp
    src/fonts.h
)
target_include_directories(arqade_core PUBLIC src)
target_link_libraries(arqade_core PUBLIC Qt6::Core Qt6::Gui Qt6::Network Qt6::Qml Qt6::Quick Qt6::Quick3D)

# Before the module, or Theme is registered as an ordinary type.
set_source_files_properties(qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_qml_module(arqade_core
    URI ARQADE
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/Theme.qml
        qml/Home.qml
        qml/Portal.qml
        qml/Emblem.qml
        qml/GameCard.qml
        qml/PlayerPanel.qml
        qml/Rankings.qml
        qml/Wordmark.qml
        qml/Heading.qml
        qml/Glass.qml
        qml/ArqButton.qml
    RESOURCES
        assets/sky.hdr
        assets/halo.png
        fonts/Cinzel.ttf
        fonts/BarlowSemiCondensed-Regular.ttf
        fonts/BarlowSemiCondensed-Medium.ttf
        fonts/BarlowSemiCondensed-SemiBold.ttf
)

qt_add_executable(arqade WIN32 src/main.cpp)
target_link_libraries(arqade PRIVATE arqade_core arqade_coreplugin Qt6::GuiPrivate Qt6::QuickControls2)
if(WIN32)
    # The high-performance graphics card is found through DXGI (main.cpp).
    target_link_libraries(arqade PRIVATE dxgi)
endif()

enable_testing()
foreach(suite arqadelink arqadeserver interface)
    qt_add_executable(tst_${suite} tests/tst_${suite}.cpp tests/fakes.h)
    target_link_libraries(tst_${suite} PRIVATE arqade_core arqade_coreplugin Qt6::QuickControls2 Qt6::Test)
    add_test(NAME ${suite} COMMAND tst_${suite})
endforeach()
