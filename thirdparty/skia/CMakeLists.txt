set(TARGET_NAME ext_skia)

add_library(${TARGET_NAME} INTERFACE)


target_include_directories(${TARGET_NAME} INTERFACE "${CMAKE_CURRENT_SOURCE_DIR}/inc")


set(SKIA_LIBS "${CMAKE_CURRENT_SOURCE_DIR}/lib")

target_link_libraries(${TARGET_NAME} INTERFACE 
  "${SKIA_LIBS}/libskia.a"
  "${SKIA_LIBS}/libsvg.a"
  "${SKIA_LIBS}/libskshaper.a"
  "${SKIA_LIBS}/libskunicode_core.a"
  "${SKIA_LIBS}/libskunicode_icu.a"
)
