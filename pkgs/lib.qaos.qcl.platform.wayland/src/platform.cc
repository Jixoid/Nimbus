/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


// basis
#include "basis.hh"
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
#include "include/gpu/ganesh/gl/GrGLDirectContext.h"
#include "include/gpu/ganesh/gl/GrGLInterface.h"
#include "include/gpu/ganesh/SkSurfaceGanesh.h"
#include "include/gpu/ganesh/gl/GrGLBackendSurface.h"
#include "include/gpu/ganesh/gl/GrGLAssembleInterface.h"
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
#include <wayland-egl.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>
// wayland extentions
#include <xdg-decoration-unstable-v1-client-protocol.h>
#include <pointer-gestures-unstable-v1-client-protocol.h>
// egl
#include <EGL/egl.h>
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
    // Yorum satırlarını veya boş satırları atla
    if (line.empty() || line[0] == '#' || line[0] == ';')
      continue;

    // Eşittir işaretini bul
    size_t eqPos = line.find('=');
    if (eqPos == string::npos)
      continue;

    // Anahtar ve Değeri ayır
    string key = trim(line.substr(0, eqPos));
    string val = trim(line.substr(eqPos + 1));

    // Kontrol et
    if (key == "gtk-cursor-theme-name")
    {
      // Tırnak işaretleri varsa temizle (Bazı configlerde "Breeze" şeklinde olabilir)
      if (val.size() >= 2 && val.front() == '"' && val.back() == '"')
        val = val.substr(1, val.size() - 2);
        
      themeName = val;
    }
    else if (key == "gtk-cursor-theme-size")
    {
      try {
        themeSize = std::stoi(val);
      } catch (...) {
        // Sayı parse edilemezse varsayılanı koru
      }
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

  // 1. Temizlik: Tırnakları ve satır sonlarını kaldır
  result.erase(std::remove(result.begin(), result.end(), '\''), result.end());
  result.erase(std::remove(result.begin(), result.end(), '\n'), result.end());

  // 2. Ayıklama: Sondaki boşluğu bul (Font Adı ve Boyut ayrımı)
  size_t last_space = result.find_last_of(' ');
  if (last_space == string::npos) return false;

  string name_part = result.substr(0, last_space);
  string size_part = result.substr(last_space + 1);

  // Name kısmının sonundaki olası ekstra boşlukları temizle (Örn: "Roboto Flex  ")
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

  // Komutu çalıştır ve çıktıyı oku
  std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(cmd.c_str(), "r"), pclose);
  if (!pipe) return false;

  while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
    raw_output += buffer.data();
  }

  if (raw_output.empty()) return false;

  // Virgüllere göre parçala
  std::stringstream ss(raw_output);
  std::string segment;
  std::vector<std::string> parts;

  while (std::getline(ss, segment, ',')) {
    parts.push_back(segment);
  }

  // KDE formatında en az 5 parça olmalı: Name, Size, PixelSize, Hint, Weight...
  if (parts.size() < 5) return false;

  try {
    name = parts[0];              // "Inter"
    size = std::stoi(parts[1]);   // 10
    wdgt = std::stoi(parts[4]);   // 400 (Normal), 700 (Bold) vb.
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



      // EGL
      h->egl_display = eglGetDisplay((EGLNativeDisplayType)h->display);
      if (h->egl_display == EGL_NO_DISPLAY)
        platform::qcl_error("EGL Display alınamadı");


      EGLint major{}, minor{};
      if (!eglInitialize(h->egl_display, &major, &minor))
        platform::qcl_error("EGL Initialize başarısız");
      

      EGLint config_attribs[] = {
        EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
        EGL_RED_SIZE, 8,
        EGL_GREEN_SIZE, 8,
        EGL_BLUE_SIZE, 8,
        EGL_ALPHA_SIZE, 8,
        EGL_DEPTH_SIZE, 0,
        EGL_STENCIL_SIZE, 8,
        EGL_RENDERABLE_TYPE, EGL_OPENGL_BIT,
        EGL_NONE
      };

      EGLint num_configs;
      if (!eglChooseConfig(h->egl_display, config_attribs, &h->egl_config, 1, &num_configs) || num_configs == 0)
        platform::qcl_error("Uygun EGL Config bulunamadı");
      
  
        
      // EGL Context
      EGLint context_attribs[] = { EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE }; // OpenGL 2+
      eglBindAPI(EGL_OPENGL_API); 

      h->egl_context = eglCreateContext(h->egl_display, h->egl_config, EGL_NO_CONTEXT, context_attribs);
      if (h->egl_context == EGL_NO_CONTEXT)
        platform::qcl_error("EGL Context oluşturulamadı");

      eglMakeCurrent(h->egl_display, EGL_NO_SURFACE, EGL_NO_SURFACE, h->egl_context);


      // Skia
      auto GetProc = [](void* ctx, const char* name) -> GrGLFuncPtr {
        return (GrGLFuncPtr)eglGetProcAddress(name);
      };

      h->SkInterface = GrGLMakeAssembledInterface(nil, GetProc);
      if (!h->SkInterface)
        platform::qcl_error("Skia GL Interface oluşturulamadı");

      h->SkContext = GrDirectContexts::MakeGL(h->SkInterface);
      if (!h->SkContext)
        platform::qcl_error("Skia Context oluşturulamadı");


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

      // EGL
      if (h->egl_display != EGL_NO_DISPLAY) {
        eglMakeCurrent(h->egl_display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
        
        if (h->egl_context != EGL_NO_CONTEXT)
          eglDestroyContext(h->egl_display, h->egl_context);
        
        eglTerminate(h->egl_display);
      }

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

      //wl_display_flush(Native->display);
    },
    .pushTask = [](handle val, void (*Fun)(u0), u0 Data) -> void
    {
      Native->pushMsg(app_message(Fun, Data, false));

      //wl_display_flush(Native->display);
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


        if (wl_display_dispatch(Native->display) == -1) {
          platform::qcl_error("Wayland bağlantısı koptu");
          break;
        }

        
        app_message Ev;
        while (Native->popMsg(Ev))
        {
          if (Ev.Type == app_msgtyp::amtMessage) [[likely]] switch (Ev.Msg.Msg)
          {
            case visual::vmPaint: {
              if (auto QWin = dynamic_cast<window*>(Ev.Msg.Ctrl); QWin) {
                #ifdef _QCL_WAYLAND_use_painttime
                auto __painttime_start = std::chrono::high_resolution_clock::now();
                #endif

                auto WWin = (win_handle*)QWin->ohid();

                if (!WWin->SkSur)
                  break;

                renderContext Context(
                  WWin->SkSur->getCanvas(),
                  SkiaPaint,
                  SkiaFont, __CurrentApp->SkFontType, __CurrentApp->SkFontManager,
                  WWin->ScaleDP, WWin->ScaleSP, 0
                );
                
                Context.canvas()->resetMatrix();
                Context.canvas()->scale(Context.dp(1), Context.dp(1));
                Context.canvas()->clear(SK_ColorTRANSPARENT);
                
                QWin->handlerPaint(Context);
               
                
                // EGL Swap
                __CurrentApp->SkContext->flush();
                eglSwapBuffers(__CurrentApp->egl_display, WWin->egl_surface);
                
                #ifdef _QCL_WAYLAND_use_painttime
                auto __painttime_end = std::chrono::high_resolution_clock::now();
                
                std::cout << "PaintTime: "
                << std::chrono::duration_cast<std::chrono::microseconds>(__painttime_end - __painttime_start)
                .count()
                << " μs" << std::endl;
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

              if (Ev.PS == wmPlatformSpecific::wmpsTouchPinchRotate)
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

      // Varsayılan boyutlar
      h->width = 800;
      h->height = 600;


      // Wayland Surface
      h->surface = wl_compositor_create_surface(__CurrentApp->compositor);
      if (!h->surface)
        platform::qcl_error("Wayland Surface oluşturulamadı");

      wl_surface_set_user_data(h->surface, h);


      // XDG Surface ve Toplevel (Pencere Rolü)
      h->xdg_surface = xdg_wm_base_get_xdg_surface(__CurrentApp->shell, h->surface);
      


      // Listener ekle
      xdg_surface_add_listener(h->xdg_surface, &listeners::xdg_surface_listener, h);

      h->toplevel = xdg_surface_get_toplevel(h->xdg_surface);
      xdg_toplevel_add_listener(h->toplevel, &listeners::xdg_toplevel_listener, h);



      // Başlık ve App ID
      xdg_toplevel_set_title(h->toplevel, "QCL Application");
      xdg_toplevel_set_app_id(h->toplevel, "com.example.app");



      // İlk "Commit": Sunucuya "Ben buradayım, beni yapılandır" diyoruz.
      wl_surface_commit(h->surface);


      
      // Sunucudan (Compositor) cevap gelene kadar bekle (Initial Configure)
      // Bu yapılmazsa buffer boyutu belirsiz kalabilir.
      wl_display_roundtrip(__CurrentApp->display);


      // EGL Native Window (Wayland <-> EGL Köprüsü)
      h->native_window = wl_egl_window_create(h->surface, h->width, h->height);
      if (!h->native_window)
        platform::qcl_error("Wayland EGL Window oluşturulamadı");


      // EGL Surface (GLMakeCurrent öncesi hazırlık)
      h->egl_surface = eglCreateWindowSurface(
        __CurrentApp->egl_display, 
        __CurrentApp->egl_config, 
        (EGLNativeWindowType)h->native_window, 
        NULL
      );

      if (h->egl_surface == EGL_NO_SURFACE)
        platform::qcl_error("EGL Surface oluşturulamadı");


      // Context'i bu pencereye bağla (GLMakeCurrent karşılığı)
      if (!eglMakeCurrent(__CurrentApp->egl_display, h->egl_surface, h->egl_surface, __CurrentApp->egl_context))
        platform::qcl_error("EGL MakeCurrent başarısız");


      // Skia Backend Setup
      GrGLFramebufferInfo fbInfo;
      fbInfo.fFBOID = 0; 
      fbInfo.fFormat = 0x8058; // EGL Config ile uyumlu olmalı (Red 8, Green 8...)


      h->SkBackendRenderTarget = GrBackendRenderTargets::MakeGL(
        h->width, h->height,
        0, // Sample count (0 = MSAA kapalı)
        8, // Stencil bits
        fbInfo
      );


      // Skia Surface
      h->SkSur = SkSurfaces::WrapBackendRenderTarget(
        __CurrentApp->SkContext.get(),
        h->SkBackendRenderTarget,
        kBottomLeft_GrSurfaceOrigin, // OpenGL genelde BottomLeft'tir
        kRGBA_8888_SkColorType,
        nullptr,
        nullptr
      );
      if (!h->SkSur)
        platform::qcl_error("Skia Surface oluşturulamadı");


      // Decoration
      if (__CurrentApp->xdg_decoration) {
        // 1. Dekorasyon objesini al
        h->decoration = zxdg_decoration_manager_v1_get_toplevel_decoration(
          __CurrentApp->xdg_decoration, 
          h->toplevel
        );

        // 2. Listener'ı bağla
        zxdg_toplevel_decoration_v1_add_listener(h->decoration, &listeners::decoration_listener, h);

        // 3. Server Side Decoration (SSD) iste
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
      Native->SkSur.reset();

      // EGL
      if (__CurrentApp->egl_display)
      {
        eglMakeCurrent(__CurrentApp->egl_display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);

        if (Native->egl_surface) {
          eglDestroySurface(__CurrentApp->egl_display, Native->egl_surface);
          Native->egl_surface = EGL_NO_SURFACE;
        }
      }

      // Wayland EGL
      if (Native->native_window) {
        wl_egl_window_destroy(Native->native_window);
        Native->native_window = nullptr;
      }

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
      if (!Native->surface || !Native->egl_surface) return;

      // 1. Context'i Aktif Et
      eglMakeCurrent(__CurrentApp->egl_display, Native->egl_surface, Native->egl_surface, __CurrentApp->egl_context);

      // 2. Zorunlu Yeniden Çizim (Force Repaint)
      // Wayland'de pencere "buffer" gönderilene kadar görünmezdir. 
      // Show çağrıldığında hemen görünmesi için bir kare çizip göndermeliyiz.
      
      SkPaint SkiaPaint;
      SkiaPaint.setAntiAlias(true);

      SkFont SkiaFont(__CurrentApp->SkFontType);
      SkiaFont.setEdging(SkFont::Edging::kSubpixelAntiAlias); // LCD pürüzsüzlüğü
      SkiaFont.setHinting(SkFontHinting::kSlight);           // Modern fontlar için ideal
      SkiaFont.setSubpixel(true);                            // Karakter aralıklarını hassaslaştırır
      SkiaFont.setBaselineSnap(true);                         //

      // Canvas'ı temizle (isteğe bağlı, handlerPaint zaten tamamını boyuyorsa gerek yok)
      // Native->SkSur->getCanvas()->clear(SK_ColorWHITE);

      // Kullanıcının çizim fonksiyonunu çağır
      renderContext Context(
        Native->SkSur->getCanvas(), 
        SkiaPaint, SkiaFont,
        __CurrentApp->SkFontType,
        __CurrentApp->SkFontManager,
        Native->ScaleDP, Native->ScaleSP, 0
      );
      Context.canvas()->resetMatrix();
      Context.canvas()->scale(Context.dp(1), Context.dp(1));
      Context.canvas()->clear(SK_ColorTRANSPARENT);
      
      __CurrentApp->SkContext->flush();

      eglSwapBuffers(__CurrentApp->egl_display, Native->egl_surface);
    },

    .hide = [](handle val)
    {
      if (!Native->surface) return;

      // Wayland'de pencereyi yok etmeden gizlemenin (Unmap) yolu,
      // yüzeye "NULL" buffer atamaktır.
      
      // 1. İçeriği kaldır
      wl_surface_attach(Native->surface, NULL, 0, 0);

      // 2. Değişikliği onayla
      // Bu komuttan sonra pencere ekrandan kaybolur ama nesneler (xdg_surface, egl) yaşamaya devam eder.
      wl_surface_commit(Native->surface);
      
      // Not: Uygulama mantığında bu pencere için "is_visible = false" set edilmeli
      // ve render döngüsü bu pencere için durdurulmalıdır. 
      // Gizli pencereye eglSwapBuffers yapmaya devam etmek gereksiz yük oluşturur.
    },

    .size = [](handle val) -> qcl::size<i32>
    {
      return qcl::size<i32>((f32)Native->width/Native->ScaleDP, (f32)Native->height/Native->ScaleDP);
    },

    .setSize = [](handle val, qcl::size<i32> Size)
    {
      // 1. Gereksiz güncellemeyi ve hatalı boyutları engelle
      if (Size.W <= 0 || Size.H <= 0 || (Size.W == Native->width && Size.H == Native->height))
        return;

      if (!Native->native_window || !__CurrentApp->SkContext)
        return;

      Size = qcl::size<i32>((f32)Size.W*Native->ScaleDP, (f32)Size.H*Native->ScaleDP);

      Native->width = Size.W;
      Native->height = Size.H;

      // 2. Wayland-EGL Penceresini Boyutlandır
      // Bu fonksiyon, bir sonraki swapBuffers işleminde allocate edilecek
      // buffer'ların boyutunu belirler. (dx, dy genellikle 0 verilir)
      wl_egl_window_resize(Native->native_window, Size.W, Size.H, 0, 0);


      // 3. Skia Surface'ı Yeniden Oluştur (Recreation)
      // Eski surface eski boyuttadır, onu yok edip yeni boyutta oluşturmalıyız.
      
      // a. Eskiyi temizle
      Native->SkSur.reset(); 
      // Not: SkBackendRenderTarget, Skia tarafından genellikle value type olarak tutulur 
      // ama wrap işlemi için yenisini oluşturacağız.

      // b. Yeni Framebuffer Bilgisi
      GrGLFramebufferInfo fbInfo;
      fbInfo.fFBOID = 0; // EGL Surface için her zaman 0
      fbInfo.fFormat = 0x8058;

      // c. Yeni Render Target
      Native->SkBackendRenderTarget = GrBackendRenderTargets::MakeGL(
        Native->width,
        Native->height,
        0, // sample count
        8, // stencil bits
        fbInfo
      );

      // d. Yeni Skia Surface
      Native->SkSur = SkSurfaces::WrapBackendRenderTarget(
        __CurrentApp->SkContext.get(),
        Native->SkBackendRenderTarget,
        kBottomLeft_GrSurfaceOrigin,
        kRGBA_8888_SkColorType,
        nullptr,
        nullptr
      );

      if (!Native->SkSur)
        platform::qcl_error("Resize sonrası Skia Surface oluşturulamadı!");

      // 4. İsteğe Bağlı: Hemen bir 'Paint' tetiklemesi yapılabilir.
      // Eğer yapmazsan, bir sonraki döngüde çizim yapılana kadar 
      // pencere içeriği eski boyutta (crop veya stretch) görünebilir 
      // ya da siyah kalabilir.
      // window_invalidate(val) gibi bir çağrı buraya eklenebilir.
    },


    .getSurface = [](handle val) -> SkSurface* {return Native->SkSur.get();},
    .getCanvas  = [](handle val) -> SkCanvas*  {return Native->SkSur->getCanvas();},

    
    .startResize = [](handle val, poit<i32> Pos, platform::resizeOp OP)
    {
      // Eğer gerekli Wayland objeleri yoksa çık
      if (!__CurrentApp->seat || !Native->toplevel)
        return;

      // ÖNEMLİ: Wayland'de bu işlemi başlatmak için Mouse Down olayından gelen
      // 'serial' numarası zorunludur. Bunu pointer listener'da kaydetmiş olmalısın.
      uint32_t serial = __CurrentApp->last_serial; 

      // 1. TAŞIMA (MOVE) İŞLEMİ
      if (OP == platform::resizeOp::wrlMove)
      {
        // Wayland koordinat sormaz, o anki mouse pozisyonunu baz alır.
        xdg_toplevel_move(Native->toplevel, __CurrentApp->seat, serial);
      }
      // 2. BOYUTLANDIRMA (RESIZE) İŞLEMİ
      else
      {
        uint32_t wl_edge = 0;

        // QCL enumlarını Wayland XDG enumlarına eşle
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
      string Cac = "zenity --file-selection";

      Cac += " --title=\""+string(Title)+"\"";

      
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


      // sonundaki \n karakterini kaldır
      if (!Ret.empty() && Ret.back() == '\n')
        Ret.pop_back();


      return Ret;
    },

    .saveFile = [](string_view Title, span<const string_view> Filters) -> string
    {
      string Cac = "zenity --file-selection --save";

      Cac += " --title=\""+string(Title)+"\"";

      
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


      // sonundaki \n karakterini kaldır
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

