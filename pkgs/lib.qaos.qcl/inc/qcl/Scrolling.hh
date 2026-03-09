/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <expected>
#include <string_view>

#include "Basis.h"

#include "qcl/Object.hh"
#include "qcl/Types.hh"
#include "qcl/Visual.hh"
#include "qcl/View.hh"
#include "DS.hh"

using namespace std;



namespace qcl
{

  struct qcl_object scrolling: view
  {
    public:
      inline scrolling() {}
      inline ~scrolling() override {}

    private:
      rect<i32> m_clientBound{};
      poit<i32> m_clientPos{};
      
      bool m_scrollVertVisible{true};
      bool m_scrollHorzVisible{true};
      
      bool ScrollableVert{false};
      bool ScrollableHorz{false};
      
      
    public:
      inline fun scrollVertVisible() const noexcept -> bool {return m_scrollVertVisible;}
      inline fun setScrollVertVisible(bool Val) noexcept -> void {m_scrollVertVisible = Val; update(); }
      inline fun propScrollVertVisible(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setScrollVertVisible(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME scrollVertVisible, READ scrollVertVisible, WRITE setScrollVertVisible, PROP propScrollVertVisible);
      
      
      inline fun scrollHorzVisible() const noexcept -> bool {return m_scrollHorzVisible;}
      inline fun setScrollHorzVisible(bool Val) noexcept -> void {m_scrollHorzVisible = Val; update(); }
      inline fun propScrollHorzVisible(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");
        
        setScrollHorzVisible(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME scrollHorzVisible, READ scrollHorzVisible, WRITE setScrollHorzVisible, PROP propScrollHorzVisible);


      inline fun clientBound() const noexcept -> rect<i32> {return m_clientBound;}
      inline fun clientPos() const noexcept -> poit<i32> {return m_clientPos;}


    public:
      inline fun findInput(poit<i32> Pos) const -> visual* override { return view::findInput(Pos -clientPos()); }

      fun draw_after(renderContext&) -> void override;
      fun fixOverScroll() -> void;

      virtual fun draw_scrollVert(renderContext&) -> void;
      virtual fun draw_scrollHorz(renderContext&) -> void;      
      
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
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
