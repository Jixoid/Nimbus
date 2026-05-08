/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "ds/ds.hh"
#include "qcl/widget.hh"
#include "qstd/monet.hh"
#include "include/core/SkSurface.h"
#include "include/core/SkRefCnt.h"

using namespace qcl;



namespace qstd
{

  struct qcl_object icon: qcl::widget
  {
    private:
      sk_sp<SkSurface> m_icon{};

      dcolor m_theme{this, monetRole::OnSurface};

      string m_path = "";

      bool m_reRender{false};


    public:
      inline fun theme() -> dcolor& { return m_theme; }
      inline fun propTheme(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_theme.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME theme, PROP propTheme);


      inline fun path() const -> string { return m_path; }
      inline fun setPath(string val) -> void { m_path = val; m_reRender = true; update(visDirtyDraw); }
      inline fun propPath(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        setPath(Prop.w_string());
        return true;
      }
      qcl_prop(TYPE string, NAME path, READ path, WRITE setPath, PROP propPath);

    public:
      fun draw(renderContext&) -> void override;

      fun doPaint_prepare(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
