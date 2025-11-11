cmake_minimum_required(VERSION 3.16)
project(NoteQuarry VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find Qt6 with MinGW support
find_package(Qt6 REQUIRED COMPONENTS Core Widgets Gui)

# Set compiler to MinGW (Windows users need to install MinGW)
if(WIN32)
    set(CMAKE_C_COMPILER "x86_64-w64-mingw32-gcc")
    set(CMAKE_CXX_COMPILER "x86_64-w64-mingw32-g++")
endif()

# Qt UI library (with C bridge for Rust)
add_library(notequarry_ui SHARED
    src/ui/mainwindow.cpp
    src/ui/mainwindow.h
    src/ui/qt_bridge.cpp
    src/ui/qt_bridge.h
)

target_link_libraries(notequarry_ui PUBLIC
    Qt6::Core
    Qt6::Widgets
    Qt6::Gui
)

target_include_directories(notequarry_ui PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src/ui
)

# Standalone Qt executable (for testing UI without Rust)
add_executable(notequarry_qt_test
    src/ui/main_test.cpp
)

target_link_libraries(notequarry_qt_test PRIVATE
    notequarry_ui
    Qt6::Widgets
)

# Set output directories
set_target_properties(notequarry_ui PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

set_target_properties(notequarry_qt_test PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
)

# Windows-specific: Copy Qt DLLs to output directory
if(WIN32)
    # Copy Qt libraries for the Qt executable
    add_custom_command(TARGET notequarry_qt_test POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:Qt6::Core>
            $<TARGET_FILE:Qt6::Gui>
            $<TARGET_FILE:Qt6::Widgets>
            $<TARGET_FILE_DIR:notequarry_qt_test>
    )

    # Also copy DLL for Rust executable
    add_custom_command(TARGET notequarry_ui POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:Qt6::Core>
            $<TARGET_FILE:Qt6::Gui>
            $<TARGET_FILE:Qt6::Widgets>
            ${CMAKE_BINARY_DIR}/bin
    )
endif()

# Installation rules
install(TARGETS notequarry_ui
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

# Add rule to build the Rust library
add_custom_target(build_rust_DEPENDS ON
    COMMAND ${CMAKE_COMMAND} -B build -S .
    COMMENT "Building Rust components"
)
