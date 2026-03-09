if(NOT LINUX)
  message(STATUS "Self Detach: QCL.PLATFORM.WAYLAND Support Library")
  return()
endif()


set(TARGET_NAME qaos_qcl_wayland)


file(GLOB_RECURSE SOURCES "src/*.cc" "src/*.c")

add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})

find_package(PkgConfig REQUIRED)
find_package(ZLIB REQUIRED)
find_package(EXPAT REQUIRED)
find_package(Freetype REQUIRED)
find_package(Fontconfig REQUIRED)
find_package(JPEG REQUIRED)

pkg_check_modules(WAYLAND REQUIRED 
  wayland-client 
  wayland-egl 
  wayland-cursor 
  wayland-protocols
)
pkg_check_modules(WAYLAND_CURSOR REQUIRED wayland-cursor)
pkg_check_modules(XKB REQUIRED xkbcommon)
pkg_check_modules(EGL REQUIRED egl)


target_include_directories(${TARGET_NAME}_shared PUBLIC Inc)
target_include_directories(${TARGET_NAME}_static PUBLIC Inc)





set(SKIA_LIBS "${CMAKE_SOURCE_DIR}/pkgs/lib.qaos.skia")

set_target_properties(${TARGET_NAME}_shared PROPERTIES
  BUILD_RPATH "${SKIA_LIBS}/lib"
  INSTALL_RPATH "${SKIA_LIBS}/lib"
)

set_target_properties(${TARGET_NAME}_static PROPERTIES
  BUILD_RPATH "${SKIA_LIBS}/lib"
  INSTALL_RPATH "${SKIA_LIBS}/lib"
)



target_include_directories(${TARGET_NAME}_shared PUBLIC inc)

target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_qcl_shared

  qaos_skia_inc
  qaos_skia_svg_lib
  qaos_skia_shaper_lib
  qaos_skia_unicode_core_lib
  qaos_skia_unicode_icu_lib
  qaos_skia_lib

  EGL
  GLESv2

  harfbuzz
  icuuc
  png
  JPEG::JPEG
  ZLIB::ZLIB
  EXPAT::EXPAT
  Fontconfig::Fontconfig
  webp webpdemux webpmux
  ${FREETYPE_LIBRARIES}

  ${WAYLAND_LIBRARIES}
  ${WAYLAND_CURSOR_LIBRARIES}
  ${XKB_LIBRARIES}
  ${EGL_LIBRARIES}
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)



target_include_directories(${TARGET_NAME}_static PUBLIC inc)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_qcl_static

  qaos_skia_inc
  qaos_skia_svg_lib
  qaos_skia_shaper_lib
  qaos_skia_unicode_core_lib
  qaos_skia_unicode_icu_lib
  qaos_skia_lib

  EGL
  GLESv2

  harfbuzz
  icuuc
  png
  JPEG::JPEG
  ZLIB::ZLIB
  EXPAT::EXPAT
  Fontconfig::Fontconfig
  webp webpdemux webpmux
  ${FREETYPE_LIBRARIES}

  ${WAYLAND_LIBRARIES}
  ${WAYLAND_CURSOR_LIBRARIES}
  ${XKB_LIBRARIES}
  ${EGL_LIBRARIES}
)
set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

