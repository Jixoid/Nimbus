
if(LINUX)
  add_subdirectory(wayland)
elseif(WIN32)
  add_subdirectory(win32)
elseif(ANDROID)
  add_subdirectory(nand)
endif()
