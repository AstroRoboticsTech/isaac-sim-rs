function(packman_pull_usd OUT_VAR)
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

    if(NOT EXISTS "${USD_LINK}/include/pxr/base/tf/token.h")
        message(STATUS "packman: fetching USD ${USD_PKG_VERSION} (~3.8 GB extracted, cached after first run)")
        execute_process(
            COMMAND "${PACKMAN}" pull "${USD_DEPS_XML}" --platform linux-x86_64
            RESULT_VARIABLE result
        )
        if(NOT result EQUAL 0)
            message(FATAL_ERROR "packman pull usd-release failed (exit ${result})")
        endif()
    endif()

    set(${OUT_VAR} "${USD_LINK}" PARENT_SCOPE)
endfunction()
