/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/
  

#include "qcl/basis.hh"
#define VK_USE_PLATFORM_WAYLAND_KHR
#include <vulkan/vulkan.hpp>
#include <deque>
#include <iostream>
#include <set>
#include <vector>
#include <wayland-client-protocol.h>
#include <wayland-cursor.h>
#include <xkbcommon/xkbcommon.h>
#include "include/gpu/ganesh/GrBackendSurface.h"
#include "include/gpu/ganesh/GrDirectContext.h"
#include "include/gpu/vk/VulkanBackendContext.h"
#include "include/gpu/vk/VulkanExtensions.h"
#include "include/gpu/ganesh/vk/GrVkDirectContext.h"
#include "include/gpu/ganesh/vk/GrVkBackendSurface.h"
#include "pointer-gestures-unstable-v1-client-protocol.h"
#include "qcl/application.hh"
#include "qcl/types.hh"
#include "qcl/visual.hh"
#include "qcl/window.hh"
#include "xdg-decoration-unstable-v1-client-protocol.h"
#include "xdg-shell-client-protocol.h"



struct mouse_up   { qcl::poit<f32> Pos; qcl::shiftStateSet Button; qcl::shiftStateSet State; };
struct mouse_down { qcl::poit<f32> Pos; qcl::shiftStateSet Button; qcl::shiftStateSet State; };
struct mouse_move { qcl::poit<f32> Pos; qcl::shiftStateSet State; };

struct mouse_scroll_vert { qcl::poit<f32> Pos; f32 Delta; qcl::shiftStateSet State; };
struct mouse_scroll_horz { qcl::poit<f32> Pos; f32 Delta; qcl::shiftStateSet State; };

struct touch_scroll_vert { qcl::poit<f32> Pos; f32 Delta; qcl::shiftStateSet State; };
struct touch_scroll_horz { qcl::poit<f32> Pos; f32 Delta; qcl::shiftStateSet State; };

struct touch_scroll_vert_finish { qcl::poit<f32> Pos; f32 AvgSpeed; qcl::shiftStateSet State; };
struct touch_scroll_horz_finish { qcl::poit<f32> Pos; f32 AvgSpeed; qcl::shiftStateSet State; };

struct touch_pinch_rotate { qcl::poit<f32> Delta; f32 Scale; f32 Rotate; qcl::shiftStateSet State; };

struct keypad_down { u64 Key; u32 KeyCode; qcl::shiftStateSet State; };
struct keypad_up { u64 Key; u32 KeyCode; qcl::shiftStateSet State; };

struct resize { qcl::size<i32> Size; };



enum struct PlatformSpecific: u8 { None = 0, TouchPinchRotate = 1 };
enum struct AppMsgType: u8 { Message = 1, Task = 2 };

struct app_message
{
  union
  {
    struct {
      qcl::visual *Ctrl;
      qcl::visual::messages Msg;
    } Msg;

    struct {
      void (*Fun)(u0);
      u0 Data;
    } Task;
  };
  
  union
  {
    mouse_move Mou_Mv;
    mouse_up   Mou_Up;
    mouse_down Mou_Do;
    mouse_scroll_vert Mou_Srl_Vt;
    mouse_scroll_horz Mou_Srl_Hz;
    touch_scroll_vert Tou_Srl_Vt;
    touch_scroll_horz Tou_Srl_Hz;
    touch_scroll_vert_finish Tou_Srl_Vt_fnsh;
    touch_scroll_horz_finish Tou_Srl_Hz_fnsh;
    touch_pinch_rotate Tou_Pinch_Rotate;
    keypad_down Key_Down;
    keypad_up   Key_Up;
    resize Resize;
  };

  AppMsgType Type;
  PlatformSpecific PS = PlatformSpecific::None;
  bool Foldable;


  inline app_message() {}

