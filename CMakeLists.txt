cmake_minimum_required(VERSION 3.2.0)
project(vcontrold C)

IF(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
# CMAKE_INSTALL_PREFIX defaults to /usr/local, but it should go to /usr
set(CMAKE_INSTALL_PREFIX /usr)
ENDIF(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)

option(MANPAGES "Build man pages via rst2man" ON)
option(VCLIENT "Build the vclient helper program (for communication with vcontrold)" ON)
option(VSIM "Build the vsim helper program (for development and testing purposes)" OFF)

find_package(LibXml2 REQUIRED)

if(MANPAGES)
    find_program(RST2MAN NAMES rst2man rst2man.py)
    if(RST2MAN)
        message("-- Found rst2man: ${RST2MAN}")
    else(RST2MAN)
        message(FATAL_ERROR
            "Could not find a rst2man executable. Either set the \"MANPAGES\" option to \"OFF\" \
             (via cmake -DMANPAGES=OFF) or install Python's Docutils \
             (cf. http://docutils.sourceforge.net/)."
        )
    endif(RST2MAN)
endif(MANPAGES)

add_custom_target(
    UpdateVersion ALL
    COMMAND ${CMAKE_COMMAND}
        -DBASE_DIR=${CMAKE_CURRENT_SOURCE_DIR}
        -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
        -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/modules/UpdateVersion.cmake
    COMMENT "Updating version header."
    BYPRODUCTS ${CMAKE_CURRENT_SOURCE_DIR}/src/version.h
)

include_directories(
    ${LIBXML2_INCLUDE_DIR}
)

set(vcontrold_SRCS
    ${CMAKE_CURRENT_SOURCE_DIR}/src/io.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/xmlconfig.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/semaphore.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/framer.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/unit.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/vcontrold.c
)

set(vclient_SRCS
    ${CMAKE_CURRENT_SOURCE_DIR}/src/common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/io.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/client.c
    ${CMAKE_CURRENT_SOURCE_DIR}/src/vclient.c
)

set(vsim_SRCS
    ${CMAKE_CURRENT_SOURCE_DIR}/src/vsim.c
)

add_executable(vcontrold ${vcontrold_SRCS})
target_link_libraries(vcontrold ${LIBXML2_LIBRARIES})
add_dependencies(vcontrold UpdateVersion)

foreach(target IN ITEMS debug release)
    set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/${target}")
    set(LIBVCONTROL "${CARGO_TARGET_DIR}/libvcontrol.a")
    set(LIBVCONTROL_INCLUDE_DIR "${CARGO_TARGET_DIR}")

    file(MAKE_DIRECTORY ${LIBVCONTROL_INCLUDE_DIR})

    if(${target} STREQUAL release)
      set(CARGO_FLAGS "--release")
    endif()

    add_custom_command(OUTPUT ${LIBVCONTROL}
        COMMAND cargo build ${CARGO_FLAGS} --verbose
    )

    add_library("libvcontrol_${target}" STATIC IMPORTED GLOBAL)
    add_custom_target("libvcontrol_target_${target}" DEPENDS "${LIBVCONTROL}")
    add_dependencies("libvcontrol_${target}" "libvcontrol_target_${target}")
    set_target_properties("libvcontrol_${target}"
        PROPERTIES
        IMPORTED_LOCATION "${LIBVCONTROL}"
        INTERFACE_INCLUDE_DIRECTORIES "${LIBVCONTROL_INCLUDE_DIR}"
    )
endforeach()

set(libvcontrol
    debug libvcontrol_debug
    optimized libvcontrol_release
)

target_link_libraries(vcontrold ${libvcontrol})

if(VCLIENT)
    add_executable(vclient ${vclient_SRCS})
    add_dependencies(vclient UpdateVersion)
    target_link_libraries(vclient ${libvcontrol})
endif(VCLIENT)

if(VSIM)
    add_executable(vsim ${vsim_SRCS})
    add_dependencies(vsim UpdateVersion)
    target_link_libraries(vsim ${libvcontrol})
endif(VSIM)

if(MANPAGES)
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/doc/man)
endif(MANPAGES)

install(TARGETS vcontrold DESTINATION ${CMAKE_INSTALL_PREFIX}/sbin)
if(VCLIENT)
    install(TARGETS vclient DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)
endif(VCLIENT)
if(VSIM)
    install(TARGETS vsim DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)
endif(VSIM)
