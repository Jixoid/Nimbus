cmake_minimum_required(VERSION 3.20)
project(QCL CXX)


set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Debug
add_compile_definitions($<$<CONFIG:Debug>:_DBG>)
#add_compile_definitions($<$<CONFIG:Debug>:_QCL_DBG>)

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



# Childs
file(GLOB childs RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}/Pkgs" "${CMAKE_CURRENT_SOURCE_DIR}/Pkgs/*")
foreach(child ${childs})
  if(
    IS_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/Pkgs/${child}" AND
    EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/Pkgs/${child}/CMakeLists.txt"
  )
    
    message(STATUS "Sub Project Added: ${child}")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/Pkgs/${child}")
      
  endif()
endforeach()

