# The QOR Launcher's Qt shell (ADR-104; DIRECTION L13): the Android launcher, and the Qt shell ADR-103's stage 2 grows
# into. Qt 6, C++20 and Qt Quick, over the Rust core (../core), which it links through the core's C bridge.
#
#   src/    Core, the core's bridge as QML sees it
#   qml/    the module "QOR": the window and its screens; Tokens.qml, which docs/design/tokens.mjs generates from
#           docs/design/tokens.json, and Theme, which binds to it (ADR-108)
#   fonts/  Cinzel, Barlow, Barlow Semi Condensed and IBM Plex Mono, under the SIL Open Font License, copied by tokens.mjs
#   tests/  Qt Test suites (on the desktop build)
#
# build.ps1 does all of it: `-Android` for the phone's APK (arm64-v8a), nothing for the desktop build and its tests.

cmake_minimum_required(VERSION 3.22)

project(QorLauncherShell VERSION 0.1.0 DESCRIPTION "QOR Launcher" LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.12 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 Test)

qt_standard_project_setup(REQUIRES 6.12)

set(QT_QML_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml")

if(MSVC)
    add_compile_options(/W4 /permissive- /utf-8 /external:anglebrackets /external:W0 /wd4702)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# The core, built by cargo for whatever this build targets: the host for the desktop, aarch64-linux-android with the
# NDK's tools for the phone. Corrosion picks the Rust target from CMake's own (ADR-104 decision 2).
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.6.1
)
FetchContent_MakeAvailable(Corrosion)
corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../core/Cargo.toml CRATES qor-launcher-core CRATE_TYPES staticlib LOCKED)

# Everything but main(), so the tests use exactly what the application does.
qt_add_library(shell_core STATIC
    src/core.h src/core.cpp
    src/fonts.h
)
target_include_directories(shell_core PUBLIC src ${CMAKE_CURRENT_SOURCE_DIR}/../core/include)
target_link_libraries(shell_core PUBLIC Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick qor_launcher_core)

# Before the module, or Theme is registered as an ordinary type.
set_source_files_properties(qml/Theme.qml qml/Tokens.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_qml_module(shell_core
    URI QOR
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/Tokens.qml
        qml/Theme.qml
    RESOURCES
        fonts/Cinzel.ttf
        fonts/Barlow-Regular.ttf
        fonts/Barlow-Medium.ttf
        fonts/BarlowSemiCondensed-Regular.ttf
        fonts/BarlowSemiCondensed-Medium.ttf
        fonts/BarlowSemiCondensed-SemiBold.ttf
        fonts/IBMPlexMono-Regular.ttf
        fonts/IBMPlexMono-Medium.ttf
)

qt_add_executable(qor-launcher src/main.cpp)
target_link_libraries(qor-launcher PRIVATE shell_core shell_coreplugin Qt6::QuickControls2)

if(ANDROID)
    set_target_properties(qor-launcher PROPERTIES
        # The app's permanent ID, the owner's choice (ADR-104 decision 1).
        QT_ANDROID_PACKAGE_NAME "app.qorsync.dev"
        QT_ANDROID_APP_NAME "QOR Launcher"
        QT_ANDROID_VERSION_NAME "${PROJECT_VERSION}"
        QT_ANDROID_VERSION_CODE 1
        QT_ANDROID_MIN_SDK_VERSION 28
        QT_ANDROID_TARGET_SDK_VERSION 36
    )
else()
    enable_testing()
    qt_add_executable(tst_core tests/tst_core.cpp)
    target_link_libraries(tst_core PRIVATE shell_core shell_coreplugin Qt6::Test)
    # The look's test reads the source itself, not the file generated from it.
    target_compile_definitions(tst_core PRIVATE SHELL_TOKENS="${CMAKE_CURRENT_SOURCE_DIR}/../../../docs/design/tokens.json")
    add_test(NAME core COMMAND tst_core)
endif()
