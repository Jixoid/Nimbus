/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"

#include "qcl/Object.hh"
#include "qcl/Types.hh"
#include "qcl/Widget.hh"
#include "qcl/View.hh"
#include <string_view>

using namespace std;



namespace qcl
{
  enum windowStates: u32
  {
    wsNormal    = 0,
    wsFullSrc   = 1,
    wsMaximized = 2,
    wsMinimized = 3,
  };

  enum windowOverlayMode: u32
  {
    womNone   = 0,
    womShadow = 1,
  };


  struct qcl_object window: view
  {
    public:
      window();
      ~window() override;

      window(handle ohid): view(), m_ohid(ohid), m_destroy_ohid(false) {}


    private:
      handle m_ohid{};
      bool m_destroy_ohid{true};

      windowStates m_windowState = windowStates::wsNormal;

      visual* m_overlay{};
      windowOverlayMode m_overlayMode{};


    public:
      inline fun ohid() const noexcept -> handle {return m_ohid;}

      inline fun windowState() const noexcept -> windowStates {return m_windowState;};

      inline fun overlay() const noexcept -> visual* { return m_overlay; }
      inline fun setOverlay(visual* val, windowOverlayMode WOM = windowOverlayMode::womNone) -> void { m_overlay = val; m_overlayMode = WOM; update(); }


    public:
      fun show() -> void;
      fun hide() -> void;


      fun setSize(qcl::size<i32> Val) -> void override;
      
      fun findInput(poit<i32> Pos) const -> visual* override;
      
    
    public:
      qcl_func qev<void> onCreate;
      qcl_func qev<void> onDestroy;
      qcl_func qev<void, windowStates /* State */> onWindowStateChanged;
      qcl_func qev<bool> onWindowClose;
      
      
      fun draw_after(renderContext&) -> void override;
      
      virtual fun handlerWindowStateChanged(windowStates State) -> void;
      virtual fun handlerWindowClose() -> void;
      
      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      virtual fun doWindowStateChanged(windowStates State) -> void;
      virtual fun doWindowClose() -> bool;
      virtual fun doCreate() -> void;
      virtual fun doDestroy() -> void;

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
