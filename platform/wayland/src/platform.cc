/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


// basis
#include "qcl/basis.hh"
// internal
#include "types.hh"
#include "listeners.hh"
// skia
#include "include/core/SkColorSpace.h"
#include "include/core/SkFontArguments.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontStyle.h"
#include "include/core/SkImage.h"
#include "include/core/SkPaint.h"
#include "include/core/SkSurface.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkTypeface.h"
#include "include/core/SkGraphics.h"
#include "include/gpu/ganesh/GrTypes.h"
#include "include/gpu/ganesh/GrBackendSurface.h"
#include "include/gpu/ganesh/GrDirectContext.h"
#include "include/gpu/ganesh/GrDirectContext.h"
#include "include/gpu/ganesh/vk/GrVkDirectContext.h"
#include "include/gpu/GpuTypes.h"
#include "src/gpu/vk/vulkanmemoryallocator/VulkanMemoryAllocatorPriv.h"
#include "include/gpu/ganesh/vk/GrVkBackendSurface.h"
#include "include/gpu/ganesh/vk/GrVkTypes.h"
#include "include/gpu/ganesh/SkSurfaceGanesh.h"
#define VK_USE_PLATFORM_WAYLAND_KHR
#include <vulkan/vulkan.hpp>
#include "include/ports/SkFontMgr_fontconfig.h"
#include "include/ports/SkFontMgr_FontConfigInterface.h"
#include "include/ports/SkFontScanner_FreeType.h"
// std
#include <iostream>
#include <chrono>
#include <cstddef>
#include <format>
#include <fstream>
#include <span>
#include <sstream>
#include <string>
#include <string_view>
// linux
#include <sys/mman.h>
// qcl
#include "qcl/application.hh"
#include "qcl/window.hh"
#include "qcl/animator.hh"
#include "qcl/platform.hh"
#include "qcl/types.hh"
#include "qcl/visual.hh"
// fontconfig
#include <fontconfig/fontconfig.h>
// xkb
#include <xkbcommon/xkbcommon.h>
// wayland
#include <wayland-cursor.h>
#include <wayland-client.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>
#include <poll.h>
// wayland extentions
#include <xdg-decoration-unstable-v1-client-protocol.h>
#include <pointer-gestures-unstable-v1-client-protocol.h>
// input
#include <linux/input-event-codes.h>

using namespace std;
using namespace qcl;

#define el else
#define ef else if

#define __CurrentApp ((app_handle*)CurrentApp->ohid())



fun trim(const string& str) -> string
{
  size_t first = str.find_first_not_of(" \t\r\n");
  if (string::npos == first) return "";
  size_t last = str.find_last_not_of(" \t\r\n");
  return str.substr(first, (last - first + 1));
}

fun loadGtkCursorConfig(const string& filePath, string& themeName, int& themeSize) -> bool
{
  ifstream file(filePath);
  if (!file.is_open())
    return false;


  string line;
  while (std::getline(file, line))
  {
    if (line.empty() || line[0] == '#' || line[0] == ';')
      continue;

    size_t eqPos = line.find('=');
    if (eqPos == string::npos)
      continue;

    string key = trim(line.substr(0, eqPos));
    string val = trim(line.substr(eqPos + 1));

    if (key == "gtk-cursor-theme-name")
    {
      if (val.size() >= 2 && val.front() == '"' && val.back() == '"')
        val = val.substr(1, val.size() - 2);
        
      themeName = val;
    }
    else if (key == "gtk-cursor-theme-size")
    {
      try {
        themeSize = std::stoi(val);
      } catch (...) {}
    }
  }

  return true;
}


fun get_gnome_font(string &name, int &size) -> bool
{
  const char* command = "gsettings get org.gnome.desktop.interface font-name";
  array<char, 128> buffer;
  string result;

  std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(command, "r"), pclose);
  if (!pipe) return false;

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr)
    result += buffer.data();

  if (result.empty()) return false;

  result.erase(std::remove(result.begin(), result.end(), '\''), result.end());
  result.erase(std::remove(result.begin(), result.end(), '\n'), result.end());

  size_t last_space = result.find_last_of(' ');
  if (last_space == string::npos) return false;

  string name_part = result.substr(0, last_space);
  string size_part = result.substr(last_space + 1);

  name_part.erase(name_part.find_last_not_of(' ') + 1);

  if (name_part.empty() || size_part.empty()) return false;

  name = name_part;
  try {
    size = std::stoi(size_part);
    return true;
  } catch (...) {
    return false;
  }
}

