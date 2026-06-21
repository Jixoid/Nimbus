cmake_minimum_required(VERSION 4.3)
project(QCL C CXX)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 26)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

add_compile_options(-gdwarf-4)




# Debug
add_compile_definitions($<$<CONFIG:Debug>:_DBG> $<$<CONFIG:RelWithDebInfo>:_DBG>)
add_compile_definitions(_QCL_WAYLAND_use_painttime)
#add_compile_definitions(_QCL_use_debug)
add_compile_definitions(_QCL_use_hwvec)

if (EMSCRIPTEN)
  add_compile_definitions("SK_TRIVIAL_ABI=[[clang::trivial_abi]]")
  add_link_options("-sASSERTIONS=1" "-sERROR_ON_UNDEFINED_SYMBOLS=1" "-sVERBOSE=1")
endif()


# Sanitizer
add_compile_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)
add_link_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)


# LTO
include(CheckIPOSupported)
check_ipo_supported(RESULT lto_supported OUTPUT lto_error)

if(lto_supported)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELWITHDEBINFO ON)
else()
  message(WARNING "LTO is not supported: ${lto_error}")
endif()


# Depency
find_package(Python3 REQUIRED) 

add_subdirectory(platform)


# Skia
add_library(ext_skia INTERFACE)

target_include_directories(ext_skia INTERFACE "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/skia")

set(SKIA_LIBS "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/skia/out/skbin")

target_link_libraries(ext_skia INTERFACE
  "${SKIA_LIBS}/libskia.a"
  "${SKIA_LIBS}/libsvg.a"
  "${SKIA_LIBS}/libskshaper.a"
  "${SKIA_LIBS}/libskunicode_core.a"
  "${SKIA_LIBS}/libskunicode_icu.a"
)



# QMC begin
file(GLOB_RECURSE HEADERS "${CMAKE_CURRENT_SOURCE_DIR}/qcl/*.hh" "${CMAKE_CURRENT_SOURCE_DIR}/qstd/*.hh")

set(QMC_SCRIPT "${CMAKE_CURRENT_SOURCE_DIR}/scripts/qmc.py")
set(QMC_SOURCES "")

foreach(HEADER ${HEADERS})
  file(RELATIVE_PATH REL_HEADER "${CMAKE_CURRENT_SOURCE_DIR}" "${HEADER}")
  
  string(REPLACE "/" "_" FLAT_HEADER "${REL_HEADER}")
  set(GENERATED_CC "${CMAKE_CURRENT_BINARY_DIR}/qmc_gen_${FLAT_HEADER}.cc")

  add_custom_command(
    OUTPUT ${GENERATED_CC}
    COMMAND ${Python3_EXECUTABLE} ${QMC_SCRIPT} ${HEADER} ${GENERATED_CC} ${REL_HEADER}
    DEPENDS ${HEADER} ${QMC_SCRIPT}
    COMMENT "qmc: meta-compiling ${REL_HEADER}..."
  )
  
  list(APPEND QMC_SOURCES ${GENERATED_CC})
endforeach()


# Engine
file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/qcl/*.cc")


add_library(qaos_qcl_shared SHARED ${SOURCES} ${RES_OBJECTS} ${QMC_SOURCES})
target_include_directories(qaos_qcl_shared PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_qcl_shared PUBLIC
  ext_skia
)


add_library(qaos_qcl_static STATIC ${SOURCES} ${RES_OBJECTS} ${QMC_SOURCES})
target_include_directories(qaos_qcl_static PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_qcl_static PUBLIC
  ext_skia
)


# Standard
file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/qstd/*.cc")


add_library(qaos_qstd_shared SHARED ${SOURCES} ${RES_OBJECTS})
target_include_directories(qaos_qstd_shared PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_qstd_shared PUBLIC
  qaos_qcl_shared
)


add_library(qaos_qstd_static STATIC ${SOURCES} ${RES_OBJECTS})
target_include_directories(qaos_qstd_static PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_qstd_static PUBLIC
  qaos_qcl_static
)


# Test App
add_subdirectory(app)
