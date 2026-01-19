set(TARGET_NAME demo_app)


file(GLOB_RECURSE SOURCES "Dev/*.cc")

add_executable(${TARGET_NAME} ${SOURCES})


target_link_libraries(${TARGET_NAME} PUBLIC
  qcl_dev
  qcl_qstd_dev

  jconf_shared
  qcl_shared
  qcl_qstd_shared
  qcl_x11_shared
)

set_target_properties(${TARGET_NAME} PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

