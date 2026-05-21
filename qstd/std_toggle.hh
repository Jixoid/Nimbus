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

  struct qcl_object toggle: qcl::widget
  {
    public:
      inline toggle() { setAutoSize(true); }


    private:
      string m_text{"toggle"};

      bool m_checked{false};
  
    public:
      inline fun text() const -> string { return m_text; }
      inline fun setText(string val) -> void { m_text = val; update(visDirtyDraw); }
      inline fun propText(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setText(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME text, READ text, WRITE setText, PROP propText);

      
      inline fun checked() const noexcept -> bool { return m_checked; }
      inline fun setChecked(bool val) -> void { m_checked = val; update(visDirtyDraw); }
      inline fun propChecked(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isBool())
          return std::unexpected("was expected bool");
      
        setChecked(Prop.w_bool());
        return true;
      }
      qcl_prop(TYPE bool, NAME checked, READ checked, WRITE setChecked, PROP propChecked);


    public:
      qcl_func qev<void, bool /* Status */> onChanged;


    public:
      fun draw(renderContext&) -> void override;

      fun handlerStateChanged(qcl::visual::stateFlagSet State) -> void override;

      fun doClick() -> void override;
      virtual fun doChanged(bool Status) -> void;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
