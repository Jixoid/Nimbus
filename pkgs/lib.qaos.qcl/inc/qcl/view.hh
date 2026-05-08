/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file view.hh
 * @brief Container class for grouping and managing visual objects.
 */

#pragma once

#include "basis.hh"
#include <expected>
#include <string_view>
#include <vector>
#include "ds/ds.hh"
#include "qcl/object.hh"
#include "qcl/types.hh"
#include "qcl/visual.hh"
#include "qcl/widget.hh"

using namespace std;



namespace qcl
{

  /**
   * @brief A composite visual element that contains and manages other objects and widgets.
   */
  struct qcl_object view: widget
  {
    public:
      /**
       * @brief Default constructor for view.
       */
      inline view() {}
      
      /**
       * @brief Destructor. Cleans up all child objects and widgets.
       */
      inline ~view() override {
        for (auto &X: m_objects) delete X;
        m_objects.clear();

        for (auto &X: m_widgets) delete X;
        m_widgets.clear();
      }

    private:
      vector<widget*> m_widgets;
      vector<object*> m_objects;

      visual *m_hoverControl{}, *m_focusControl{};

      fun f_tilingScanH(vector<widget*> &HList, widget* This) -> void;
      fun f_tilingScanV(vector<widget*> &HList, widget* This) -> void;
    
    protected:
      /**
       * @brief Sets the visual control that currently has mouse hover state.
       * 
       * @param Ctrl Pointer to the hovered visual.
       */
      fun setHoverControl(visual* Ctrl) -> void;
      
      /**
       * @brief Sets the visual control that currently has input focus.
       * 
       * @param Ctrl Pointer to the focused visual.
       */
      fun setFocusControl(visual* Ctrl) -> void;

    public:
      /**
       * @brief Gets a list of child widgets.
       */
      inline fun widgets() const noexcept -> const vector<widget*>& {return m_widgets;};
      
      /**
       * @brief Gets a list of all child objects (non-visual included).
       */
      inline fun objects() const noexcept -> const vector<object*>& {return m_objects;};
      
      /**
       * @brief Computes a unified list of all children.
       */
      fun childs() const noexcept -> const vector<object*>;

      /**
       * @brief Gets the control currently being hovered.
       */
      inline fun hoverControl() const noexcept -> visual* {return m_hoverControl;}
      
      /**
       * @brief Gets the control currently holding focus.
       */
      inline fun focusControl() const noexcept -> visual* {return m_focusControl;}


    public:
      /**
       * @brief Adds a child object to the view.
       */
      fun addChild(object*) -> void;
      
      /**
       * @brief Removes a child object from the view and deletes it.
       */
      fun remChild(object*) -> void;
      
      /**
       * @brief Releases a child object from the view without deleting it.
       * @return Pointer to the released object.
       */
      [[nodiscard]] fun relChild(object*) -> object*;

      /**
       * @brief Finds the target visual control for input at the specified coordinates.
       * 
       * @param Pos Input position relative to this view.
       * @return Pointer to the targeted visual, or null.
       */
      virtual fun findInput(poit<i32> Pos) const -> visual*;

      fun draw_after(renderContext&) -> void override;
      fun calcAutoSize(renderContext&) -> void override;
      
      fun handlerStateChanged(visual::stateFlagSet State) -> void override;
      
      /**
       * @brief Performs layout calculations for child widgets.
       */
      virtual fun doTiling(renderContext&) -> void;
      
      fun doReset         (renderContext&) -> void override;
      fun doPaint_prepare (renderContext&) -> void override;
      fun doPaint         (renderContext&) -> void override;
      fun doResize        () -> void override;

      fun doMouseDown       (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseUp         (poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove       (poit<f32> Pos, shiftStateSet State) -> void override;
      fun doMouseScrollVert (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;
      fun doMouseScrollHorz (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;

      fun doTouchScrollVert (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;
      fun doTouchScrollHorz (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;
      fun doTouchPinch      (poit<f32> Delta, f32 Scale, shiftStateSet State) -> void override;
      fun doTouchRotate     (poit<f32> Delta, f32 Rotate, shiftStateSet State) -> void override;
      
      fun doKeyDown (u64 Key, u32 KeyCode, shiftStateSet State) -> void override;
      fun doKeyUp   (u64 Key, u32 KeyCode, shiftStateSet State) -> void override;

    public:
      fun getProp(string_view Name) -> expected<std::any, bool> override;
      fun setProp(string_view Name, std::any Prop) -> expected<void, bool> override;
      fun loadProp(string_view Name, const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap) -> expected<bool, string> override;
  };

}
