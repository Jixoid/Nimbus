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

  struct qcl_object progbar: qcl::widget
  {
    public:
      inline progbar() { setSize({140, 24}); }


    private:
      string m_text{"progbar"};
      sk_sp<SkTypeface> m_font;

      u32 m_max{100}, m_value{30};


    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun max() const noexcept -> u32 { return m_max; }
      inline fun setMax(u32 val) -> void { m_max = val; update(visDirtyDraw); }
      inline fun propMax(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setMax(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME max, READ max, WRITE setMax, PROP propMax);

      
      inline fun value() const noexcept -> u32 { return m_value; }
      inline fun setValue(u32 val) -> void { m_value = val; update(visDirtyDraw); }
      inline fun propValue(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setValue(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME value, READ value, WRITE setValue, PROP propValue);
      

    public:
      fun draw(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
