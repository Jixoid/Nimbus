set(TARGET_NAME qaos_qcl_platform)


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




target_include_directories(${TARGET_NAME}_shared PUBLIC src)

target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_qcl_shared
  ext_skia

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



target_include_directories(${TARGET_NAME}_static PUBLIC src)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_qcl_static
  ext_skia

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
