/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/
  

#include "Basis.hh"
#include "include/core/SkColorSpace.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontArguments.h"
#include "include/core/SkFontStyle.h"
#include "include/core/SkImage.h"
#include "include/core/SkPaint.h"
#include "include/core/SkGraphics.h"
#include "include/gpu/ganesh/GrTypes.h"
#include "qcl/Animator.hh"
#include "qcl/Platform.hh"


#include "include/gpu/ganesh/GrBackendSurface.h"
#include "include/gpu/ganesh/GrDirectContext.h"
#include "include/gpu/ganesh/gl/GrGLDirectContext.h"
#include "include/gpu/ganesh/gl/GrGLInterface.h"
#include "include/gpu/ganesh/SkSurfaceGanesh.h"
#include "include/gpu/ganesh/gl/GrGLBackendSurface.h"
#include "include/gpu/ganesh/gl/GrGLAssembleInterface.h"

#include "include/core/SkSurface.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkTypeface.h"

#include "include/ports/SkFontMgr_fontconfig.h"
#include "include/ports/SkFontMgr_FontConfigInterface.h"
#include "include/ports/SkFontScanner_FreeType.h"


#include "qcl/Types.hh"
#include "qcl/Visual.hh"
#include <cmath>
#include <cstddef>
#include <deque>
#include <fontconfig/fontconfig.h>
#include <format>
#include <fstream>
#include <set>
#include <span>
#include <sstream>
#include <string>
#include <string_view>
#define el else
#define ef else if


#include <iostream>

#include <xkbcommon/xkbcommon.h>
#include <sys/mman.h>

#include <wayland-cursor.h>
#include <wayland-client.h>
#include <wayland-egl.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>

// wayland extentions
#include <xdg-decoration-unstable-v1-client-protocol.h>
#include <pointer-gestures-unstable-v1-client-protocol.h>


#include <EGL/egl.h>

#include <linux/input-event-codes.h>

#include "qcl/Application.hh"
#include "qcl/Window.hh"

using namespace std;




struct mouse_up {
  qcl::poit<f32> Pos;
  qcl::shiftStateSet Button;
  qcl::shiftStateSet State;
};

struct mouse_down {
  qcl::poit<f32> Pos;
  qcl::shiftStateSet Button;
  qcl::shiftStateSet State;
};

struct mouse_move {
  qcl::poit<f32> Pos;
  qcl::shiftStateSet State;
};

struct mouse_scroll_vert {
  qcl::poit<f32> Pos;
  f32 Delta;
  qcl::shiftStateSet State;
};

struct mouse_scroll_horz {
  qcl::poit<f32> Pos;
  f32 Delta;
  qcl::shiftStateSet State;
};

struct touch_scroll_vert {
  qcl::poit<f32> Pos;
  f32 Delta;
  qcl::shiftStateSet State;
};

struct touch_scroll_horz {
  qcl::poit<f32> Pos;
  f32 Delta;
  qcl::shiftStateSet State;
};

struct touch_scroll_vert_finish {
  qcl::poit<f32> Pos;
  f32 AvgSpeed;
  qcl::shiftStateSet State;
};

struct touch_scroll_horz_finish {
  qcl::poit<f32> Pos;
  f32 AvgSpeed;
  qcl::shiftStateSet State;
};

struct touch_pinch_rotate {
  qcl::poit<f32> Delta;
  f32 Scale;
  f32 Rotate;
  qcl::shiftStateSet State;
};

struct keypad_down {
  u64 Key;
  u32 KeyCode;
  qcl::shiftStateSet State;
};

struct keypad_up {
  u64 Key;
  u32 KeyCode;
  qcl::shiftStateSet State;
};


struct resize
{
  qcl::size<i32> Size;
};



enum wmPlatformSpecific: u8
{
  wmpsNone = 0,

  wmpsTouchPinchRotate = 1
};



