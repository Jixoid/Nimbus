/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "qcl/Visual.hh"

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
  
  extern "C" [[gnu::noreturn]] fun qcl_error(string_view) -> void;
  extern "C" fun qcl_warning(string_view) -> void;
  
  

  enum resizeOp
  {
    wrlMove    = 0,
    wrlTop     = 1,
    wrlBot     = 2,
    wrlLeft    = 3,
    wrlRigh    = 4,
    wrlTopLeft = 5,
    wrlTopRigh = 6,
    wrlBotLeft = 7,
    wrlBotRigh = 8,
  };

  
  struct api
  {
    struct
    {
      fun (*new_)() -> handle{};
      fun (*dis_)(handle) -> void{};


      fun (*checkQueue)(handle, visual* Ctrl, visual::messages Msg) -> bool{};
      fun (*pushMessage)(handle, visual* Ctrl, visual::messages Msg) -> void{};
      fun (*pushTask)(handle, void (*Fun)(u0), u0 Data) -> void{};

      fun (*run)(handle) -> void{};
    }
    Application;

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

    struct
    {
      fun (*openFile)(string_view Title, span<const string_view> Filters) -> string{};
      fun (*saveFile)(string_view Title, span<const string_view> Filters) -> string{};
      fun (*message)(string_view Text) -> void{};
    }
    Dialog;
  };

  extern api *API;
}
