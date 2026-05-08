/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file platform.hh
 * @brief Platform-specific backend integrations and API wrappers.
 */

#pragma once

#include "include/gpu/ganesh/GrDirectContext.h"
#include "qcl/visual.hh"
#include <EGL/egl.h>
#include <GLES2/gl2.h>
#include "include/core/SkCanvas.h"
#include <span>
#include <string_view>

using namespace std;



namespace qcl::platform
{
  #ifdef __unix__
  extern "C" [[gnu::weak]] fun qcl_entry1(ohid App_H, ohid Win_H) -> int;
  extern "C" [[gnu::weak]] fun qcl_entry2(ohid App_H) -> int;
  extern "C" [[gnu::weak]] fun qcl_entry3() -> int;
  #endif
  
  /**
   * @brief Logs a fatal platform error and terminates the application.
   * 
   * @param Msg The error message to log.
   */
  extern "C" [[gnu::noreturn]] fun qcl_error(string_view Msg) -> void;
  
  /**
   * @brief Logs a platform warning.
   * 
   * @param Msg The warning message to log.
   */
  extern "C" fun qcl_warning(string_view Msg) -> void;
  
  

  /**
   * @brief Indicates the type of window resizing or moving operation.
   */
  enum resizeOp {
    wrlMove    = 0, ///< Move operation
    wrlTop     = 1, ///< Top edge resize
    wrlBot     = 2, ///< Bottom edge resize
    wrlLeft    = 3, ///< Left edge resize
    wrlRigh    = 4, ///< Right edge resize
    wrlTopLeft = 5, ///< Top-left corner resize
    wrlTopRigh = 6, ///< Top-right corner resize
    wrlBotLeft = 7, ///< Bottom-left corner resize
    wrlBotRigh = 8, ///< Bottom-right corner resize
  };

  
  /**
   * @brief Structure defining the function pointers for platform-specific APIs.
   */
  struct api
  {
    /**
     * @brief Application-level platform API hooks.
     */
    struct
    {
      fun (*new_)() -> handle{};
      fun (*dis_)(handle) -> void{};

      fun (*checkQueue)(handle, visual* Ctrl, visual::messages Msg) -> bool{};
      fun (*pushMessage)(handle, visual* Ctrl, visual::messages Msg) -> void{};
      fun (*pushTask)(handle, void (*Fun)(u0), u0 Data) -> void{};

      fun (*skiaGrContext)(handle) -> GrDirectContext*{};

      fun (*run)(handle) -> void{};
    }
    Application;

    /**
     * @brief Window-level platform API hooks.
     */
    struct
    {
      fun (*new_)() -> handle{};
      fun (*dis_)(handle) -> void{};

      fun (*show)(handle) -> void{};
      fun (*hide)(handle) -> void{};

      fun (*size)(handle) -> size<i32>{};
      fun (*setSize)(handle, qcl::size<i32> Size) -> void{};

      fun (*getSurface)(handle) -> SkSurface*;
      fun (*getCanvas)(handle) -> SkCanvas*;

      fun (*startResize)(handle, poit<i32> Pos, resizeOp OP) -> void{};
    }
    Window;

    /**
     * @brief Native dialog platform API hooks.
     */
    struct
    {
      fun (*openFile)(string_view Title, span<const string_view> Filters) -> string{};
      fun (*saveFile)(string_view Title, span<const string_view> Filters) -> string{};
      fun (*message)(string_view Text) -> void{};
    }
    Dialog;
  };

  /**
   * @brief Global pointer to the active platform API implementation.
   */
  extern api *API;
}
