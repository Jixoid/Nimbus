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
#include <vector>

#include "Basis.h"

#include "qcl/Object.hh"
#include "qcl/Types.hh"
#include "qcl/Visual.hh"
#include "qcl/Widget.hh"
#include "DS.hh"

using namespace std;



namespace qcl
{

  struct qcl_object view: widget
  {
    public:
      inline view() {}
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
      fun setHoverControl(visual*) -> void;
      fun setFocusControl(visual*) -> void;

    public:
      inline fun widgets() const noexcept -> const vector<widget*>& {return m_widgets;};
      inline fun objects() const noexcept -> const vector<object*>& {return m_objects;};
      fun childs() const noexcept -> const vector<object*>;

      inline fun hoverControl() const noexcept -> visual* {return m_hoverControl;}
      inline fun focusControl() const noexcept -> visual* {return m_focusControl;}


    public:
      fun addChild(object*) -> void;
      fun remChild(object*) -> void;
      [[nodiscard]] fun relChild(object*) -> object*;

      virtual fun findInput(poit<i32> Pos) const -> visual*;

      fun draw_after(renderContext&) -> void override;
      fun calcAutoSize(renderContext&) -> void override;
      
      fun handlerStateChanged(visual::stateFlagSet State) -> void override;
      
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
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