enum app_msgtyp: u8
{
  amtMessage = 1,
  amtTask    = 2,
};

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
    mouse_up Mou_Up;
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

  app_msgtyp Type;
  wmPlatformSpecific PS{wmPlatformSpecific::wmpsNone};
  bool Foldable;


  inline app_message() {}

  inline app_message(qcl::visual *nCtrl, qcl::visual::messages nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = nMsg};
  }

  inline app_message(qcl::visual *nCtrl, mouse_down nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseDown};
    Mou_Do = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_up nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseUp};
    Mou_Up = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_move nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseMove};
    Mou_Mv = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, resize nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmResize};
    Resize = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_scroll_vert nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseScrollVert};
    Mou_Srl_Vt = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, mouse_scroll_horz nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmMouseScrollHorz};
    Mou_Srl_Hz = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_vert nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollVert};
    Tou_Srl_Vt = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_horz nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollHorz};
    Tou_Srl_Hz = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_vert_finish nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollVertFinish};
    Tou_Srl_Vt_fnsh = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_scroll_horz_finish nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmTouchScrollHorzFinish};
    Tou_Srl_Hz_fnsh = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, touch_pinch_rotate nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmPlatformSpecific};
    PS = wmPlatformSpecific::wmpsTouchPinchRotate;
    Tou_Pinch_Rotate = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, keypad_down nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmKeyDown};
    Key_Down = nMsg;
  }

  inline app_message(qcl::visual *nCtrl, keypad_up nMsg, bool nFoldable) : Type(app_msgtyp::amtMessage), Foldable(nFoldable)
  {
    Msg = {.Ctrl = nCtrl, .Msg = qcl::visual::messages::vmKeyUp};
    Key_Up = nMsg;
  }


  inline app_message(void (*nFun)(u0), u0 nData, bool nFoldable) : Type(app_msgtyp::amtTask), Foldable(nFoldable)
  {
    Task = {.Fun = nFun, .Data = nData};
  }

  
  inline bool operator==(const app_message &p0) const
  {
    if (Type != p0.Type)
      return false;


    switch (Type)
    {
      case amtMessage:
        return (
          Msg.Ctrl == p0.Msg.Ctrl &&
          Msg.Msg  == p0.Msg.Msg
        );

      case amtTask:
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



enum pointer_scroll_type
{
  pstMouse,
  pstTouch,
};



struct app_handle;

struct win_handle
{
  // The App
  app_handle* app{};

  // Wayland
  wl_surface* surface{};       // Ekranda bir dikdörtgen (Canvas)
  xdg_surface* xdg_surface{};   // Yüzeyin "Pencere" rolü
  xdg_toplevel* toplevel{};      // Başlık, min/max butonları vb. için arayüz
  
  // EGL
  wl_egl_window* native_window{}; // Wayland surface ile EGL arasındaki boyutlanabilir köprü
  EGLSurface     egl_surface{};   // Üzerine çizim yapılan GL buffer'ı

  // Input
  // XIC (X Input Context) yerine Wayland'de "text_input_v3" protokolü kullanılır.
  // Şimdilik basit tutuyoruz, ancak klavye odağı için buraya listener eklenebilir.

  // State
  int width{}, height{};
  bool configured{false};

  // Decoration
  zxdg_toplevel_decoration_v1* decoration{};

  // Skia
  sk_sp<SkSurface> SkSur{};
  GrBackendRenderTarget SkBackendRenderTarget{};

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
  xdg_wm_base* shell{};      // Pencere yönetim protokolü
  wl_seat* seat{};       // Klavye/Mouse girişi için (X11'deki global input yerine)
  wl_shm* shm{};        // Cursor vb. için shared memory (Opsiyonel ama önerilir)

  // EGL
  EGLDisplay     egl_display{};
  EGLContext     egl_context{};
  EGLConfig      egl_config{};

  // Skia
  sk_sp<const GrGLInterface> SkInterface{};
  sk_sp<GrDirectContext>     SkContext{};
  
  // Sk Font
  sk_sp<SkFontMgr>           SkFontManager{};
  sk_sp<SkTypeface>          SkFontType{};

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
  pointer_scroll_type pointer_last_scroll_type{};
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
    if (newMsg.Type == app_msgtyp::amtMessage && dynamic_cast<qcl::window*>(newMsg.Msg.Ctrl) && newMsg.Msg.Msg == qcl::visual::vmResize)
    {
      auto qwin = static_cast<qcl::window*>(newMsg.Msg.Ctrl);
      auto wwin = (win_handle*)qwin->ohid();

      wwin->FullFolded.insert(qcl::visual::vmResize);
      wwin->FF_Resize = newMsg;
      return;
    }

    if (newMsg.Type == app_msgtyp::amtMessage && dynamic_cast<qcl::window*>(newMsg.Msg.Ctrl) && newMsg.Msg.Msg == qcl::visual::vmPaint)
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

      (lastMsg.Type == app_msgtyp::amtMessage ?

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
        if (lastMsg.PS == wmPlatformSpecific::wmpsTouchPinchRotate)
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



#define __CurrentApp ((app_handle*)CurrentApp->ohid())


using namespace qcl;



inline fun findWin(win_handle* WIN) -> qcl::window*
{
  for (auto &[WayWin, Win]: CurrentApp->winList())
    if (WayWin == (handle)WIN)
      return Win;

  return Nil;
};



namespace listeners
{
  // Ping Pong
  static fun xdg_wm_base_ping(void *data, struct xdg_wm_base *xdg_wm_base, uint32_t serial) -> void
  {
    xdg_wm_base_pong(xdg_wm_base, serial);
  }

  static const struct xdg_wm_base_listener shell_listener = {
    .ping = xdg_wm_base_ping,
  };




  // MOUSE (POINTER)
  static fun pointer_handle_enter(void* data, wl_pointer* pointer, uint32_t serial, wl_surface* surface, wl_fixed_t sx, wl_fixed_t sy) -> void
  {
    app_handle* app = (app_handle*)data;
    
    auto wwin = (win_handle*)wl_surface_get_user_data(surface);
    app->pointer_focus_window = wwin;
    
    f32 x = wl_fixed_to_int(sx);
    f32 y = wl_fixed_to_int(sy);
    app->pointer_last_pos = poit<f32>{x,y} /wwin->ScaleDP;
    app->pointer_last_state = {};


    // Cursor
    if (app->default_cursor && app->cursor_surface) {
      // İmlecin ilk karesini (image) al (Animasyonlu imleçler için index değişir)
      wl_cursor_image* image = app->default_cursor->images[0];
      
      // Resmin buffer'ını al
      wl_buffer* buffer = wl_cursor_image_get_buffer(image);
      
      // 1. Resmi yüzeye yapıştır
      wl_surface_attach(app->cursor_surface, buffer, 0, 0);
      
      // 2. Yüzeye hasar ver (Çizilmesini sağla)
      wl_surface_damage(app->cursor_surface, 0, 0, image->width, image->height);
      
      // 3. Onayla
      wl_surface_commit(app->cursor_surface);
      
      // 4. Pointer'a bu yüzeyi atadığını söyle
      // hotspot_x/y: Okun ucunun neresi olduğu (sol üst köşe)
      wl_pointer_set_cursor(pointer, serial, app->cursor_surface, image->hotspot_x, image->hotspot_y);
    }
  }

  static fun pointer_handle_leave(void* data, wl_pointer* pointer, uint32_t serial, wl_surface* surface) -> void
  {
    app_handle* app = (app_handle*)data;

    
    auto wwin = (win_handle*)wl_surface_get_user_data(surface);
    app->pointer_focus_window = Nil;

    app->pointer_last_state = {};
  }

  static fun pointer_handle_motion(void* data, wl_pointer* pointer, uint32_t time, wl_fixed_t sx, wl_fixed_t sy) -> void
  {
    app_handle* app = (app_handle*)data;

    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);
    
    f32 x = wl_fixed_to_int(sx);
    f32 y = wl_fixed_to_int(sy);
    app->pointer_last_pos = poit<f32>{x,y} /wwin->ScaleDP;


    app->pushMsg(app_message(qwin, mouse_move{poit<f32>{x,y} /wwin->ScaleDP, app->pointer_last_state}, true));
  }

  static fun pointer_handle_button(void* data, wl_pointer* pointer, uint32_t serial, uint32_t time, uint32_t button, uint32_t state) -> void
  {
    app_handle* app = (app_handle*)data;
    app->last_serial = serial;


    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);
    

    shiftStates LBtn{};
    switch (button)
    {
      case BTN_LEFT:   LBtn = qcl::shiftStates::ssLeft;   break;
      case BTN_RIGHT:  LBtn = qcl::shiftStates::ssRight;  break;
      case BTN_MIDDLE: LBtn = qcl::shiftStates::ssMiddle; break;

      default: LBtn = {};
    }
    

    shiftStateSet &LSta = app->pointer_last_state;
    if (state == WL_POINTER_BUTTON_STATE_PRESSED)
      LSta |= LBtn;
    else
      LSta &= ~LBtn;

    
    if (state == WL_POINTER_BUTTON_STATE_PRESSED)
      app->pushMsg(app_message(qwin, mouse_down{app->pointer_last_pos, LBtn, LSta &~LBtn}, false));
    else
      app->pushMsg(app_message(qwin, mouse_up{app->pointer_last_pos, LBtn, LSta &~LBtn}, false));
  }

  static fun pointer_handle_axis(void* data, wl_pointer* pointer, uint32_t time, uint32_t axis, wl_fixed_t value) -> void
  {
    app_handle* app = (app_handle*)data;

    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);

    f32 delta = -wl_fixed_to_double(value);


    if (axis == 0)
      app->pointer_last_scroll_speed_y = (app->pointer_last_scroll_speed_y*0.3) +(delta*0.7);
    ef (axis == 1)
      app->pointer_last_scroll_speed_x = (app->pointer_last_scroll_speed_x*0.3) +(delta*0.7);
    
    
    if (app->pointer_last_scroll_type == pointer_scroll_type::pstMouse)
    {
      if (axis == 0)
        app->pushMsg(app_message(qwin, mouse_scroll_vert{app->pointer_last_pos, delta, app->pointer_last_state}, true));
      ef (axis == 1)
        app->pushMsg(app_message(qwin, mouse_scroll_horz{app->pointer_last_pos, delta, app->pointer_last_state}, true));
    }
    ef (app->pointer_last_scroll_type == pointer_scroll_type::pstTouch)
    {
      if (axis == 0)
        app->pushMsg(app_message(qwin, touch_scroll_vert{app->pointer_last_pos, delta, app->pointer_last_state}, true));
      ef (axis == 1)
        app->pushMsg(app_message(qwin, touch_scroll_horz{app->pointer_last_pos, delta, app->pointer_last_state}, true));
    }

  }

  static fun pointer_handle_axis_source(void *data, wl_pointer *wl_pointer, uint32_t axis_source) -> void
  {
    app_handle* app = (app_handle*)data;

    if (axis_source == WL_POINTER_AXIS_SOURCE_WHEEL)
      app->pointer_last_scroll_type = pointer_scroll_type::pstMouse;
    
    ef (axis_source == WL_POINTER_AXIS_SOURCE_FINGER)
      app->pointer_last_scroll_type = pointer_scroll_type::pstTouch;
    
    else
      app->pointer_last_scroll_type = pointer_scroll_type::pstMouse;
  }

  static fun pointer_handle_axis_stop(void *data, wl_pointer *wl_pointer, uint32_t time, uint32_t axis) -> void
  {
    app_handle* app = (app_handle*)data;

    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);


    if (axis == 0)
    {
      app->pushMsg(app_message(qwin, touch_scroll_vert_finish{app->pointer_last_pos, app->pointer_last_scroll_speed_y, app->pointer_last_state}, false));
      
      //cerr << "Avg Speed Y: " << app->pointer_last_scroll_speed_y << endl;
      app->pointer_last_scroll_speed_y = 0;
    }
    ef (axis == 1)
    {
      app->pushMsg(app_message(qwin, touch_scroll_horz_finish{app->pointer_last_pos, app->pointer_last_scroll_speed_x, app->pointer_last_state}, false));
      
      //cerr << "Avg Speed X: " << app->pointer_last_scroll_speed_x << endl;
      app->pointer_last_scroll_speed_x = 0;
    }
  }

  static const struct wl_pointer_listener pointer_listener = {
    .enter = pointer_handle_enter,
    .leave = pointer_handle_leave,
    .motion = pointer_handle_motion,
    .button = pointer_handle_button,
    .axis = pointer_handle_axis,
    .frame = [](void *data, wl_pointer *wl_pointer){},
    .axis_source = pointer_handle_axis_source,
    .axis_stop = pointer_handle_axis_stop,
    .axis_discrete = [](void *data, wl_pointer *wl_pointer, uint32_t axis, int32_t discrete){},
    .axis_value120 = [](void *data, wl_pointer *wl_pointer, uint32_t axis, int32_t value120){},
    .axis_relative_direction = [](void *data, wl_pointer *wl_pointer, uint32_t axis, uint32_t direction){},
  };




  // KEYBOARD
  static fun keyboard_handle_keymap(void *data, wl_keyboard *wl_keyboard, uint32_t format, int32_t fd, uint32_t size) -> void
  {
    app_handle* app = (app_handle*)data;
    return;

    char *map_str = (char*)mmap(NULL, size, PROT_READ, MAP_PRIVATE, fd, 0);
    
    app->xkb_keymap = xkb_keymap_new_from_string(app->xkb_context, map_str, XKB_KEYMAP_FORMAT_TEXT_V1, XKB_KEYMAP_COMPILE_NO_FLAGS);
    app->xkb_state = xkb_state_new(app->xkb_keymap);
    
    munmap(map_str, size);
    close(fd);
  }
  
  static fun keyboard_handle_key(void* data, wl_keyboard* keyboard, uint32_t serial, uint32_t time, uint32_t key, uint32_t state) -> void
  {
    app_handle* app = (app_handle*)data;

    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);

    return;

    if (state == WL_KEYBOARD_KEY_STATE_PRESSED) {
      xkb_keycode_t keycode = key +8;
      
      union {
        char buffer[128];
        u64 QKey;
      };
      if (xkb_state_key_get_utf8(app->xkb_state, keycode, buffer, sizeof(buffer)) <= 0)
        QKey = 0;

      app->pushMsg(app_message(qwin, keypad_down{QKey, keycode, app->pointer_last_state}, false));
    }
    ef (state == WL_KEYBOARD_KEY_STATE_RELEASED) {
      xkb_keycode_t keycode = key +8;
      
      union {
        char buffer[128];
        u64 QKey;
      };
      if (xkb_state_key_get_utf8(app->xkb_state, keycode, buffer, sizeof(buffer)) <= 0)
        QKey = 0;

      app->pushMsg(app_message(qwin, keypad_up{QKey, keycode, app->pointer_last_state}, false));
    }
  }

  static fun keyboard_handle_modifiers(void *data, struct wl_keyboard *keyboard, uint32_t serial, uint32_t mods_depressed, uint32_t mods_latched, uint32_t mods_locked, uint32_t group) -> void
  {
    app_handle* app = (app_handle*)data;

    return;
    xkb_state_update_mask(app->xkb_state, mods_depressed, mods_latched, mods_locked, 0, 0, group);
  }

  static fun keyboard_handle_repeat(void *data, wl_keyboard *wl_keyboard, int32_t rate, int32_t delay) -> void
  {
    cerr << "REPEAT" << endl;
  }

  static const struct wl_keyboard_listener keyboard_listener = {
    .keymap = keyboard_handle_keymap, // Keymap yükleme (XKB)
    .enter = [](void*, wl_keyboard*, uint32_t, wl_surface*, wl_array*){}, // Focus in
    .leave = [](void*, wl_keyboard*, uint32_t, wl_surface*){}, // Focus out
    .key = keyboard_handle_key,
    .modifiers = keyboard_handle_modifiers,
    .repeat_info = keyboard_handle_repeat,
  };




  // Gestures
  static fun gesture_pinch_begin(void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t serial, uint32_t time, struct wl_surface *surface, uint32_t fingers)
  {
    app_handle* app = (app_handle*)data;

    app->last_gestures_pinch = 1;
  }

  static fun gesture_pinch_update(void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t time, wl_fixed_t dx, wl_fixed_t dy, wl_fixed_t scale, wl_fixed_t rotation) -> void
  {
    app_handle* app = (app_handle*)data;

    auto wwin = app->pointer_focus_window;
    auto qwin = findWin(wwin);


    f32 delta_x = wl_fixed_to_double(dx);
    f32 delta_y = wl_fixed_to_double(dy);
    
    f32 n_scale = wl_fixed_to_double(scale);
    f32 n_rotate = wl_fixed_to_double(rotation);

    auto new_scale = n_scale -app->last_gestures_pinch;
    app->last_gestures_pinch = n_scale;

    app->pushMsg(app_message(qwin, touch_pinch_rotate{poit<f32>(delta_x, delta_y), new_scale, n_rotate, app->pointer_last_state}, true));
  }

  static const struct zwp_pointer_gesture_pinch_v1_listener gestures_pinch_listener = {
    .begin = gesture_pinch_begin,
    .update = gesture_pinch_update,

    .end = [](void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t serial, uint32_t time, int32_t cancelled){},
  };




  // Seat
  static fun seat_handle_capabilities(void *data, struct wl_seat *seat, uint32_t caps) -> void
  {
    app_handle *app = (app_handle*)data;

    // --- POINTER (MOUSE) ---
    if ((caps & WL_SEAT_CAPABILITY_POINTER) && !app->WlPointer) {
      app->WlPointer = wl_seat_get_pointer(seat);
      wl_pointer_add_listener(app->WlPointer, &listeners::pointer_listener, app);


      if (app->gestures_manager)
      {
        app->WlGestures_Pinch = zwp_pointer_gestures_v1_get_pinch_gesture(app->gestures_manager, app->WlPointer);
        zwp_pointer_gesture_pinch_v1_add_listener(app->WlGestures_Pinch, &listeners::gestures_pinch_listener, app);
      }
    }
    else if (!(caps & WL_SEAT_CAPABILITY_POINTER) && app->WlPointer) {

      if (app->gestures_manager)
      {
        zwp_pointer_gesture_pinch_v1_destroy(app->WlGestures_Pinch);
        app->WlGestures_Pinch = Nil;
      }

      wl_pointer_destroy(app->WlPointer);
      app->WlPointer = Nil;
    }


    // --- KEYBOARD ---
    if ((caps & WL_SEAT_CAPABILITY_KEYBOARD) && !app->WlKeyboard) {
      app->WlKeyboard = wl_seat_get_keyboard(seat);
      wl_keyboard_add_listener(app->WlKeyboard, &listeners::keyboard_listener, app);
    }
    else if (!(caps & WL_SEAT_CAPABILITY_KEYBOARD) && app->WlKeyboard) {
      wl_keyboard_destroy(app->WlKeyboard);
      app->WlKeyboard = Nil;
    }
  }

  static const struct wl_seat_listener seat_listener = {
    .capabilities = seat_handle_capabilities,
    .name = [](void*, wl_seat*, const char*){}, // Koltuk adı (örn: "seat0")
  };




  // Register
  static fun register_handle_add(void* data, struct wl_registry* reg, uint32_t id, const char* interface, uint32_t ver) -> void
  {
    auto h = static_cast<app_handle*>(data);

    if (strcmp(interface, "wl_compositor") == 0) {
      h->compositor = (wl_compositor*)wl_registry_bind(reg, id, &wl_compositor_interface, 3);
    }

    ef (strcmp(interface, "xdg_wm_base") == 0) {
      h->shell = (xdg_wm_base*)wl_registry_bind(reg, id, &xdg_wm_base_interface, 1);
      
      xdg_wm_base_add_listener(h->shell, &shell_listener, h);
    }

    ef (strcmp(interface, "wl_seat") == 0) {
      h->seat = (wl_seat*)wl_registry_bind(reg, id, &wl_seat_interface, 7);

      wl_seat_add_listener(h->seat, &listeners::seat_listener, h);
    }

    ef (strcmp(interface, "wl_shm") == 0) {
      h->shm = (wl_shm*)wl_registry_bind(reg, id, &wl_shm_interface, 1);
    }

    ef (strcmp(interface, zxdg_decoration_manager_v1_interface.name) == 0) {
      h->xdg_decoration = (zxdg_decoration_manager_v1*)wl_registry_bind(reg, id, &zxdg_decoration_manager_v1_interface, 1);
    }

    ef (strcmp(interface, zwp_pointer_gestures_v1_interface.name) == 0) {
      h->gestures_manager = (zwp_pointer_gestures_v1*)wl_registry_bind(reg, id, &zwp_pointer_gestures_v1_interface, 1);
    }

  };

  static const struct wl_registry_listener registry_listener = {
    .global = register_handle_add,
    .global_remove = [](void* data, struct wl_registry* reg, uint32_t id) {}
  };




  // XDG decoration
  static void decoration_configure(void *data, struct zxdg_toplevel_decoration_v1 *decoration, uint32_t mode)
  {}

  static const struct zxdg_toplevel_decoration_v1_listener decoration_listener = {
    .configure = decoration_configure,
  };




  // Window
  static fun xdg_surface_configure(void *data, struct xdg_surface *xdg_surface, uint32_t serial) -> void
  {
    // Wayland sunucusuna "Boyutlandırmayı kabul ettim" onayı
    xdg_surface_ack_configure(xdg_surface, serial);
    
    auto h = (win_handle*)data;
    auto &app = h->app;
    
    if (!h->configured) {
      h->configured = true;
      // İlk çizim burada tetiklenebilir
    }
    
    auto win = findWin(h);

    
    // PAINT (X11 Expose ve ConfigureNotify karşılığı)
    if (!win)
      return;
    
    if (h->width != h->pending_width || h->height != h->pending_height)
      app->pushMsg(app_message(win, resize{qcl::size<i32>{i32((f32)h->pending_width/h->ScaleDP), i32((f32)h->pending_height/h->ScaleDP)}}, true));

    app->pushMsg(app_message(win, qcl::visual::vmPaint, true));
  }

  static const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
  };


  static fun xdg_toplevel_configure(void *data, struct xdg_toplevel *xdg_toplevel, int32_t width, int32_t height, struct wl_array *states) -> void
  {
    // X11: ConfigureNotify (Resize)
    auto h = (win_handle*)data;
    auto win = findWin(h);

    
    if (width > 0 && height > 0) {
      h->pending_width = width;
      h->pending_height = height;
    } else {
      // 0 gelirse mevcut boyutu koru veya varsayılanı kullan
      h->pending_width = h->width;
      h->pending_height = h->height;
    }
  }

  static fun __qcl_close_win(u0 data) -> void { auto win = (window*)data; delete win; }


  static fun xdg_toplevel_close(void* data, xdg_toplevel*) -> void
  {
    auto h = (win_handle*)data;
    auto &app = h->app;
    auto win = findWin(h);

    app->pushMsg(app_message(__qcl_close_win, (u0)win, false));
  }

  static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_configure,
    .close = xdg_toplevel_close, 
  };
  
}



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



