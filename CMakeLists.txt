cmake_minimum_required(VERSION 3.16)
project(NoteQuarry VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find Qt6
find_package(Qt6 REQUIRED COMPONENTS Core Widgets Gui)

# Qt UI library
add_library(notequarry_ui SHARED
    src/ui/mainwindow.cpp
    src/ui/mainwindow.h
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

# Windows-specific: Copy Qt DLLs to output directory
if(WIN32)
    add_custom_command(TARGET notequarry_qt_test POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:Qt6::Core>
            $<TARGET_FILE:Qt6::Gui>
            $<TARGET_FILE:Qt6::Widgets>
            $<TARGET_FILE_DIR:notequarry_qt_test>
    )
endif()

# Installation rules
install(TARGETS notequarry_ui
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)