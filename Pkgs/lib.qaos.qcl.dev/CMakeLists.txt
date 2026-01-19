set(TARGET_NAME qcl_dev)


add_library(${TARGET_NAME} INTERFACE)

target_include_directories(${TARGET_NAME} INTERFACE Inc)

target_link_libraries(${TARGET_NAME} INTERFACE
  basis_dev
  jconf_dev
)

