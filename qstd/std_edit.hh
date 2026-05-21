/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/ds/ds.hh"
#include "qcl/widget.hh"

using namespace qcl;



namespace qstd
{

  struct qcl_object edit: qcl::widget
  {
    public:
      inline edit() { setSize({160, 40}); }

    private:
      string m_text{}, m_title{};
      u16 m_borderRadius{6};

      sk_sp<SkTypeface> m_font{};
      float m_scrollX{};

      i0 m_pos{}, m_pos2{-1};

    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);
      

      inline fun title() const -> string { return m_title; }
      inline fun setTitle(string val) -> void { m_title = val; update(visDirtyDraw); }
      inline fun propTitle(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setTitle(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME title, READ title, WRITE setTitle, PROP propTitle);
      

      inline fun borderRadius() const noexcept -> u16 { return m_borderRadius; }
      inline fun setBorderRadius(u16 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> expected<bool, string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE u16, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);

    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void override;
      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      fun doKeyDown(u64 Key, u32 KeyCode, qcl::shiftStateSet State) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
