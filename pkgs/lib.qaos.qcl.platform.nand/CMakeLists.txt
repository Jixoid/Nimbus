if(NOT ANDROID)
  message(STATUS "Self Detach: QCL.PLATFORM.NAND Support Library")
  return()
endif()


set(TARGET_NAME qcl_nand)


file(GLOB_RECURSE SOURCES "Dev/*.cc")

add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})


target_include_directories(${TARGET_NAME}_shared PUBLIC Inc ${ANDROID_NDK}/sources/android/native_app_glue)
target_include_directories(${TARGET_NAME}_static PUBLIC Inc ${ANDROID_NDK}/sources/android/native_app_glue)



target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qcl_dev
)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qcl_dev
)
