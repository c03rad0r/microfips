# ESP-IDF component that builds the microfips Rust firmware image.
#
# esp-hal firmware is a complete no_std binary, so this component does not
# register C sources. Instead it defines a CMake function the project calls to
# build (and optionally flash) the Rust firmware for the active target:
#
#   idf_component_register(SRCS "main.c" INCLUDE_DIRS ".")
#   microfips_esp32_firmware(TRANSPORT wifi)
#
# Requirements: the `esp` Rust toolchain (espup) and the Xtensa GCC on PATH.

# Resolve the cargo crate this component wraps (sibling of esp-idf/).
get_filename_component(_MICROFIPS_CRATE_DIR
    "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
set(MICROFIPS_MANIFEST "${_MICROFIPS_CRATE_DIR}/Cargo.toml")

# Map the ESP-IDF target -> (rust triple, cargo chip feature).
if(IDF_TARGET STREQUAL "esp32")
    set(_MICROFIPS_TRIPLE "xtensa-esp32-none-elf")
    set(_MICROFIPS_CHIP_FEATURE "esp32")
    set(_MICROFIPS_DEFAULT_BIN "microfips-esp32")
elseif(IDF_TARGET STREQUAL "esp32s3")
    set(_MICROFIPS_TRIPLE "xtensa-esp32s3-none-elf")
    set(_MICROFIPS_CHIP_FEATURE "esp32s3")
    set(_MICROFIPS_DEFAULT_BIN "microfips-esp32s3")
else()
    message(WARNING "microfips-esp32: target '${IDF_TARGET}' not supported (esp32/esp32s3 only)")
    return()
endif()

function(microfips_esp32_firmware)
    set(options)
    set(oneValueArgs TRANSPORT PROFILE)
    set(multiValueArgs)
    cmake_parse_arguments(MF "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})

    if(NOT MF_TRANSPORT)
        set(MF_TRANSPORT "uart")
    endif()
    if(NOT MF_PROFILE)
        set(MF_PROFILE "release")
    endif()

    set(_features "${_MICROFIPS_CHIP_FEATURE}")
    if(NOT MF_TRANSPORT STREQUAL "uart")
        list(APPEND _features "${MF_TRANSPORT}")
        set(_bin "${_MICROFIPS_DEFAULT_BIN}-${MF_TRANSPORT}")
    else()
        set(_bin "${_MICROFIPS_DEFAULT_BIN}")
    endif()

    set(_elf "${_MICROFIPS_CRATE_DIR}/target/${_MICROFIPS_TRIPLE}/${MF_PROFILE}/${_bin}")

    find_program(MICROFIPS_CARGO cargo REQUIRED)

    add_custom_command(
        OUTPUT "${_elf}"
        COMMAND "${MICROFIPS_CARGO}"
                $<$<BOOL:$ENV{MICROFIPS_RUST_TOOLCHAIN}>:+esp>
                build --profile ${MF_PROFILE}
                --target ${_MICROFIPS_TRIPLE}
                --manifest-path "${MICROFIPS_MANIFEST}"
                --features ${_features}
                --bin ${_bin}
        WORKING_DIRECTORY "${_MICROFIPS_CRATE_DIR}"
        DEPENDS "${MICROFIPS_MANIFEST}"
        COMMENT "[microfips-esp32] cargo build (chip=${_MICROFIPS_CHIP_FEATURE}, transport=${MF_TRANSPORT})"
        VERBATIM
    )

    add_custom_target(microfips_esp32_firmware ALL DEPENDS "${_elf}")
    set_property(DIRECTORY APPEND PROPERTY ADDITIONAL_MAKE_CLEAN_FILES "${_elf}")

    message(STATUS "microfips-esp32: firmware ELF -> ${_elf}")
endfunction()