  inline app_message(qcl::visual *nCtrl, qcl::visual::messages nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = nMsg};
  }

  inline app_message(qcl::visual *nCtrl, mouse_down nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseDown};
    Mou_Do = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_up nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseUp};
    Mou_Up = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_move nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseMove};
    Mou_Mv = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, resize nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmResize};
    Resize = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_scroll_vert nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseScrollVert};
    Mou_Srl_Vt = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_scroll_horz nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseScrollHorz};
    Mou_Srl_Hz = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_vert nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollVert};
    Tou_Srl_Vt = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_horz nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollHorz};
    Tou_Srl_Hz = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_vert_finish nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollVertFinish};
    Tou_Srl_Vt_fnsh = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_horz_finish nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollHorzFinish};
    Tou_Srl_Hz_fnsh = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_pinch_rotate nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmPlatformSpecific};
    PS = PlatformSpecific::TouchPinchRotate;
    Tou_Pinch_Rotate = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, keypad_down nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmKeyDown};
    Key_Down = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, keypad_up nMsg, bool nFoldable) : Type(AppMsgType::Message), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmKeyUp};
    Key_Up = nMsg;
  }


  inline app_message(void (*nFun)(u0), u0 nData, bool nFoldable) : Type(AppMsgType::Task), Foldable(nFoldable)
  {
    Task = {.Fun = nFun, .Data = nData};
  }

  
  inline bool operator==(const app_message &p0) const
  {
    if (Type != p0.Type)
      return false;


    switch (Type)
    {
      case AppMsgType::Message:
        return (
          Msg.Ctrl == p0.Msg.Ctrl &&
          Msg.Msg  == p0.Msg.Msg
        );

      case AppMsgType::Task:
        return (
          Task.Fun  == p0.Task.Fun &&
          Task.Data == p0.Task.Data
        );
    
      default:
        cerr << "qcl_error: Unknown message type" << endl;
        return false;
    }
  }

};



enum struct PointerScrollType { Mouse = 1, Touch = 2 };



struct app_handle;

struct win_handle
{
  // The App
  app_handle* app{};

  // Wayland
  wl_surface* surface{};
  xdg_surface* xdg_surface{};
  xdg_toplevel* toplevel{};
  
  // Vulkan Swapchain
  vk::SurfaceKHR vk_surface;
  vk::SwapchainKHR vk_swapchain;
  std::vector<vk::Image> swapchain_images;
  std::vector<vk::ImageView> swapchain_image_views;
  std::vector<sk_sp<SkSurface>> sk_surfaces;
  vk::Semaphore image_available_semaphore;
  vk::Semaphore render_finished_semaphore;
  vk::Fence in_flight_fence;
  vk::PresentModeKHR vk_present_mode{vk::PresentModeKHR::eFifo};
  uint32_t current_image_index{0};

  // State
  int width{}, height{};
  bool configured{false};

  // Decoration
  zxdg_toplevel_decoration_v1* decoration{};

  // Scale
  f64 ScaleDP{1}, ScaleSP{1};

  // FF Event
  set<qcl::visual::messages> FullFolded;
  app_message FF_Resize, FF_Paint;


  int pending_width{}, pending_height{};
  bool needs_resize{false};
};


struct app_handle
{
  // Wayland
  wl_display* display{};
  wl_registry* registry{};
  wl_compositor* compositor{};
  xdg_wm_base* shell{}; 
  wl_seat* seat{};
  wl_shm* shm{};

  // Vulkan
  vk::Instance vk_instance;
  vk::PhysicalDevice vk_physical_device;
  vk::Device vk_device;
  vk::Queue vk_queue;
  uint32_t vk_queue_family_index{0};
  
  // Skia
  skgpu::VulkanExtensions vk_extensions;
  skgpu::VulkanBackendContext vk_backend_context;
  sk_sp<GrDirectContext>     SkContext{};
  
  // Sk Font
  sk_sp<SkFontMgr>           SkFontManager{};
  sk_sp<SkTypeface>          SkFontType{};

  // Filament
  #ifdef _QCL_use_filament
  filament::Engine *FlEngine{};
  #endif

  // Cursor
  wl_cursor_theme* cursor_theme{};
  wl_cursor* default_cursor{};
  wl_surface* cursor_surface{};

  // Decoration
  zxdg_decoration_manager_v1* xdg_decoration{};

  // Gestures
  zwp_pointer_gestures_v1 *gestures_manager{};
  f32 last_gestures_pinch{};

  // Input
  win_handle* pointer_focus_window{};
  qcl::poit<f32> pointer_last_pos{};
  qcl::shiftStateSet pointer_last_state{};
  PointerScrollType pointer_last_scroll_type{};
  f32 pointer_last_scroll_speed_x{};
  f32 pointer_last_scroll_speed_y{};

  wl_pointer*  WlPointer{};
  wl_keyboard* WlKeyboard{};
  zwp_pointer_gesture_pinch_v1 *WlGestures_Pinch{};

