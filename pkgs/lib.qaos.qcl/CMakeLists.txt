set(TARGET_NAME qaos_qcl)


file(GLOB_RECURSE SOURCES "src/*.cc")
file(GLOB_RECURSE HEADERS "inc/*.hh")


# QMC begin
find_package(Python3 REQUIRED)

set(QMC_SCRIPT "${CMAKE_CURRENT_SOURCE_DIR}/../../script/qmc.py")

set(QMC_SOURCES "")

foreach(HEADER ${HEADERS})
  # İçe aktarma (include) için göreceli yolu bul (Örn: "ui/widget.hh")
  file(RELATIVE_PATH REL_HEADER "${CMAKE_CURRENT_SOURCE_DIR}/inc" "${HEADER}")
  
  # Çıktı dosyasının adını benzersiz yapmak için '/' karakterlerini '_' ile değiştir
  string(REPLACE "/" "_" FLAT_HEADER "${REL_HEADER}")
  set(GENERATED_CC "${CMAKE_CURRENT_BINARY_DIR}/qmc_gen_${FLAT_HEADER}.cc")

  # QMC argümanları: qmc.py <giriş_dosyası> <çıkış_dosyası> <inc_file>
  add_custom_command(
    OUTPUT ${GENERATED_CC}
    COMMAND ${Python3_EXECUTABLE} ${QMC_SCRIPT} ${HEADER} ${GENERATED_CC} ${REL_HEADER}
    DEPENDS ${HEADER} ${QMC_SCRIPT}
    COMMENT "QMC: Meta-compiling ${REL_HEADER}..."
  )
  
  # Üretilen dosyayı kaynak listesine ekle
  list(APPEND QMC_SOURCES ${GENERATED_CC})
endforeach()
# QMC end


add_library(${TARGET_NAME}_shared SHARED ${SOURCES} ${QMC_SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES} ${QMC_SOURCES})


target_include_directories(${TARGET_NAME}_shared PUBLIC inc)

target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_skia_inc

  qaos_basis
  qaos_vfs_shared
  qaos_qcl_ds_shared
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)



target_include_directories(${TARGET_NAME}_static PUBLIC inc)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_skia_inc

  qaos_basis
  qaos_vfs_static
  qaos_qcl_ds_static
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

