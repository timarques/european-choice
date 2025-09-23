cmake_minimum_required(VERSION 3.15)
project(european-choice)

option(GVSBUILD "Download GVSBUILD" OFF)

include(GNUInstallDirs)

find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(RUSTC_EXECUTABLE rustc REQUIRED)
find_program(GLIB_COMPILE_SCHEMAS glib-compile-schemas)
find_package(PkgConfig)

set(CMAKE_BUILD_TYPE Release)
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(APP_BUILD_BIN_ARGS "")
    set(APP_BUILD_TYPE "debug")
elseif(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(APP_BUILD_BIN_ARGS "--release")
    set(APP_BUILD_TYPE "release")
elseif(CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
    set(APP_BUILD_BIN_ARGS "--release")
    set(APP_BUILD_TYPE "release")
elseif(CMAKE_BUILD_TYPE STREQUAL "MinSizeRel")
    set(APP_BUILD_BIN_ARGS "--release")
    set(APP_BUILD_TYPE "release")
else()
    set(APP_BUILD_BIN_ARGS "--release")
    set(APP_BUILD_TYPE "release")
    message(WARNING "Unknown build type '${CMAKE_BUILD_TYPE}', defaulting to release mode")
endif()

set(APP_SOURCE_DIR "${CMAKE_SOURCE_DIR}")
set(APP_MANIFEST_FILE "${APP_SOURCE_DIR}/Cargo.toml")

execute_process(
    COMMAND ${CMAKE_COMMAND} -E echo "Extracting project metadata from Cargo.toml"
    COMMAND ${CARGO_EXECUTABLE}
        metadata
        --manifest-path=${APP_MANIFEST_FILE}
        --format-version=1
        --no-deps
    OUTPUT_VARIABLE CARGO_METADATA
    OUTPUT_STRIP_TRAILING_WHITESPACE
    RESULT_VARIABLE METADATA_RESULT
)

if(NOT METADATA_RESULT EQUAL 0)
    message(FATAL_ERROR "Failed to extract metadata from Cargo.toml")
endif()

string(JSON APP_VERSION GET ${CARGO_METADATA} "packages" 0 "version")
string(JSON APP_NAME GET ${CARGO_METADATA} "packages" 0 "name")
string(JSON APP_DESCRIPTION GET ${CARGO_METADATA} "packages" 0 "description")
string(JSON APP_REPOSITORY GET ${CARGO_METADATA} "packages" 0 "repository")
string(JSON APP_AUTHOR GET ${CARGO_METADATA} "packages" 0 "metadata" "author")
string(JSON APP_TITLE GET ${CARGO_METADATA} "packages" 0 "metadata" "title")
string(JSON APP_ID GET ${CARGO_METADATA} "packages" 0 "metadata" "id")
string(JSON APP_PREFIX GET ${CARGO_METADATA} "packages" 0 "metadata" "prefix")

set(APP_OUTPUT_DIR "${CMAKE_BINARY_DIR}")
set(APP_DATA_DIR "${APP_SOURCE_DIR}/data")
set(APP_TARGET_DIR "${APP_OUTPUT_DIR}/target")
set(APP_README_FILE "${APP_SOURCE_DIR}/")
set(APP_README_FILE "${APP_SOURCE_DIR}/README.md")
set(APP_LICENSE_FILE "${APP_SOURCE_DIR}/LICENSE.txt")

set(APP_EXECUTABLE_FILE_NAME "${APP_NAME}")
set(APP_EXECUTABLE_FILE "${APP_TARGET_DIR}/${APP_BUILD_TYPE}/${APP_EXECUTABLE_FILE_NAME}")

set(APP_ICON_FILE_NAME "icon.svg")
set(APP_ICON_FILE "${APP_DATA_DIR}/${APP_ICON_FILE_NAME}")

set(APP_DESKTOP_FILE_NAME "${APP_ID}.desktop")
set(APP_DESKTOP_FILE "${APP_OUTPUT_DIR}/${APP_DESKTOP_FILE_NAME}")
set(APP_DESKTOP_TEAMPLATE_FILE "${APP_DATA_DIR}/app.desktop.in")

set(APP_SCHEMAS_TEMPLATE_FILE "${APP_DATA_DIR}/schemas.xml.in")
set(APP_SCHEMAS_FILE "${APP_OUTPUT_DIR}/${APP_ID}.gschema.xml")

if(WIN32)
    set(APP_EXECUTABLE_FILE_NAME "${APP_EXECUTABLE_FILE_NAME}.exe")
    set(APP_EXECUTABLE_FILE "${APP_EXECUTABLE_FILE}.exe")

    set(APP_ICON_FILE_NAME "icon.ico")
    set(APP_ICON_FILE "${APP_DATA_DIR}/${APP_ICON_FILE_NAME}")
endif()

set(APP_PATH "$ENV{PATH}")
set(APP_PKG_CONFIG_PATH "$ENV{PKG_CONFIG_PATH}")

set(APP_GVSBUILD_GTK_DIR ${APP_OUTPUT_DIR}/gtk4)
set(APP_GVSBUILD_SCHEMAS_DIR "${APP_GVSBUILD_GTK_DIR}/share/glib-2.0/schemas")
set(APP_GVSBUILD_SCHEMAS_FILE "${APP_GVSBUILD_GTK_DIR}/share/glib-2.0/schemas/gschemas.compiled")
set(APP_GVSBUILD_ZIP ${APP_OUTPUT_DIR}/gvsbuild.zip)
set(APP_GVSBUILD_URL "https://github.com/wingtk/gvsbuild/releases/download/2025.9.0/GTK4_Gvsbuild_2025.9.0_x64.zip")

set(CPACK_VERBATIM_VARIABLES ON)
set(CPACK_PACKAGE_NAME "${APP_NAME}")
set(CPACK_PACKAGE_VERSION "${APP_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "${APP_DESCRIPTION}")
set(CPACK_PACKAGE_HOMEPAGE_URL "${APP_REPOSITORY}")
set(CPACK_PACKAGE_CONTACT "${APP_AUTHOR}")
set(CPACK_PACKAGE_VENDOR "${APP_AUTHOR}")
set(CPACK_PACKAGE_DIRECTORY "${APP_OUTPUT_DIR}")
set(CPACK_PACKAGE_INSTALL_DIRECTORY "${APP_TITLE}")
set(CPACK_PACKAGE_ICON "${APP_ICON_FILE}")
set(CPACK_RESOURCE_FILE_LICENSE "${APP_LICENSE_FILE}")
set(CPACK_RESOURCE_FILE_README "${APP_README_FILE}")
set(CPACK_THREADS 0)

if (UNIX AND NOT APPLE)
    set(CPACK_GENERATOR "DEB;RPM;ZIP")
elseif (WIN32 AND EXISTS ${APP_GVSBUILD_GTK_DIR})
    set(CPACK_GENERATOR "IFW;ZIP")
else()
    set(CPACK_GENERATOR "ZIP")
endif()

set(CPACK_IFW_PACKAGE_TITLE "${APP_TITLE}")
set(CPACK_IFW_PACKAGE_DESCRIPTION "${APP_DESCRIPTION}")
set(CPACK_IFW_PACKAGE_ICON "${APP_ICON_FILE}")
set(CPACK_IFW_PACKAGE_PUBLISHER "${APP_AUTHOR}")
set(CPACK_IFW_PACKAGE_VERSION "${APP_VERSION}")
set(CPACK_IFW_PACKAGE_START_MENU_DIRECTORY "${APP_TITLE}")
set(CPACK_IFW_PACKAGE_WIZARD_STYLE "Modern")
set(CPACK_IFW_PACKAGE_RUN_PROGRAM "@TargetDir@/${CMAKE_INSTALL_BINDIR}/${APP_EXECUTABLE_FILE_NAME}")
set(CPACK_IFW_PACKAGE_MAINTENANCE_TOOL_NAME "${APP_NAME}MaintenanceTool")
set(CPACK_IFW_PACKAGE_MAINTENANCE_TOOL_INI_FILE "${APP_NAME}MaintenanceTool.ini")
set(CPACK_IFW_PRODUCT_URL "${APP_REPOSITORY}")
set(CPACK_IFW_TARGET_DIRECTORY "@ApplicationsDir@/${APP_TITLE}")

set(CPACK_DEBIAN_PACKAGE_SECTION "Utilities")
set(CPACK_DEBIAN_PACKAGE_DEPENDS "libgtk-4, libadwaita-1")
set(CPACK_RPM_PACKAGE_REQUIRES "gtk4 >= 4.10, libadwaita >= 1.6")

include(CPack)
include(CPackIFW)

if(WIN32 AND (GVSBUILD OR NOT PkgConfig_FOUND))
    if(NOT EXISTS ${APP_GVSBUILD_GTK_DIR})
        message(STATUS "GTK4 dependencies not found - downloading gvsbuild package")
        message(STATUS "Download URL: ${APP_GVSBUILD_URL}")

        file(
            DOWNLOAD ${APP_GVSBUILD_URL} ${APP_GVSBUILD_ZIP} SHOW_PROGRESS
            STATUS DOWNLOAD_STATUS
            LOG DOWNLOAD_LOG
        )

        list(GET DOWNLOAD_STATUS 0 STATUS_CODE)
        if(NOT STATUS_CODE EQUAL 0)
            message(FATAL_ERROR "Failed to download GTK4 dependencies from gvsbuild: ${DOWNLOAD_LOG}")
        endif()

        message(STATUS "Extracting GTK4 dependencies to ${APP_GVSBUILD_GTK_DIR}")
        file(MAKE_DIRECTORY ${APP_GVSBUILD_GTK_DIR})
        execute_process(
            COMMAND ${CMAKE_COMMAND} -E tar xzf ${APP_GVSBUILD_ZIP}
            WORKING_DIRECTORY ${APP_GVSBUILD_GTK_DIR}
            RESULT_VARIABLE EXTRACT_RESULT
        )

        if(NOT EXTRACT_RESULT EQUAL 0)
            message(FATAL_ERROR "Failed to extract GTK4 dependencies archive")
        endif()

        message(STATUS "Cleaning up downloaded archive")
        file(REMOVE ${APP_GVSBUILD_ZIP})
        message(STATUS "GTK4 dependencies successfully installed")
    else()
        message(STATUS "Using existing GTK4 dependencies at ${APP_GVSBUILD_GTK_DIR}")
    endif()

    set(APP_PATH "${APP_GVSBUILD_GTK_DIR}/bin;${APP_PATH}")

    file(TO_CMAKE_PATH "${APP_GVSBUILD_GTK_DIR}/lib/pkgconfig" APP_PKG_CONFIG_PATH)
    string(REGEX REPLACE "^([A-Z]):" "/\\1" APP_PKG_CONFIG_PATH "${APP_PKG_CONFIG_PATH}")
    string(REPLACE "\\" "/" APP_PKG_CONFIG_PATH "${APP_PKG_CONFIG_PATH}")

    set(GLIB_COMPILE_SCHEMAS "${APP_GVSBUILD_GTK_DIR}/bin/glib-compile-schemas.exe")
elseif(PkgConfig_FOUND AND GLIB_COMPILE_SCHEMAS)
    message(STATUS "Using system GTK4 dependencies via pkg-config")
    message(STATUS "Using system glib-compile-schemas: ${GLIB_COMPILE_SCHEMAS}")
    pkg_check_modules(GTK4 REQUIRED gtk4>=4.10)
    pkg_check_modules(LIBADWAITA REQUIRED libadwaita-1>=1.6)
    pkg_check_modules(GLIB REQUIRED glib-2.0)
    pkg_check_modules(GIO REQUIRED gio-2.0)
else()
    message(FATAL_ERROR "Could not find GTK4 dependencies: neither pkg-config nor gvsbuild available")
endif()

add_custom_target(BUILD_BIN ALL
    COMMAND ${CMAKE_COMMAND} -E env
        "OUTPUT_DIR=${APP_OUTPUT_DIR}"
        "SCHEMAS_INSTALLED=1"
        "GSETTINGS_SCHEMA_DIR=${CMAKE_INSTALL_DATAROOTDIR}/glib-2.0/schemas"
        "PATH=${APP_PATH}"
        "PKG_CONFIG_PATH=${APP_PKG_CONFIG_PATH}"
        ${CARGO_EXECUTABLE}
        build
        ${APP_BUILD_BIN_ARGS}
        --manifest-path=${APP_MANIFEST_FILE}
        --target-dir=${APP_TARGET_DIR}
        --color=always
    WORKING_DIRECTORY ${APP_SOURCE_DIR}
    USES_TERMINAL
    COMMENT "Building ${APP_EXECUTABLE_FILE_NAME}"
)

configure_file(${APP_SCHEMAS_TEMPLATE_FILE} ${APP_SCHEMAS_FILE} @ONLY)

if(WIN32 AND GVSBUILD)
    add_custom_target(BUILD_SCHEMAS_GVSBUILD ALL
        COMMAND ${CMAKE_COMMAND} -E copy "${APP_SCHEMAS_FILE}" "${APP_GVSBUILD_SCHEMAS_DIR}"
        COMMAND ${GLIB_COMPILE_SCHEMAS} --strict "${APP_GVSBUILD_SCHEMAS_DIR}"
        COMMENT "Compiling GLib schemas for Windows"
        VERBATIM
        DEPENDS BUILD_BIN
    )
endif()

if(UNIX AND NOT APPLE)
    configure_file("${APP_DESKTOP_TEAMPLATE_FILE}" ${APP_DESKTOP_FILE} @ONLY)
endif()

if(UNIX AND NOT APPLE)
    install(
        FILES ${APP_DESKTOP_FILE}
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/applications
        COMPONENT applications
    )

    install(
        FILES ${APP_ICON_FILE} 
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/scalable/apps
        RENAME ${APP_ID}.svg
        COMPONENT applications
    )
endif()

if(WIN32 AND GVSBUILD)
    install(
        DIRECTORY ${APP_GVSBUILD_GTK_DIR}/bin/
        DESTINATION ${CMAKE_INSTALL_BINDIR}
        COMPONENT runtime
        FILES_MATCHING PATTERN "*.dll"
    )

    install(
        DIRECTORY ${APP_GVSBUILD_GTK_DIR}/lib/gdk-pixbuf-2.0
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
        COMPONENT runtime
    )

    install(
        FILES ${APP_GVSBUILD_SCHEMAS_DIR}/gschemas.compiled
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/glib-2.0/schemas
        COMPONENT runtime
    )

    install(
        DIRECTORY ${APP_GVSBUILD_GTK_DIR}/share/icons/Adwaita
        DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/icons
        COMPONENT runtime
    )
endif()

install(
    FILES "${APP_EXECUTABLE_FILE}"
    DESTINATION ${CMAKE_INSTALL_BINDIR}
    COMPONENT applications
)

install(
    FILES "${APP_LICENSE_FILE}"
    DESTINATION ${CMAKE_INSTALL_DOCDIR}
    COMPONENT documentation
)

install(
    FILES "${APP_README_FILE}"
    DESTINATION ${CMAKE_INSTALL_DOCDIR}
    COMPONENT documentation
)

install(
    FILES ${APP_SCHEMAS_FILE}
    DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/glib-2.0/schemas
    COMPONENT applications
)

install(CODE "
    execute_process(
        COMMAND ${GLIB_COMPILE_SCHEMAS} --strict \"\${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_DATAROOTDIR}/glib-2.0/schemas\"
        RESULT_VARIABLE result
    )
    if(NOT result EQUAL 0)
        message(FATAL_ERROR \"glib-compile-schemas failed\")
    endif()
")