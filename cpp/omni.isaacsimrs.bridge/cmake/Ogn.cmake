function(add_ogn_node)
    set(one_value_args OGN_FILE EXTENSION MODULE)
    cmake_parse_arguments(ARG "" "${one_value_args}" "" ${ARGN})

    if(NOT ARG_OGN_FILE)
        message(FATAL_ERROR "add_ogn_node: OGN_FILE is required")
    endif()
    if(NOT ARG_EXTENSION)
        message(FATAL_ERROR "add_ogn_node: EXTENSION is required")
    endif()

    get_filename_component(NODE_NAME ${ARG_OGN_FILE} NAME_WE)
    set(GEN_DIR "${CMAKE_CURRENT_BINARY_DIR}/ogn")
    set(DATABASE_H "${GEN_DIR}/${NODE_NAME}Database.h")
    set(MODULE_NAME ${ARG_MODULE})
    if(NOT MODULE_NAME)
        set(MODULE_NAME ${ARG_EXTENSION})
    endif()

    set(SCRIPT "${CMAKE_CURRENT_SOURCE_DIR}/scripts/run_ogn_codegen.py")
    set(KIT_APP "${ISAAC_SIM}/apps/isaacsim.exp.base.kit")

    add_custom_command(
        OUTPUT "${DATABASE_H}"
        COMMAND ${CMAKE_COMMAND} -E env
                "OGN_FILE=${ARG_OGN_FILE}"
                "OGN_CLASS_NAME=${NODE_NAME}"
                "OGN_EXTENSION=${ARG_EXTENSION}"
                "OGN_MODULE=${MODULE_NAME}"
                "OGN_OUT_DIR=${GEN_DIR}"
                "${ISAAC_SIM}/kit/kit"
                "${KIT_APP}"
                --no-window --no-ros-env
                --enable omni.graph.tools
                --exec "${SCRIPT}"
        DEPENDS "${ARG_OGN_FILE}" "${SCRIPT}"
        COMMENT "OGN codegen: ${NODE_NAME}"
        JOB_POOL ogn_codegen
        VERBATIM
    )

    set(${NODE_NAME}_DATABASE_H "${DATABASE_H}" PARENT_SCOPE)
    set(OGN_GEN_DIR "${GEN_DIR}" PARENT_SCOPE)
endfunction()
