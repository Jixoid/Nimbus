set(TARGET_NAME qcl_qstd)


file(GLOB_RECURSE SOURCES "Dev/*.cc")

add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})


target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qcl_qstd_dev
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)


target_link_libraries(${TARGET_NAME}_static PUBLIC
  qcl_qstd_dev
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

