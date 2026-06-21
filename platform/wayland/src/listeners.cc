/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/
  

#include "qcl/basis.hh"
#include "types.hh"
#include "listeners.hh"
#include <linux/input-event-codes.h>
#include <sys/mman.h>
#include <unistd.h>
#include "xdg-shell-client-protocol.h"
#include "qcl/types.hh"

using namespace std;
using namespace qcl;



inline fun findWin(win_handle* WIN) -> qcl::window*
{
  for (auto &[WayWin, Win]: CurrentApp->winList())
    if (WayWin == (handle)WIN)
      return Win;

  return nil;
};


namespace listeners
{
  // Ping Pong
  xdg_wm_base_listener shell_listener = {
    .ping = [](void *data, struct xdg_wm_base *xdg_wm_base, uint32_t serial) -> void
    {
      xdg_wm_base_pong(xdg_wm_base, serial);
    },
  };



  // MOUSE
  wl_pointer_listener pointer_listener = {
    .enter = [](void* data, wl_pointer* pointer, uint32_t serial, wl_surface* surface, wl_fixed_t sx, wl_fixed_t sy) -> void
    {
      app_handle* app = (app_handle*)data;
      
      auto wwin = (win_handle*)wl_surface_get_user_data(surface);
      app->pointer_focus_window = wwin;
      
      f32 x = wl_fixed_to_int(sx);
      f32 y = wl_fixed_to_int(sy);
      app->pointer_last_pos = qcl::poit<f32>{x,y} /wwin->ScaleDP;
      app->pointer_last_state = {};


      // Cursor
      if (app->default_cursor && app->cursor_surface) {
        wl_cursor_image* image = app->default_cursor->images[0];
        
        wl_buffer* buffer = wl_cursor_image_get_buffer(image);
        
        wl_surface_attach(app->cursor_surface, buffer, 0, 0);
        wl_surface_damage(app->cursor_surface, 0, 0, image->width, image->height);
        wl_surface_commit(app->cursor_surface);
        wl_pointer_set_cursor(pointer, serial, app->cursor_surface, image->hotspot_x, image->hotspot_y);
      }
    },
    .leave = [](void* data, wl_pointer* pointer, uint32_t serial, wl_surface* surface) -> void
    {
      app_handle* app = (app_handle*)data;

      
      auto wwin = (win_handle*)wl_surface_get_user_data(surface);
      app->pointer_focus_window = nil;

      app->pointer_last_state = {};
    },
    .motion = [](void* data, wl_pointer* pointer, uint32_t time, wl_fixed_t sx, wl_fixed_t sy) -> void
    {
      app_handle* app = (app_handle*)data;

      auto wwin = app->pointer_focus_window;
      auto qwin = findWin(wwin);
      
      f32 x = wl_fixed_to_int(sx);
      f32 y = wl_fixed_to_int(sy);
      app->pointer_last_pos = qcl::poit<f32>{x,y} /wwin->ScaleDP;


      app->pushMsg(app_message(qwin, mouse_move{poit<f32>{x,y} /wwin->ScaleDP, app->pointer_last_state}, true));
    },
    .button = [](void* data, wl_pointer* pointer, uint32_t serial, uint32_t time, uint32_t button, uint32_t state) -> void
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
    },
    .axis = [](void* data, wl_pointer* pointer, uint32_t time, uint32_t axis, wl_fixed_t value) -> void
    {
      app_handle* app = (app_handle*)data;

      auto wwin = app->pointer_focus_window;
      auto qwin = findWin(wwin);

      f32 delta = -wl_fixed_to_double(value);


      if (axis == 0)
        app->pointer_last_scroll_speed_y = (app->pointer_last_scroll_speed_y*0.3) +(delta*0.7);
      ef (axis == 1)
        app->pointer_last_scroll_speed_x = (app->pointer_last_scroll_speed_x*0.3) +(delta*0.7);
      
      
      if (app->pointer_last_scroll_type == PointerScrollType::Mouse)
      {
        if (axis == 0)
          app->pushMsg(app_message(qwin, mouse_scroll_vert{app->pointer_last_pos, delta, app->pointer_last_state}, true));
        ef (axis == 1)
          app->pushMsg(app_message(qwin, mouse_scroll_horz{app->pointer_last_pos, delta, app->pointer_last_state}, true));
      }
      ef (app->pointer_last_scroll_type == PointerScrollType::Touch)
      {
        if (axis == 0)
          app->pushMsg(app_message(qwin, touch_scroll_vert{app->pointer_last_pos, delta, app->pointer_last_state}, true));
        ef (axis == 1)
          app->pushMsg(app_message(qwin, touch_scroll_horz{app->pointer_last_pos, delta, app->pointer_last_state}, true));
      }

    },
    .frame = [](void *data, wl_pointer *wl_pointer){},
    .axis_source = [](void *data, wl_pointer *wl_pointer, uint32_t axis_source) -> void
    {
      app_handle* app = (app_handle*)data;

      if (axis_source == WL_POINTER_AXIS_SOURCE_WHEEL)
        app->pointer_last_scroll_type = PointerScrollType::Mouse;
      
      ef (axis_source == WL_POINTER_AXIS_SOURCE_FINGER)
        app->pointer_last_scroll_type = PointerScrollType::Touch;
      
      else
        app->pointer_last_scroll_type = PointerScrollType::Mouse;
    },
    .axis_stop = [](void *data, wl_pointer *wl_pointer, uint32_t time, uint32_t axis) -> void
    {
      app_handle* app = (app_handle*)data;

      auto wwin = app->pointer_focus_window;
      auto qwin = findWin(wwin);


      if (axis == 0)
      {
        app->pushMsg(app_message(qwin, touch_scroll_vert_finish{app->pointer_last_pos, app->pointer_last_scroll_speed_y, app->pointer_last_state}, false));
        app->pointer_last_scroll_speed_y = 0;
      }
      ef (axis == 1)
      {
        app->pushMsg(app_message(qwin, touch_scroll_horz_finish{app->pointer_last_pos, app->pointer_last_scroll_speed_x, app->pointer_last_state}, false));
        app->pointer_last_scroll_speed_x = 0;
      }
    },
    .axis_discrete = [](void *data, wl_pointer *wl_pointer, uint32_t axis, int32_t discrete){},
    .axis_value120 = [](void *data, wl_pointer *wl_pointer, uint32_t axis, int32_t value120){},
    .axis_relative_direction = [](void *data, wl_pointer *wl_pointer, uint32_t axis, uint32_t direction){},
  };



  // KEYBOARD
  wl_keyboard_listener keyboard_listener = {
    .keymap = [](void *data, wl_keyboard *wl_keyboard, uint32_t format, int32_t fd, uint32_t size) -> void
    {
      app_handle* app = (app_handle*)data;
      return;

      char *map_str = (char*)mmap(NULL, size, PROT_READ, MAP_PRIVATE, fd, 0);
      
      app->xkb_keymap = xkb_keymap_new_from_string(app->xkb_context, map_str, XKB_KEYMAP_FORMAT_TEXT_V1, XKB_KEYMAP_COMPILE_NO_FLAGS);
      app->xkb_state = xkb_state_new(app->xkb_keymap);
      
      munmap(map_str, size);
      close(fd);
    },
    .enter = [](void*, wl_keyboard*, uint32_t, wl_surface*, wl_array*){},
    .leave = [](void*, wl_keyboard*, uint32_t, wl_surface*){},
    .key = [](void* data, wl_keyboard* keyboard, uint32_t serial, uint32_t time, uint32_t key, uint32_t state) -> void
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
    },
    .modifiers = [](void *data, struct wl_keyboard *keyboard, uint32_t serial, uint32_t mods_depressed, uint32_t mods_latched, uint32_t mods_locked, uint32_t group) -> void
    {
      app_handle* app = (app_handle*)data;

      return;
      xkb_state_update_mask(app->xkb_state, mods_depressed, mods_latched, mods_locked, 0, 0, group);
    },
    .repeat_info = [](void *data, wl_keyboard *wl_keyboard, int32_t rate, int32_t delay) -> void
    {
      cerr << "REPEAT" << endl;
    },
  };



  // Gestures
  zwp_pointer_gesture_pinch_v1_listener gestures_pinch_listener = {
    .begin = [](void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t serial, uint32_t time, struct wl_surface *surface, uint32_t fingers)
    {
      app_handle* app = (app_handle*)data;

      app->last_gestures_pinch = 1;
    },
    .update = [](void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t time, wl_fixed_t dx, wl_fixed_t dy, wl_fixed_t scale, wl_fixed_t rotation) -> void
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
    },

    .end = [](void *data, struct zwp_pointer_gesture_pinch_v1 *zwp_pointer_gesture_pinch_v1, uint32_t serial, uint32_t time, int32_t cancelled){},
  };



  // Seat
  wl_seat_listener seat_listener = {
    .capabilities = [](void *data, struct wl_seat *seat, uint32_t caps) -> void
    {
      app_handle *app = (app_handle*)data;

      /// MOUSE
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
          app->WlGestures_Pinch = nil;
        }

        wl_pointer_destroy(app->WlPointer);
        app->WlPointer = nil;
      }


      /// KEYBOARD
      if ((caps & WL_SEAT_CAPABILITY_KEYBOARD) && !app->WlKeyboard) {
        app->WlKeyboard = wl_seat_get_keyboard(seat);
        wl_keyboard_add_listener(app->WlKeyboard, &listeners::keyboard_listener, app);
      }
      else if (!(caps & WL_SEAT_CAPABILITY_KEYBOARD) && app->WlKeyboard) {
        wl_keyboard_destroy(app->WlKeyboard);
        app->WlKeyboard = nil;
      }
    },
    .name = [](void*, wl_seat*, const char*){},
  };



  // Register
  wl_registry_listener registry_listener = {
    .global = [](void* data, struct wl_registry* reg, uint32_t id, const char* interface, uint32_t ver) -> void
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

    },
    .global_remove = [](void* data, struct wl_registry* reg, uint32_t id) {}
  };



  // XDG decoration
  zxdg_toplevel_decoration_v1_listener decoration_listener = {
    .configure = [](void *data, struct zxdg_toplevel_decoration_v1 *decoration, uint32_t mode){},
  };



  // Window
  struct xdg_surface_listener xdg_surface_listener = {
    .configure = [](void *data, struct xdg_surface *xdg_surface, uint32_t serial) -> void
    {
      xdg_surface_ack_configure(xdg_surface, serial);
      
      auto h = (win_handle*)data;
      auto &app = h->app;
      
      if (!h->configured) { h->configured = true; }
      
      auto win = findWin(h);

      if (!win)
        return;
      
      if (h->width != h->pending_width || h->height != h->pending_height)
        app->pushMsg(app_message(win, resize{qcl::size<i32>{i32((f32)h->pending_width/h->ScaleDP), i32((f32)h->pending_height/h->ScaleDP)}}, true));

      app->pushMsg(app_message(win, qcl::visual::vmPaint, true));
    },
  };



  // Toplevel
  struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = [](void *data, struct xdg_toplevel *xdg_toplevel, int32_t width, int32_t height, struct wl_array *states) -> void
    {
      auto h = (win_handle*)data;
      auto win = findWin(h);

      
      if (width > 0 && height > 0) {
        h->pending_width = width;
        h->pending_height = height;
      } else {
        h->pending_width = h->width;
        h->pending_height = h->height;
      }
    },
    .close = [](void* data, xdg_toplevel*) -> void
    {
      auto h = (win_handle*)data;
      auto &app = h->app;
      auto win = findWin(h);

      static auto __qcl_close_win = [](u0 data) -> void {
        auto win = (window*)data;
        delete win;
      };

      app->pushMsg(app_message(__qcl_close_win, (u0)win, false));
    }, 
  };
  
}
