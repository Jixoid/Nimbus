/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/
  

#include "pointer-gestures-unstable-v1-client-protocol.h"
#include "xdg-decoration-unstable-v1-client-protocol.h"
#include "xdg-shell-client-protocol.h"



namespace listeners
{
  // Ping Pong
  extern xdg_wm_base_listener shell_listener;

  // Mouse
  extern wl_pointer_listener pointer_listener;

  // Keypad
  extern wl_keyboard_listener keyboard_listener;

  // Gestures
  extern zwp_pointer_gesture_pinch_v1_listener gestures_pinch_listener;

  // Seat
  extern wl_seat_listener seat_listener;

  // Register
  extern wl_registry_listener registry_listener;

  // XDG decoration
  extern zxdg_toplevel_decoration_v1_listener decoration_listener;
  
  // Window
  extern struct xdg_surface_listener xdg_surface_listener;

  // Toplevel
  extern struct xdg_toplevel_listener xdg_toplevel_listener;
}
