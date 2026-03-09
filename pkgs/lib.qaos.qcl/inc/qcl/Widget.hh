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

#include <sys/time.h>

#include "DS.hh"

#include "qcl/Object.hh"
#include "qcl/Popup.hh"
#include "qcl/Types.hh"
#include "qcl/Visual.hh"

using namespace std;

#define ef else if
#define el else


namespace qcl
{
  class view;  // Forward
  

  class widget;

  enum widgetAnchorSide: u16 { wasBeg = 1, wasEnd = 2, wasCen = 3 };

  class widgetAnchor
  {
    friend class widget;

    private:
      widget* m_owner{};
      bool m_active{};
      widget* m_control{};
      widgetAnchorSide m_side = widgetAnchorSide::wasBeg;

    public:
      widgetAnchor(widget* nOwner): m_owner(nOwner) {}

    public:
      inline fun active() const noexcept -> const bool {return m_active;}
      inline fun control() const noexcept -> widget* {return m_control;}
      inline fun side() const noexcept -> const widgetAnchorSide {return m_side;}

      inline fun setActive(bool Val) noexcept -> void {m_active = Val; update();}
      inline fun setControl(widget* Val) noexcept -> void {m_control = Val; update();}
      inline fun setSide(widgetAnchorSide Val) noexcept -> void {m_side = Val; update();}

      inline fun set(bool Act, widget* Ctrl, widgetAnchorSide Side) noexcept -> void {m_active = Act; m_control = Ctrl; m_side = Side; update();}

    private:
      fun update() -> void;
  };

  class widgetAnchors
  {
    friend class widget;
    
    private:
      widgetAnchor m_left, m_top, m_righ, m_bot;

    public:
      widgetAnchors(widget* nOwner)
        : m_left(nOwner), m_top(nOwner)
        , m_righ(nOwner), m_bot(nOwner)
      {}

    public:
      inline fun top() noexcept -> widgetAnchor& {return m_top;}
      inline fun left() noexcept -> widgetAnchor& {return m_left;}
      inline fun right() noexcept -> widgetAnchor& {return m_righ;}
      inline fun bottom() noexcept -> widgetAnchor& {return m_bot;}
  };

  class widgetMargins
  {
    friend class widget;

    private:
      widget *m_owner{};
      i32 m_left{}, m_righ{}, m_top{}, m_bot{};

    public:
      widgetMargins(widget* nOwner): m_owner(nOwner) {}

    public:
      inline fun top() const noexcept -> const i32 {return m_top;}
      inline fun left() const noexcept -> const i32 {return m_left;}
      inline fun right() const noexcept -> const i32 {return m_righ;}
      inline fun bottom() const noexcept -> const i32 {return m_bot;}
      
      
      inline fun setTop(i32 Val) noexcept -> void {m_top = Val; update();}
      inline fun setLeft(i32 Val) noexcept -> void {m_left = Val; update();}
      inline fun setRight(i32 Val) noexcept -> void {m_righ = Val; update();}
      inline fun setBottom(i32 Val) noexcept -> void {m_bot = Val; update();}
      inline fun set(rect<i32> Val) noexcept -> void {m_top = Val.Y1, m_left = Val.X1; m_bot = Val.Y2; m_righ = Val.X2; update();}
  
    private:
      fun update() -> void;
  };


  struct qcl_object widget: visual
  {
    public:
      friend struct view;

    public:
      inline constexpr widget(): m_anchors(this), m_margins(this) {}
      inline constexpr ~widget() override {}


    private:
      widgetAnchors m_anchors;
      widgetMargins m_margins;
      
      contextMenu* m_contextMenu{};
      
      f32 m_opacity{1.0};

      bool m_visible{true};
      bool m_enabled{true};
      bool m_transparent{false};      


    public:
      inline fun visible() const noexcept -> const bool {return m_visible;}
      inline fun setVisible(bool Val) -> void { if (m_visible != Val) { m_visible = Val; update(visDirtyRebound); }}
      inline fun propVisible(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setVisible(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME visible, READ visible, WRITE setVisible, PROP propVisible);
      

      inline fun enabled() const noexcept -> const bool {return m_enabled;}
      inline fun setEnabled(bool Val) -> void { if (m_enabled != Val) { m_enabled = Val; update(visDirtyDraw); }}
      inline fun propEnabled(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setEnabled(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME enabled, READ enabled, WRITE setEnabled, PROP propEnabled);
      

      inline fun contextMenu() const noexcept -> contextMenu* {return m_contextMenu;}
      inline fun setContextMenu(qcl::contextMenu* Val) noexcept -> void {m_contextMenu = Val;}
      

      inline fun transparent() const noexcept -> const bool {return m_transparent;}
      inline fun setTransparent(bool Val) noexcept -> void {m_transparent = Val;}
      inline fun propTransparent(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool()) return std::unexpected("was expected bool");

        setTransparent(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME transparent, READ transparent, WRITE setTransparent, PROP propTransparent);


      inline fun opacity() const noexcept -> const f32 {return m_opacity;}
      inline fun setOpacity(f32 Val) noexcept -> void {m_opacity = Val;}
      inline fun propOpacity(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isFloat()) return std::unexpected("was expected float");

        setOpacity(Prop.w_float());
        return true;
      }
      qcl_prop(TYPE f32, NAME opacity, READ opacity, WRITE setOpacity, PROP propOpacity);
      
      
      inline fun anchors() noexcept -> widgetAnchors& {return m_anchors;}
      fun propAnchors(const ds::value& Prop) -> expected<bool, string>;
      qcl_prop(TYPE qcl::rect<i32>, NAME anchors, PROP propAnchors);
      
      
      fun propMargins(const ds::value& Prop) -> expected<bool, string>;
      inline fun margins() noexcept -> widgetMargins& {return m_margins;}
      qcl_prop(TYPE qcl::rect<i32>, NAME margins, PROP propMargins);

    public:
      inline fun canAutoSizeVert() const noexcept -> bool {return autoSizeVert() && !(m_anchors.m_top.active() && m_anchors.m_bot.active());}
      inline fun canAutoSizeHorz() const noexcept -> bool {return autoSizeHorz() && !(m_anchors.m_left.active() && m_anchors.m_righ.active());}
      
    public:
      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      
    protected:
      bool _TC_Visited{false};
      bool _TC_Visiting{false};

    public:
      fun getProp(string_view) -> expected<any, bool> override;
      fun setProp(string_view, any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
