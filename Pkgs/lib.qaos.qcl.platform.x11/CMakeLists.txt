if(NOT LINUX)
  message(STATUS "Self Detach: QCL.PLATFORM.X11 Support Library")
  return()
endif()


set(TARGET_NAME qcl_x11)


file(GLOB_RECURSE SOURCES "Dev/*.cc")

add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})

find_package(OpenGL REQUIRED)
find_package(GLEW REQUIRED)
find_package(PkgConfig REQUIRED)

pkg_check_modules(CAIRO REQUIRED IMPORTED_TARGET cairo)


target_include_directories(${TARGET_NAME}_shared PUBLIC Inc)
target_include_directories(${TARGET_NAME}_static PUBLIC Inc)



include(FetchContent)

FetchContent_Declare(
  Corrosion
  GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
)
FetchContent_MakeAvailable(Corrosion)

FetchContent_Declare(
  resvg
  GIT_REPOSITORY https://github.com/linebender/resvg.git
  GIT_TAG main
)
FetchContent_MakeAvailable(resvg)

corrosion_import_crate(MANIFEST_PATH ${resvg_SOURCE_DIR}/crates/c-api/Cargo.toml)



target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qcl_dev

  resvg

  OpenGL::GL
  GLEW::GLEW
  PkgConfig::CAIRO
)

target_include_directories(${TARGET_NAME}_shared PUBLIC
  ${resvg_SOURCE_DIR}/crates/c-api
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)


target_link_libraries(${TARGET_NAME}_static PUBLIC
  qcl_dev

  resvg
  
  OpenGL::GL
  GLEW::GLEW
  PkgConfig::CAIRO
)

target_include_directories(${TARGET_NAME}_static PUBLIC
  ${resvg_SOURCE_DIR}/crates/c-api
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)