  // XKB Keypad
  xkb_context *xkb_context{};
  xkb_keymap *xkb_keymap{};
  xkb_state *xkb_state{};

  u32 last_serial;

  // Event Manager
  deque<app_message> Messages;

  fun pushMsg(app_message newMsg) -> void // Specific behavior: wmResize, wmPaint, wm*Scroll*, wmPlatformSpecific
  {
    if (newMsg.Type == AppMsgType::Message && dynamic_cast<qcl::window*>(newMsg.Msg.Ctrl) && newMsg.Msg.Msg == qcl::visual::vmResize)
    {
      auto qwin = static_cast<qcl::window*>(newMsg.Msg.Ctrl);
      auto wwin = (win_handle*)qwin->ohid();

      wwin->FullFolded.insert(qcl::visual::vmResize);
      wwin->FF_Resize = newMsg;
      return;
    }

    if (newMsg.Type == AppMsgType::Message && dynamic_cast<qcl::window*>(newMsg.Msg.Ctrl) && newMsg.Msg.Msg == qcl::visual::vmPaint)
    {
      auto qwin = static_cast<qcl::window*>(newMsg.Msg.Ctrl);
      auto wwin = (win_handle*)qwin->ohid();

      wwin->FullFolded.insert(qcl::visual::vmPaint);
      wwin->FF_Paint = newMsg;
      return;
    }


    if (Messages.empty() || !newMsg.Foldable) {
      Messages.push_back(newMsg);
      return;
    }

    auto &lastMsg = Messages.back();

    if (
      lastMsg.Foldable && lastMsg.Type == newMsg.Type &&

      (lastMsg.Type == AppMsgType::Message ?

        lastMsg.Msg.Ctrl == newMsg.Msg.Ctrl &&
        lastMsg.Msg.Msg  == newMsg.Msg.Msg
        :
        lastMsg.Task.Fun == newMsg.Task.Fun
      )
    ){
      if (
        lastMsg.Msg.Msg == qcl::visual::vmMouseScrollVert || lastMsg.Msg.Msg == qcl::visual::vmMouseScrollHorz ||
        lastMsg.Msg.Msg == qcl::visual::vmTouchScrollVert || lastMsg.Msg.Msg == qcl::visual::vmTouchScrollHorz
      )
        lastMsg.Mou_Srl_Vt.Delta += newMsg.Mou_Srl_Vt.Delta;

      ef (lastMsg.Msg.Msg == qcl::visual::vmPlatformSpecific)
      {
        if (lastMsg.PS == PlatformSpecific::TouchPinchRotate)
        {
          lastMsg.Tou_Pinch_Rotate.Delta += newMsg.Tou_Pinch_Rotate.Delta;
          lastMsg.Tou_Pinch_Rotate.Rotate += newMsg.Tou_Pinch_Rotate.Rotate;

          if (signbit(lastMsg.Tou_Pinch_Rotate.Scale) != signbit(newMsg.Tou_Pinch_Rotate.Scale))
            lastMsg.Tou_Pinch_Rotate.Scale = newMsg.Tou_Pinch_Rotate.Scale;
          else
            lastMsg.Tou_Pinch_Rotate.Scale += newMsg.Tou_Pinch_Rotate.Scale;
        }
      }

      else
        lastMsg = newMsg;
    }

    else
      Messages.push_back(newMsg);
  }

  fun popMsg(app_message& ret) -> bool // Specific behavior: wmResize, wmPaint
  {
    for (auto [WayW, QWin]: qcl::CurrentApp->winList())
    {
      auto WWin = (win_handle*)WayW;

      if (WWin->FullFolded.contains(qcl::visual::vmResize))
      {
        WWin->FullFolded.erase(qcl::visual::vmResize);
        ret = WWin->FF_Resize;
        return true;
      }
    }

    if (!Messages.empty())
    {
      ret = Messages.front();
      Messages.pop_front();
      return true;
    }

    for (auto [WayW, QWin]: qcl::CurrentApp->winList())
    {
      auto WWin = (win_handle*)WayW;

      if (WWin->FullFolded.contains(qcl::visual::vmPaint))
      {
        WWin->FullFolded.erase(qcl::visual::vmPaint);
        ret = WWin->FF_Paint;
        return true;
      }
    }

    return false;
  }

};
