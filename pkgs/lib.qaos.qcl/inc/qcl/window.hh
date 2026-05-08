/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file window.hh
 * @brief Top-level window container managing a platform surface and event loop integration.
 */

#pragma once

#include "basis.hh"

#include "qcl/object.hh"
#include "qcl/types.hh"
#include "qcl/widget.hh"
#include "qcl/view.hh"
#include <string_view>

using namespace std;



namespace qcl
{
  /**
   * @brief High-level window state descriptors.
   */
  enum windowStates: u32
  {
    wsNormal    = 0,
    wsFullSrc   = 1,
    wsMaximized = 2,
    wsMinimized = 3,
  };

  /**
   * @brief Presentation mode for window overlays.
   */
  enum windowOverlayMode: u32
  {
    womNone   = 0,
    womShadow = 1,
  };


  /**
   * @brief Top-level UI container corresponding to an OS window.
   */
  struct qcl_object window: view
  {
    public:
      /**
       * @brief Default constructor.
       */
      window();
      
      /**
       * @brief Destructor.
       */
      ~window() override;

      /**
       * @brief Constructs window wrapping an existing native handle.
       * 
       * @param ohid Native OS window handle.
       */
      window(handle ohid): view(), m_ohid(ohid), m_destroy_ohid(false) {}


    private:
      handle m_ohid{};
      bool m_destroy_ohid{true};

      windowStates m_windowState = windowStates::wsNormal;

      visual* m_overlay{};
      windowOverlayMode m_overlayMode{};


    public:
      /**
       * @brief Gets the native OS handle for this window.
       */
      inline fun ohid() const noexcept -> handle {return m_ohid;}

      /**
       * @brief Gets the current window state (e.g. Normal, Maximized).
       */
      inline fun windowState() const noexcept -> windowStates {return m_windowState;};

      /**
       * @brief Gets the active overlay visual.
       */
      inline fun overlay() const noexcept -> visual* { return m_overlay; }
      
      /**
       * @brief Sets the active overlay visual, adjusting the background rendering mode.
       * 
       * @param val Pointer to the overlay visual to present.
       * @param WOM The visual style of the overlay background.
       */
      inline fun setOverlay(visual* val, windowOverlayMode WOM = windowOverlayMode::womNone) -> void { m_overlay = val; m_overlayMode = WOM; update(); }


    public:
      /**
       * @brief Maps and presents the window on the screen.
       */
      fun show() -> void;
      
      /**
       * @brief Unmaps and hides the window.
       */
      fun hide() -> void;


      fun setSize(qcl::size<i32> Val) -> void override;
      
      /**
       * @brief Traverses the UI tree to find the visual accepting input at a position.
       * 
       * @param Pos Window-relative input coordinates.
       */
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
      fun getProp(string_view Name) -> expected<std::any, bool> override;
      fun setProp(string_view Name, std::any Prop) -> expected<void, bool> override;
      fun loadProp(string_view Name, const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap) -> expected<bool, string> override;
  };

}
