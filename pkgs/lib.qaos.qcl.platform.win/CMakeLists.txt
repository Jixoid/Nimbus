if(NOT WIN32)
  message(STATUS "Self Detach: QCL.PLATFORM.WIN Support Library")
  return()
endif()


set(TARGET_NAME qaos_qcl_win)


file(GLOB_RECURSE SOURCES "src/*.cc")

add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})


target_include_directories(${TARGET_NAME}_shared PUBLIC Inc)
target_include_directories(${TARGET_NAME}_static PUBLIC Inc)



target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_qcl_shared
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)


target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_qcl_static
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)
