function(packman_pull_usd OUT_VAR)
    if(NOT EXT_PLATFORM STREQUAL "linux-x86_64")
        message(FATAL_ERROR
            "USD packman package is x86_64-only (got EXT_PLATFORM=${EXT_PLATFORM}). "
            "Aarch64 USD support requires a different package name + version.")
    endif()

    set(PACKMAN "${ISAAC_SIM}/kit/dev/tools/packman/packman")
    set(DEPS_DIR "${CMAKE_BINARY_DIR}/deps")
    set(USD_LINK "${DEPS_DIR}/usd-release")
    file(MAKE_DIRECTORY "${DEPS_DIR}")

    set(USD_DEPS_XML "${CMAKE_BINARY_DIR}/usd-deps.packman.xml")
    set(USD_PKG_NAME "usd.py311.manylinux_2_35_x86_64.stock.release")
    set(USD_PKG_VERSION "0.24.05.kit.7-gl.16400+05f48f24")
    file(WRITE "${USD_DEPS_XML}"
"<project toolsVersion=\"6.11\">
    <dependency name=\"usd-release\" linkPath=\"${USD_LINK}\">
        <package name=\"${USD_PKG_NAME}\" version=\"${USD_PKG_VERSION}\" />
    </dependency>
</project>
")

    # A header-existence check is fragile under partial extraction
    # (Ctrl-C mid-pull): a stray header survives, the symlink+marker
    # don't, but the next configure thinks the cache is good. Use a
    # post-extract marker that we write only after a successful pull.
    set(USD_COMPLETE "${USD_LINK}/.packman_complete")
    if(NOT EXISTS "${USD_COMPLETE}")
        message(STATUS "packman: fetching USD ${USD_PKG_VERSION} (~3.8 GB extracted, cached after first run)")
        execute_process(
            COMMAND "${PACKMAN}" pull "${USD_DEPS_XML}" --platform "${EXT_PLATFORM}"
            RESULT_VARIABLE result
        )
        if(NOT result EQUAL 0)
            message(FATAL_ERROR "packman pull usd-release failed (exit ${result})")
        endif()
        file(WRITE "${USD_COMPLETE}" "${USD_PKG_VERSION}\n")
    endif()

    set(${OUT_VAR} "${USD_LINK}" PARENT_SCOPE)
endfunction()
