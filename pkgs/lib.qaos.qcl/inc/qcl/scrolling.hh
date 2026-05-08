/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file scrolling.hh
 * @brief Scrollable view components and event handling.
 */

#pragma once

#include "basis.hh"
#include <expected>
#include <string_view>
#include "qcl/object.hh"
#include "qcl/types.hh"
#include "qcl/visual.hh"
#include "qcl/view.hh"
#include "ds/ds.hh"

using namespace std;



namespace qcl
{

  /**
   * @brief A scrollable view container that manages content overflow and scrollbars.
   */
  struct qcl_object scrolling: view
  {
    public:
      /**
       * @brief Default constructor for scrolling view.
       */
      inline scrolling() {}
      
      /**
       * @brief Virtual destructor.
       */
      inline ~scrolling() override {}

    private:
      rect<i32> m_clientBound{};
      poit<i32> m_clientPos{};
      
      bool m_scrollVertVisible{true};
      bool m_scrollHorzVisible{true};
      
      bool ScrollableVert{false};
      bool ScrollableHorz{false};
      
      
    public:
      /**
       * @brief Gets whether the vertical scrollbar is visible.
       * 
       * @return True if visible, false otherwise.
       */
      inline fun scrollVertVisible() const noexcept -> bool {return m_scrollVertVisible;}
      
      /**
       * @brief Sets the visibility of the vertical scrollbar.
       * 
       * @param Val True to show, false to hide.
       */
      inline fun setScrollVertVisible(bool Val) noexcept -> void {m_scrollVertVisible = Val; update(); }
      
      /**
       * @brief Loads the vertical scroll visibility property from a ds::value.
       * 
       * @param Prop The property value definition.
       * @return True on success, or an error string.
       */
      inline fun propScrollVertVisible(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setScrollVertVisible(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME scrollVertVisible, READ scrollVertVisible, WRITE setScrollVertVisible, PROP propScrollVertVisible);
      
      
      /**
       * @brief Gets whether the horizontal scrollbar is visible.
       * 
       * @return True if visible, false otherwise.
       */
      inline fun scrollHorzVisible() const noexcept -> bool {return m_scrollHorzVisible;}
      
      /**
       * @brief Sets the visibility of the horizontal scrollbar.
       * 
       * @param Val True to show, false to hide.
       */
      inline fun setScrollHorzVisible(bool Val) noexcept -> void {m_scrollHorzVisible = Val; update(); }
      
      /**
       * @brief Loads the horizontal scroll visibility property from a ds::value.
       * 
       * @param Prop The property value definition.
       * @return True on success, or an error string.
       */
      inline fun propScrollHorzVisible(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");
        
        setScrollHorzVisible(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME scrollHorzVisible, READ scrollHorzVisible, WRITE setScrollHorzVisible, PROP propScrollHorzVisible);


      /**
       * @brief Gets the bounding rectangle of the scrollable client area.
       * 
       * @return The bounding rectangle dimensions.
       */
      inline fun clientBound() const noexcept -> rect<i32> {return m_clientBound;}
      
      /**
       * @brief Gets the current scroll position within the client area.
       * 
       * @return The scroll position point.
       */
      inline fun clientPos() const noexcept -> poit<i32> {return m_clientPos;}


    public:
      inline fun findInput(poit<i32> Pos) const -> visual* override { return view::findInput(Pos -clientPos()); }

      fun draw_after(renderContext&) -> void override;
      
      /**
       * @brief Corrects the scroll position if it exceeds the bounds of the scrollable area.
       */
      fun fixOverScroll() -> void;

      /**
       * @brief Draws the vertical scrollbar.
       * 
       * @param Ctx The render context.
       */
      virtual fun draw_scrollVert(renderContext& Ctx) -> void;
      
      /**
       * @brief Draws the horizontal scrollbar.
       * 
       * @param Ctx The render context.
       */
      virtual fun draw_scrollHorz(renderContext& Ctx) -> void;      
      
      inline fun doTiling (renderContext& R) -> void override { view::doTiling(R); fixOverScroll(); }
      inline fun doResize () -> void override { fixOverScroll(); view::doResize(); }
      
      inline fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override { view::doMouseDown(Pos -m_clientPos, Button, State); }
      inline fun doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override { view::doMouseUp(Pos -m_clientPos, Button, State); }
      inline fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override { view::doMouseMove(Pos -m_clientPos, State); }

      fun doMouseScrollVert (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;
      fun doMouseScrollHorz (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;

      fun doTouchScrollVert (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;
      fun doTouchScrollHorz (poit<f32> Pos, f32 Delta, shiftStateSet State) -> void override;

    public:
      fun getProp(string_view Name) -> expected<std::any, bool> override;
      fun setProp(string_view Name, std::any Prop) -> expected<void, bool> override;
      fun loadProp(string_view Name, const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap) -> expected<bool, string> override;
  };

}
