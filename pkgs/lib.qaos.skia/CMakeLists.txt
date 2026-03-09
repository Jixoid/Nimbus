set(TARGET_NAME qaos_skia_inc)

add_library(${TARGET_NAME} INTERFACE)

target_include_directories(${TARGET_NAME} INTERFACE inc)



set(SKIA_LIBS "${CMAKE_SOURCE_DIR}/pkgs/lib.qaos.skia")

add_library(qaos_skia_lib SHARED IMPORTED GLOBAL)
set_target_properties(qaos_skia_lib PROPERTIES IMPORTED_LOCATION "${SKIA_LIBS}/lib/libskia.a")

add_library(qaos_skia_svg_lib SHARED IMPORTED GLOBAL)
set_target_properties(qaos_skia_svg_lib PROPERTIES IMPORTED_LOCATION "${SKIA_LIBS}/lib/libsvg.a")

add_library(qaos_skia_shaper_lib SHARED IMPORTED GLOBAL)
set_target_properties(qaos_skia_shaper_lib PROPERTIES IMPORTED_LOCATION "${SKIA_LIBS}/lib/libskshaper.a")

add_library(qaos_skia_unicode_core_lib SHARED IMPORTED GLOBAL)
set_target_properties(qaos_skia_unicode_core_lib PROPERTIES IMPORTED_LOCATION "${SKIA_LIBS}/lib/libskunicode_core.a")

add_library(qaos_skia_unicode_icu_lib SHARED IMPORTED GLOBAL)
set_target_properties(qaos_skia_unicode_icu_lib PROPERTIES IMPORTED_LOCATION "${SKIA_LIBS}/lib/libskunicode_icu.a")