fun get_kde_font(int kde_ver, string &name, int &size, int &wdgt) -> bool
{
  string cmd = "kreadconfig"+to_string(kde_ver)+" --group \"General\" --key \"font\"";

  std::array<char, 128> buffer;
  std::string raw_output;

  std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(cmd.c_str(), "r"), pclose);
  if (!pipe) return false;

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
    raw_output += buffer.data();
  }

  if (raw_output.empty()) return false;

  std::stringstream ss(raw_output);
  std::string segment;
  std::vector<std::string> parts;

  while (std::getline(ss, segment, ',')) {
    parts.push_back(segment);
  }

  if (parts.size() < 5) return false;

  try {
    name = parts[0];
    size = std::stoi(parts[1]);
    wdgt = std::stoi(parts[4]);
    return true;
  } catch (...) {
    return false;
  }
}



qcl::platform::api API_Wayland = {

  .Application = {
    #define Native ((app_handle*)val)

    .new_ = []() -> handle
    {
      auto h = new app_handle();

      // Wayland
      h->display = wl_display_connect(NULL);
      if (!h->display)
        platform::qcl_error("Wayland bağlantısı kurulamadı");


      // Registry and Listeners
      h->registry = wl_display_get_registry(h->display);
      wl_registry_add_listener(h->registry, &listeners::registry_listener, h);

      wl_display_roundtrip(h->display);

      if (!h->compositor || !h->shell)
        platform::qcl_error("Gerekli Wayland protokolleri (Compositor veya XDG-Shell) bulunamadı");



      // Vulkan Initialization
      uint32_t apiVersion = VK_API_VERSION_1_1;
      auto enumerateInstanceVersion = (PFN_vkEnumerateInstanceVersion)vkGetInstanceProcAddr(nullptr, "vkEnumerateInstanceVersion");
      if (enumerateInstanceVersion)
        enumerateInstanceVersion(&apiVersion);
      
      vk::ApplicationInfo appInfo("QCL Application", 1, "QAOS", 1, apiVersion);
      
      std::vector<const char*> instanceExtensions = {
        VK_KHR_SURFACE_EXTENSION_NAME,
        VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME
      };
      
      vk::InstanceCreateInfo createInfo({}, &appInfo, {}, instanceExtensions);
      h->vk_instance = vk::createInstance(createInfo);

      auto physicalDevices = h->vk_instance.enumeratePhysicalDevices();
      if (physicalDevices.empty())
          platform::qcl_error("Vulkan destekli GPU bulunamadı");
      h->vk_physical_device = physicalDevices.front();

      std::vector<vk::QueueFamilyProperties> queueFamilies = h->vk_physical_device.getQueueFamilyProperties();
      uint32_t graphicsQueueFamilyIndex = 0;
      bool foundQueue = false;
      for (uint32_t i = 0; i < queueFamilies.size(); i++) {
        if (queueFamilies[i].queueFlags & vk::QueueFlagBits::eGraphics) {
          graphicsQueueFamilyIndex = i;
          foundQueue = true;
          break;
        }
      }
      if (!foundQueue)
        platform::qcl_error("Vulkan Graphics Queue bulunamadı");

      h->vk_queue_family_index = graphicsQueueFamilyIndex;

      float queuePriority = 1.0f;
      vk::DeviceQueueCreateInfo queueCreateInfo({}, graphicsQueueFamilyIndex, 1, &queuePriority);
      std::vector<const char*> deviceExtensions = { VK_KHR_SWAPCHAIN_EXTENSION_NAME };
      
      vk::DeviceCreateInfo deviceCreateInfo({}, queueCreateInfo, {}, deviceExtensions);
      h->vk_device = h->vk_physical_device.createDevice(deviceCreateInfo);
      h->vk_queue = h->vk_device.getQueue(graphicsQueueFamilyIndex, 0);

      auto GetProc = [](const char* name, VkInstance instance, VkDevice device) {
        PFN_vkVoidFunction proc = nullptr;
        if (device != VK_NULL_HANDLE)
          proc = vkGetDeviceProcAddr(device, name);
        else
          proc = vkGetInstanceProcAddr(instance, name);
        
        if (!proc) std::cerr << "Skia GetProc failed to load: " << name << std::endl;
        
        return proc;
      };

      h->vk_extensions.init(
        GetProc, h->vk_instance, h->vk_physical_device,
        instanceExtensions.size(), instanceExtensions.data(),
        deviceExtensions.size(), deviceExtensions.data()
      );

      h->vk_backend_context.fInstance = h->vk_instance;
      h->vk_backend_context.fPhysicalDevice = h->vk_physical_device;
      h->vk_backend_context.fDevice = h->vk_device;
      h->vk_backend_context.fQueue = h->vk_queue;
      h->vk_backend_context.fGraphicsQueueIndex = graphicsQueueFamilyIndex;
      h->vk_backend_context.fMaxAPIVersion = apiVersion;
      h->vk_backend_context.fGetProc = GetProc;
      h->vk_backend_context.fVkExtensions = &h->vk_extensions;
      
      h->vk_backend_context.fMemoryAllocator = skgpu::VulkanMemoryAllocators::Make(
        h->vk_backend_context,
        (skgpu::ThreadSafe)false
      );

      h->SkContext = GrDirectContexts::MakeVulkan(h->vk_backend_context);
      if (!h->SkContext)
        platform::qcl_error("Skia Vulkan Context oluşturulamadı");


      // XKB Keypad
      h->xkb_context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
      if (!h->xkb_context)
        platform::qcl_error("XKB context oluşturulamadı!");
      

      // Font
      auto SkFontFreeType = SkFontScanner_Make_FreeType();
      h->SkFontManager = SkFontMgr_New_FontConfig(nullptr, std::move(SkFontFreeType));
      
      string font_name{}; int font_size{10}, font_wght{400};
      
      if (get_gnome_font(font_name, font_size));
      ef (get_kde_font(5, font_name, font_size, font_wght));
      ef (get_kde_font(6, font_name, font_size, font_wght));
      el {}


      SkFontStyle FontStyle(font_wght, font_size, SkFontStyle::kUpright_Slant);
      
      h->SkFontType = h->SkFontManager->matchFamilyStyle(font_name.c_str(), FontStyle);
      if (!h->SkFontType)
        platform::qcl_error(format("font not loaded {{.name = {}, .size = {}}}", font_name, font_size));

      

      // Cursor
      string CursorTheme{}; int CursorSize{};

      string home = std::getenv("HOME") ? std::getenv("HOME") : "";

      if (loadGtkCursorConfig(home+"/.config/gtk-3.0/settings.ini", CursorTheme, CursorSize));
      ef (loadGtkCursorConfig(home+"/.config/gtk-4.0/settings.ini", CursorTheme, CursorSize));
      else {
        CursorTheme = "Adwaita";
        CursorSize = 24;
      }
      
      
      h->cursor_theme = wl_cursor_theme_load(CursorTheme.c_str(), CursorSize, h->shm);
      
      if (h->cursor_theme) {
        h->default_cursor = wl_cursor_theme_get_cursor(h->cursor_theme, "left_ptr");
        h->cursor_surface = wl_compositor_create_surface(h->compositor);
      }

      return (handle)h;
    },

    .dis_ = [](handle val) -> void
    {
      auto h = (app_handle*)val;


      // Cursor
      if (h->cursor_surface) wl_surface_destroy(h->cursor_surface);
      if (h->cursor_theme) wl_cursor_theme_destroy(h->cursor_theme);

      // XKB
      if (h->xkb_state) xkb_state_unref(h->xkb_state);
      if (h->xkb_keymap) xkb_keymap_unref(h->xkb_keymap);
      if (h->xkb_context) xkb_context_unref(h->xkb_context);
      
      // Skia
      if (h->SkContext) h->SkContext->abandonContext(); 

      // Skia Cache
      SkGraphics::PurgeAllCaches();

      // Font Config
      FcFini();

      // Vulkan
      if (h->vk_device) h->vk_device.destroy();
      if (h->vk_instance) h->vk_instance.destroy();

      // Events
      if (h->WlPointer) wl_pointer_destroy(h->WlPointer);
      if (h->WlKeyboard) wl_keyboard_destroy(h->WlKeyboard);
      if (h->WlGestures_Pinch) zwp_pointer_gesture_pinch_v1_destroy(h->WlGestures_Pinch);
      
      // Wayland Protocols
      if (h->shell) xdg_wm_base_destroy(h->shell);
      if (h->compositor) wl_compositor_destroy(h->compositor);
      if (h->shm) wl_shm_destroy(h->shm);
      if (h->seat) wl_seat_destroy(h->seat);

      if (h->xdg_decoration) zxdg_decoration_manager_v1_destroy(h->xdg_decoration);
      if (h->gestures_manager) zwp_pointer_gestures_v1_destroy(h->gestures_manager);

      // Wayland Core
      if (h->registry) wl_registry_destroy(h->registry);
      if (h->display) wl_display_disconnect(h->display);

      delete h;
    },


    .checkQueue = [](handle val, visual* Ctrl, visual::messages Msg) -> bool {return false;},
    .pushMessage = [](handle val, visual* Ctrl, visual::messages Msg) -> void
    {
      Native->pushMsg(app_message(Ctrl, Msg, false));
    },
    .pushTask = [](handle val, void (*Fun)(u0), u0 Data) -> void
    {
      Native->pushMsg(app_message(Fun, Data, false));
    },


    .skiaGrContext = [](handle val) -> GrDirectContext*
    {
      return Native->SkContext.get();
    },


    .run = [](handle val)
    {
      // Skia
      SkPaint SkiaPaint;
      SkiaPaint.setAntiAlias(true);

      SkFont SkiaFont(__CurrentApp->SkFontType);
      SkiaFont.setEdging(SkFont::Edging::kSubpixelAntiAlias);
      SkiaFont.setHinting(SkFontHinting::kSlight);
      SkiaFont.setSubpixel(true);
      SkiaFont.setBaselineSnap(true); 


      // Event Loop
      while (true)
      {
        if (CurrentApp->winList().empty())
          break;


        bool has_animations = !anim::Anims.empty();
        int timeout = has_animations ? 16 : -1;

        while (wl_display_prepare_read(Native->display) != 0)
          wl_display_dispatch_pending(Native->display);

        wl_display_flush(Native->display);

        struct pollfd pfd;
        pfd.fd = wl_display_get_fd(Native->display);
        pfd.events = POLLIN;

        int ret = poll(&pfd, 1, timeout);

        if (ret < 0) {
          wl_display_cancel_read(Native->display);
          break;
        } else if (ret == 0) {
          // Timeout
          wl_display_cancel_read(Native->display);
        } else {
          // Event
          wl_display_read_events(Native->display);
        }

        if (wl_display_dispatch_pending(Native->display) == -1) {
          platform::qcl_error("Wayland bağlantısı koptu");
          break;
        }

        
        app_message Ev;
        while (Native->popMsg(Ev))
        {
          if (Ev.Type == AppMsgType::Message) [[likely]] switch (Ev.Msg.Msg)
          {
            case visual::vmPaint: {
              if (auto QWin = dynamic_cast<window*>(Ev.Msg.Ctrl); QWin) {
                #ifdef _QCL_WAYLAND_use_painttime
                auto __painttime_start = std::chrono::high_resolution_clock::now();
                #endif

                auto WWin = (win_handle*)QWin->ohid();

                if (WWin->sk_surfaces.empty())
                  break;

                (void)__CurrentApp->vk_device.waitForFences(1, &WWin->in_flight_fence, VK_TRUE, UINT64_MAX);
                
                uint32_t imageIndex;
                vk::Result res = __CurrentApp->vk_device.acquireNextImageKHR(WWin->vk_swapchain, UINT64_MAX, WWin->image_available_semaphore, nullptr, &imageIndex);
                if (res != vk::Result::eSuccess && res != vk::Result::eSuboptimalKHR) {
                  continue;
                }
                
                __CurrentApp->vk_device.resetFences(1, &WWin->in_flight_fence);
                WWin->current_image_index = imageIndex;

                auto SkSur = WWin->sk_surfaces[imageIndex];

                renderContext Context(
                  SkSur->getCanvas(),
                  SkiaPaint,
                  SkiaFont, __CurrentApp->SkFontType, __CurrentApp->SkFontManager,
                  WWin->ScaleDP, WWin->ScaleSP, 0
                );
                
                Context.canvas()->resetMatrix();
                Context.canvas()->scale(Context.dp(1), Context.dp(1));
                Context.canvas()->clear(SK_ColorTRANSPARENT);
                
                QWin->handlerPaint(Context);
               
                __CurrentApp->SkContext->flush();
                __CurrentApp->SkContext->submit();

                // Submit to Vulkan Queue
                vk::PipelineStageFlags waitStages[] = { vk::PipelineStageFlagBits::eColorAttachmentOutput };
                vk::SubmitInfo submitInfo(1, &WWin->image_available_semaphore, waitStages, 0, nullptr, 1, &WWin->render_finished_semaphore);
                __CurrentApp->vk_queue.submit(1, &submitInfo, WWin->in_flight_fence);

                vk::PresentInfoKHR presentInfo(1, &WWin->render_finished_semaphore, 1, &WWin->vk_swapchain, &imageIndex);
                (void)__CurrentApp->vk_queue.presentKHR(&presentInfo);
                
                #ifdef _QCL_WAYLAND_use_painttime
                auto __painttime_end = std::chrono::high_resolution_clock::now();
                
                std::cout << "PaintTime: " << std::chrono::duration_cast<std::chrono::microseconds>(__painttime_end - __painttime_start).count() << " μs" << std::endl;
                #endif
              }

              break;
            }

            case visual::vmResize: {
              Ev.Msg.Ctrl->setSize(Ev.Resize.Size);
              break;
            }

            case visual::vmMouseMove: {
              auto &MMv = Ev.Mou_Mv;
              Ev.Msg.Ctrl->handlerMouseMove(MMv.Pos, MMv.State);
              break;
            }

            case visual::vmMouseDown: {
              auto &MDo = Ev.Mou_Do;
              Ev.Msg.Ctrl->handlerMouseDown(MDo.Pos, MDo.Button, MDo.State);
              break;
            }

            case visual::vmMouseUp: {
              auto &MUp = Ev.Mou_Up;
              Ev.Msg.Ctrl->handlerMouseUp(MUp.Pos, MUp.Button, MUp.State);
              break;
            }

            case visual::vmMouseScrollVert: {
              auto &SV = Ev.Mou_Srl_Vt;
              Ev.Msg.Ctrl->handlerMouseScrollVert(SV.Pos, SV.Delta, SV.State);
              break;
            }

            case visual::vmMouseScrollHorz: {
              auto &SH = Ev.Mou_Srl_Hz;
              Ev.Msg.Ctrl->handlerMouseScrollHorz(SH.Pos, SH.Delta, SH.State);
              break;
            }

            case visual::vmTouchScrollVert: {
              auto &SV = Ev.Tou_Srl_Vt;
              Ev.Msg.Ctrl->handlerTouchScrollVert(SV.Pos, SV.Delta, SV.State);
              break;
            }

            case visual::vmTouchScrollHorz: {
              auto &SH = Ev.Tou_Srl_Hz;
              Ev.Msg.Ctrl->handlerTouchScrollHorz(SH.Pos, SH.Delta, SH.State);
              break;
            }

            case visual::vmTouchScrollVertFinish: {
              auto &SV = Ev.Tou_Srl_Vt_fnsh;
              Ev.Msg.Ctrl->handlerTouchScrollVertFinish(SV.Pos, SV.AvgSpeed, SV.State);
              break;
            }

            case visual::vmTouchScrollHorzFinish: {
              auto &SH = Ev.Tou_Srl_Hz_fnsh;
              Ev.Msg.Ctrl->handlerTouchScrollHorzFinish(SH.Pos, SH.AvgSpeed, SH.State);
              break;
            }

            case visual::vmKeyDown: {
              auto &KDo = Ev.Key_Down;
              Ev.Msg.Ctrl->handlerKeyDown(KDo.Key, KDo.KeyCode, KDo.State);
              break;
            }

            case visual::vmKeyUp: {
              auto &KUp = Ev.Key_Up;
              Ev.Msg.Ctrl->handlerKeyUp(KUp.Key, KUp.KeyCode, KUp.State);
              break;
            }

            case visual::vmPlatformSpecific: {
              if (Ev.PS == PlatformSpecific::TouchPinchRotate)
              {
                if (Ev.Tou_Pinch_Rotate.Scale != 1)
                  Ev.Msg.Ctrl->handlerTouchPinch(Ev.Tou_Pinch_Rotate.Delta, Ev.Tou_Pinch_Rotate.Scale, Ev.Tou_Pinch_Rotate.State);

                if (Ev.Tou_Pinch_Rotate.Rotate != 0)
                  Ev.Msg.Ctrl->handlerTouchRotate(Ev.Tou_Pinch_Rotate.Delta, Ev.Tou_Pinch_Rotate.Rotate, Ev.Tou_Pinch_Rotate.State);
                
                break;
              }
            }

            default:
              cerr << "Ev.Ctrl: " << Ev.Msg.Ctrl << "Ev.Msg: " << Ev.Msg.Msg << endl;
          }

          else [[unlikely]]
            Ev.Task.Fun(Ev.Task.Data);
        }

        if (!anim::Anims.empty())
          anim::AnimWorker();
      }
    },

    #undef Native
  },

  .Window = {
    #define Native ((win_handle*)val)

    .new_ = []() -> handle
    {
      auto h = new win_handle();
      h->app = __CurrentApp;

      h->width = 800;
      h->height = 600;


      // Wayland Surface
      h->surface = wl_compositor_create_surface(__CurrentApp->compositor);
      if (!h->surface)
        platform::qcl_error("Wayland Surface oluşturulamadı");

      wl_surface_set_user_data(h->surface, h);


      // XDG Surface ve Toplevel
      h->xdg_surface = xdg_wm_base_get_xdg_surface(__CurrentApp->shell, h->surface);
      

      // Listener ekle
      xdg_surface_add_listener(h->xdg_surface, &listeners::xdg_surface_listener, h);

      h->toplevel = xdg_surface_get_toplevel(h->xdg_surface);
      xdg_toplevel_add_listener(h->toplevel, &listeners::xdg_toplevel_listener, h);


      // Başlık ve App ID
      xdg_toplevel_set_title(h->toplevel, "QCL Application");
      xdg_toplevel_set_app_id(h->toplevel, "com.example.app");

      wl_surface_commit(h->surface);
      wl_display_roundtrip(__CurrentApp->display);


      // Vulkan Surface
      vk::WaylandSurfaceCreateInfoKHR surfaceCreateInfo({}, __CurrentApp->display, h->surface);
      h->vk_surface = __CurrentApp->vk_instance.createWaylandSurfaceKHR(surfaceCreateInfo);
      
      if (!__CurrentApp->vk_physical_device.getSurfaceSupportKHR(__CurrentApp->vk_queue_family_index, h->vk_surface))
        platform::qcl_error("Vulkan Surface desteği yok");

      vk::SurfaceCapabilitiesKHR capabilities = __CurrentApp->vk_physical_device.getSurfaceCapabilitiesKHR(h->vk_surface);
      vk::Extent2D extent(h->width, h->height);
      
      auto presentModes = __CurrentApp->vk_physical_device.getSurfacePresentModesKHR(h->vk_surface);
      for (const auto& mode : presentModes) {
        if (mode == vk::PresentModeKHR::eMailbox) {
          h->vk_present_mode = mode;
          break;
        }
      }

      vk::SwapchainCreateInfoKHR swapchainCreateInfo(
        {}, h->vk_surface, 2, vk::Format::eB8G8R8A8Unorm, vk::ColorSpaceKHR::eSrgbNonlinear,
        extent, 1, vk::ImageUsageFlagBits::eColorAttachment | vk::ImageUsageFlagBits::eTransferDst | vk::ImageUsageFlagBits::eTransferSrc | vk::ImageUsageFlagBits::eSampled,
        vk::SharingMode::eExclusive, {}, capabilities.currentTransform,
        vk::CompositeAlphaFlagBitsKHR::eOpaque, h->vk_present_mode, true, nullptr
      );

      h->vk_swapchain = __CurrentApp->vk_device.createSwapchainKHR(swapchainCreateInfo);
      h->swapchain_images = __CurrentApp->vk_device.getSwapchainImagesKHR(h->vk_swapchain);

      // Create Skia Surfaces for each Swapchain Image
      for (size_t i = 0; i < h->swapchain_images.size(); i++) {
        GrVkImageInfo vkInfo;
        vkInfo.fImage = h->swapchain_images[i];
        vkInfo.fAlloc = skgpu::VulkanAlloc();
        vkInfo.fImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
        vkInfo.fImageTiling = VK_IMAGE_TILING_OPTIMAL;
        vkInfo.fFormat = VK_FORMAT_B8G8R8A8_UNORM;
        vkInfo.fImageUsageFlags = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
        vkInfo.fLevelCount = 1;
        vkInfo.fCurrentQueueFamily = VK_QUEUE_FAMILY_IGNORED;

        GrBackendRenderTarget renderTarget = GrBackendRenderTargets::MakeVk(h->width, h->height, vkInfo);
        
        auto sk_sur = SkSurfaces::WrapBackendRenderTarget(
          __CurrentApp->SkContext.get(), renderTarget, kTopLeft_GrSurfaceOrigin,
          kBGRA_8888_SkColorType, nullptr, nullptr
        );
        if (!sk_sur) platform::qcl_error("SkSurfaces::WrapBackendRenderTarget failed at initialization!");
        h->sk_surfaces.push_back(sk_sur);
      }

      h->image_available_semaphore = __CurrentApp->vk_device.createSemaphore(vk::SemaphoreCreateInfo());
      h->render_finished_semaphore = __CurrentApp->vk_device.createSemaphore(vk::SemaphoreCreateInfo());
      h->in_flight_fence = __CurrentApp->vk_device.createFence(vk::FenceCreateInfo(vk::FenceCreateFlagBits::eSignaled));


      // Decoration
      if (__CurrentApp->xdg_decoration) {
        h->decoration = zxdg_decoration_manager_v1_get_toplevel_decoration(
          __CurrentApp->xdg_decoration, 
          h->toplevel
        );

        zxdg_toplevel_decoration_v1_add_listener(h->decoration, &listeners::decoration_listener, h);

        zxdg_toplevel_decoration_v1_set_mode(
          h->decoration, 
          ZXDG_TOPLEVEL_DECORATION_V1_MODE_SERVER_SIDE
        );
      }

      return (handle)h;
    },

    .dis_ = [](handle val)
    {
      if (Native->decoration)
        zxdg_toplevel_decoration_v1_destroy(Native->decoration);
      

      // Skia
      Native->sk_surfaces.clear();

      // Vulkan
      if (Native->in_flight_fence) __CurrentApp->vk_device.destroyFence(Native->in_flight_fence);
      if (Native->image_available_semaphore) __CurrentApp->vk_device.destroySemaphore(Native->image_available_semaphore);
      if (Native->render_finished_semaphore) __CurrentApp->vk_device.destroySemaphore(Native->render_finished_semaphore);
      if (Native->vk_swapchain) __CurrentApp->vk_device.destroySwapchainKHR(Native->vk_swapchain);
      if (Native->vk_surface) __CurrentApp->vk_instance.destroySurfaceKHR(Native->vk_surface);

      // XDG Shell
      if (Native->toplevel) {
        xdg_toplevel_destroy(Native->toplevel);
        Native->toplevel = nullptr;
      }

      if (Native->xdg_surface) {
        xdg_surface_destroy(Native->xdg_surface);
        Native->xdg_surface = nullptr;
      }

      // Wayland Surface
      if (Native->surface) {
        wl_surface_destroy(Native->surface);
        Native->surface = nullptr;
      }

      delete Native;
    },


    .show = [](handle val)
    {
      if (!Native->surface || !Native->vk_surface) return;

      SkPaint SkiaPaint;
      SkiaPaint.setAntiAlias(true);

      SkFont SkiaFont(__CurrentApp->SkFontType);
      SkiaFont.setEdging(SkFont::Edging::kSubpixelAntiAlias);
      SkiaFont.setHinting(SkFontHinting::kSlight);
      SkiaFont.setSubpixel(true);
      SkiaFont.setBaselineSnap(true);

      if (Native->sk_surfaces.empty() || Native->current_image_index >= Native->sk_surfaces.size() || !Native->sk_surfaces[Native->current_image_index]) return;
      renderContext Context(
        Native->sk_surfaces[Native->current_image_index]->getCanvas(), 
        SkiaPaint, SkiaFont,
        __CurrentApp->SkFontType,
        __CurrentApp->SkFontManager,
        Native->ScaleDP, Native->ScaleSP, 0
      );
      Context.canvas()->resetMatrix();
      Context.canvas()->scale(Context.dp(1), Context.dp(1));
      Context.canvas()->clear(SK_ColorTRANSPARENT);
      
      __CurrentApp->SkContext->flush();
      __CurrentApp->SkContext->submit();
    },

    .hide = [](handle val)
    {
      if (!Native->surface) return;

      wl_surface_attach(Native->surface, NULL, 0, 0);
      wl_surface_commit(Native->surface);
    },

    .size = [](handle val) -> qcl::size<i32>
    {
      return qcl::size<i32>((f32)Native->width/Native->ScaleDP, (f32)Native->height/Native->ScaleDP);
    },

    .setSize = [](handle val, qcl::size<i32> Size)
    {
      if (Size.W <= 0 || Size.H <= 0 || (Size.W == Native->width && Size.H == Native->height))
        return;

      if (!Native->vk_surface || !__CurrentApp->SkContext)
        return;

      Size = qcl::size<i32>((f32)Size.W*Native->ScaleDP, (f32)Size.H*Native->ScaleDP);

      Native->width = Size.W;
      Native->height = Size.H;

      __CurrentApp->SkContext->flush();
      (void)__CurrentApp->vk_device.waitForFences(1, &Native->in_flight_fence, VK_TRUE, UINT64_MAX);


      Native->sk_surfaces.clear();
      
      vk::Extent2D extent(Native->width, Native->height);

      vk::SwapchainCreateInfoKHR swapchainCreateInfo(
        {}, Native->vk_surface, 2, vk::Format::eB8G8R8A8Unorm, vk::ColorSpaceKHR::eSrgbNonlinear,
        extent, 1, vk::ImageUsageFlagBits::eColorAttachment | vk::ImageUsageFlagBits::eTransferDst | vk::ImageUsageFlagBits::eTransferSrc | vk::ImageUsageFlagBits::eSampled,
        vk::SharingMode::eExclusive, {}, vk::SurfaceTransformFlagBitsKHR::eIdentity,
        vk::CompositeAlphaFlagBitsKHR::eOpaque, Native->vk_present_mode, true, Native->vk_swapchain
      );

      vk::SwapchainKHR old_swapchain = Native->vk_swapchain;
      Native->vk_swapchain = __CurrentApp->vk_device.createSwapchainKHR(swapchainCreateInfo);
      if (old_swapchain) __CurrentApp->vk_device.destroySwapchainKHR(old_swapchain);

      Native->swapchain_images = __CurrentApp->vk_device.getSwapchainImagesKHR(Native->vk_swapchain);

      for (size_t i = 0; i < Native->swapchain_images.size(); i++) {
        GrVkImageInfo vkInfo;
        vkInfo.fImage = Native->swapchain_images[i];
        vkInfo.fAlloc = skgpu::VulkanAlloc();
        vkInfo.fImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
        vkInfo.fImageTiling = VK_IMAGE_TILING_OPTIMAL;
        vkInfo.fFormat = VK_FORMAT_B8G8R8A8_UNORM;
        vkInfo.fImageUsageFlags = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
        vkInfo.fLevelCount = 1;
        vkInfo.fCurrentQueueFamily = VK_QUEUE_FAMILY_IGNORED;

        GrBackendRenderTarget renderTarget = GrBackendRenderTargets::MakeVk(Native->width, Native->height, vkInfo);
        
        auto sk_sur = SkSurfaces::WrapBackendRenderTarget(
          __CurrentApp->SkContext.get(), renderTarget, kTopLeft_GrSurfaceOrigin,
          kBGRA_8888_SkColorType, nullptr, nullptr
        );
        if (!sk_sur) platform::qcl_error("SkSurfaces::WrapBackendRenderTarget failed at resize!");
        Native->sk_surfaces.push_back(sk_sur);
      }

      if (Native->sk_surfaces.empty())
        platform::qcl_error("Resize sonrası Skia Surface oluşturulamadı!");
    },


    .getSurface = [](handle val) -> SkSurface* {return Native->sk_surfaces.empty() || Native->current_image_index >= Native->sk_surfaces.size() ? nullptr : Native->sk_surfaces[Native->current_image_index].get();},
    .getCanvas  = [](handle val) -> SkCanvas*  {return Native->sk_surfaces.empty() || Native->current_image_index >= Native->sk_surfaces.size() ? nullptr : Native->sk_surfaces[Native->current_image_index]->getCanvas();},

    
    .startResize = [](handle val, poit<i32> Pos, platform::resizeOp OP)
    {
      if (!__CurrentApp->seat || !Native->toplevel)
        return;

      u32 serial = __CurrentApp->last_serial; 

      if (OP == platform::resizeOp::wrlMove)
        xdg_toplevel_move(Native->toplevel, __CurrentApp->seat, serial);
      
      else {
        u32 wl_edge = 0;

        switch (OP)
        {
          case platform::resizeOp::wrlTop:     wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_TOP;          break;
          case platform::resizeOp::wrlBot:     wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_BOTTOM;       break;
          case platform::resizeOp::wrlLeft:    wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_LEFT;         break;
          case platform::resizeOp::wrlRigh:    wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_RIGHT;        break;
          case platform::resizeOp::wrlTopLeft: wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_TOP_LEFT;     break;
          case platform::resizeOp::wrlTopRigh: wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_TOP_RIGHT;    break;
          case platform::resizeOp::wrlBotRigh: wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_BOTTOM_RIGHT; break;
          case platform::resizeOp::wrlBotLeft: wl_edge = XDG_TOPLEVEL_RESIZE_EDGE_BOTTOM_LEFT;  break;
          default: return;
        }

        xdg_toplevel_resize(Native->toplevel, __CurrentApp->seat, serial, wl_edge);
      }
    },

    #undef Native
  },

  .Dialog = {

    .openFile = [](string_view Title, span<const string_view> Filters) -> string
    {
      string Cac = "zenity --file-selection --title=\""+string(Title)+"\"";
      
      for (auto &X: Filters)
        Cac += " --file-filter=\""+string(X)+"\"";

      FILE *fp = popen(Cac.c_str(), "r");
      if (fp == nil)
        return "";


      string Ret;

      char c;
      while (fread(&c, 1, 1, fp) == 1)
        Ret += c;

      int ret = pclose(fp);
      if (ret != 0 || Ret.empty())
        return "";

      if (!Ret.empty() && Ret.back() == '\n')
        Ret.pop_back();

      return Ret;
    },

    .saveFile = [](string_view Title, span<const string_view> Filters) -> string
    {
      string Cac = "zenity --file-selection --save --title=\""+string(Title)+"\"";
      
      for  (auto &X: Filters)
        Cac += " --file-filter=\""+string(X)+"\"";

      FILE *fp = popen(Cac.c_str(), "r");
      if (fp == nil)
        return "";

      string Ret;

      char c;
      while (fread(&c, 1, 1, fp) == 1)
        Ret += c;

      int ret = pclose(fp);
      if (ret != 0 || Ret.empty())
        return "";

      if (!Ret.empty() && Ret.back() == '\n')
        Ret.pop_back();

      return Ret;
    },

    .message = [](string_view Text)
    {
      string Cac = "zenity --info --text=\""+string(Text)+"\"";

      FILE *fp = popen(Cac.c_str(), "r");
      if (fp == nil)
        return;

      pclose(fp);
    },

  },
  
};

#undef __CurrentApp




[[gnu::weak]] int main()
{
  qcl::platform::API = &API_Wayland;


  if (qcl::platform::qcl_entry1)
  {
    ohid App_H = API_Wayland.Application.new_();
    ohid Win_H = API_Wayland.Window.new_();
    
    return qcl::platform::qcl_entry1(App_H, Win_H);
  }

  ef (qcl::platform::qcl_entry2)
  {
    ohid App_H = API_Wayland.Application.new_();
    
    return qcl::platform::qcl_entry2(App_H);
  }

  ef (qcl::platform::qcl_entry3)
  {
    return qcl::platform::qcl_entry3();
  }

  el [[unlikely]] {
    #ifdef _DBG
    cerr << "Not found a qcl entry" << endl;
    #endif

    return -1;
  }
}

