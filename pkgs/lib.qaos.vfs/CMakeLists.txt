set(TARGET_NAME qaos_vfs)


file(GLOB_RECURSE SOURCES "src/*.cc")


if (NOT EMSCRIPTEN)
  add_library(${TARGET_NAME}_shared SHARED ${SOURCES})

  target_include_directories(${TARGET_NAME}_shared PUBLIC inc)

  target_link_libraries(${TARGET_NAME}_shared PUBLIC
    qaos_basis
  )

  set_target_properties(${TARGET_NAME}_shared PROPERTIES 
    OUTPUT_NAME "${TARGET_NAME}"
  )
endif()



add_library(${TARGET_NAME}_static STATIC ${SOURCES})

target_include_directories(${TARGET_NAME}_static PUBLIC inc)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_basis
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