qcl::platform::api API_X11 = {

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
      auto get_proc = [](void* ctx, const char* name) -> GrGLFuncPtr {
        return (GrGLFuncPtr)eglGetProcAddress(name);
      };

      h->SkInterface = GrGLMakeAssembledInterface(nullptr, get_proc);

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


    /*
    .checkQueue = [](handle val, widget* Ctrl, widgetMessages Msg) -> bool
    {
      switch (Msg)
      {
        case wmPaint:
        {
          x11_message XMsg(Ctrl, Msg);

          XEvent Cac;
          if (XPeekIfEvent(Native->dpy, &Cac, &__CheckDuplicate, (XPointer)&XMsg))
            return true;
          

          if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
            return __CheckWEvent(Native->dpy, C, Expose);
        }

        case wmMouseDown:
        {
          if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
            return __CheckWEvent(Native->dpy, C, ButtonPress);
        }

        case wmMouseUp:
        {
          if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
            return __CheckWEvent(Native->dpy, C, ButtonRelease);
        }

        case wmMouseMove:
        {
          if (auto C = dynamic_cast<qcl::window*>(Ctrl); C != Nil)
            return __CheckWEvent(Native->dpy, C, MotionNotify);
        }

        default:
          return false;
      }
    },

    .pushMessage = [](handle val, widget* Ctrl, widgetMessages Msg)
    {
      if (auto W = dynamic_cast<window*>(Ctrl); W && Msg == widgetMessages::wmPaint)
      {
        XExposeEvent expose;

        expose.type = Expose;
        expose.display = __CurrentApp->dpy;
        expose.window = ((win_handle*)W->ohid())->window;
        expose.x = 0;
        expose.y = 0;
        expose.width = 800;
        expose.height = 600;
        expose.count = 0;

        XSendEvent(__CurrentApp->dpy, ((win_handle*)W->ohid())->window, False, ExposureMask, (XEvent *)&expose);
        XFlush(__CurrentApp->dpy);
        return;
      }
      

      XEvent Ev = {};
      
      Ev.type = ClientMessage;
      Ev.xclient.message_type = __CurrentApp->qcl_ipc;
      Ev.xclient.display = Native->dpy;
      Ev.xclient.window = DefaultRootWindow(Native->dpy);
      Ev.xclient.format = 32;

      x11_message *XMsg = new x11_message(Ctrl, Msg);

      Ev.xclient.data.l[0] = (long)((u0)XMsg & 0xffffffff);
      Ev.xclient.data.l[1] = (long)(((u0)XMsg >> 32) & 0xffffffff);


      XSendEvent(Native->dpy, Native->win, False, NoEventMask, &Ev);
      XFlush(Native->dpy);
    },

    .pushTask = [](handle val, void (*Fun)(u0), u0 Data)
    {
      XEvent Ev = {};
      
      Ev.type = ClientMessage;
      Ev.xclient.message_type = __CurrentApp->qcl_ipc;
      Ev.xclient.display = Native->dpy;
      Ev.xclient.window = DefaultRootWindow(Native->dpy);
      Ev.xclient.format = 32;

      x11_message *XMsg = new x11_message(Fun, Data);

      Ev.xclient.data.l[0] = (long)((u0)XMsg & 0xffffffff);
      Ev.xclient.data.l[1] = (long)(((u0)XMsg >> 32) & 0xffffffff);
      

      XSendEvent(Native->dpy, Native->win, False, NoEventMask, &Ev);
      XFlush(Native->dpy);
    },
    */


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
                break;
              }
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


      // 7. Input Method (XIM Karşılığı)
      // Wayland'de "text_input_v3" protokolü kullanılır. 
      // Şimdilik burayı boş geçiyoruz, klavye eventleri "wl_seat" üzerinden gelecek.
      // (X11'deki XCreateIC yerine ileride text_input manager eklenebilir)

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
      if (fp == Nil)
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
      if (fp == Nil)
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
      if (fp == Nil)
        return;

      pclose(fp);
    },

  },
  
};

#undef __CurrentApp




[[gnu::weak]] int main()
{
  qcl::platform::API = &API_X11;


  if (qcl::platform::qcl_entry1)
  {
    ohid App_H = API_X11.Application.new_();
    ohid Win_H = API_X11.Window.new_();
    
    return qcl::platform::qcl_entry1(App_H, Win_H);
  }

  ef (qcl::platform::qcl_entry2)
  {
    ohid App_H = API_X11.Application.new_();
    
    return qcl::platform::qcl_entry2(App_H);
  }

  ef (qcl::platform::qcl_entry3)
  {
    return qcl::platform::qcl_entry3();
  }

  el {
    #ifdef _DBG
    cerr << "Not found a qcl entry" << endl;
    #endif

    return -1;
  }
}

