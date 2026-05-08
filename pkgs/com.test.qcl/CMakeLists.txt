set(TARGET_NAME com_test_qcl)



file(GLOB_RECURSE RES_FILES RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "res/*")
set(RES_OBJECTS "")


if (NOT EMSCRIPTEN)
  foreach(RES_FILE ${RES_FILES})
    set(OBJ_FILE "${CMAKE_CURRENT_BINARY_DIR}/${RES_FILE}.o")

    string(REPLACE "/" "::" SYM_NAME "${RES_FILE}")
    string(REPLACE "." "_"  SYM_NAME "${SYM_NAME}")
    
    add_custom_command(
      OUTPUT ${OBJ_FILE}
      COMMAND ld -r -b binary -o ${OBJ_FILE} ${RES_FILE}
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
      DEPENDS ${RES_FILE}
      COMMENT "Embeding: ${RES_FILE}"
    )
    
    list(APPEND RES_OBJECTS ${OBJ_FILE})
  endforeach()
endif()



file(GLOB_RECURSE SOURCES "src/*.cc")

add_executable(${TARGET_NAME} ${SOURCES} ${RES_OBJECTS})


target_include_directories(${TARGET_NAME} PUBLIC inc)

target_link_libraries(${TARGET_NAME} PRIVATE
  qaos_vfs_shared
  
  qaos_qcl_shared
  qaos_qcl_qstd_shared
  
  $<$<PLATFORM_ID:Linux>:qaos_qcl_wayland_shared>
)


target_link_options(${TARGET_NAME} PUBLIC -rdynamic)
