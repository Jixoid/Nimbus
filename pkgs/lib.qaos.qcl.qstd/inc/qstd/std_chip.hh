/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "ds/ds.hh"
#include "qcl/view.hh"
#include "qcl/widget.hh"
#include "qstd/monet.hh"

using namespace qcl;



namespace qstd
{

  struct qcl_object chip: qcl::view
  {
    private:
      dcolor m_color{this, monetRole::SurfaceBright};
      f32 m_borderRadius{-1};

    public:
      inline fun color() -> dcolor& { return m_color; }
      inline fun propColor(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isString()) return std::unexpected("was expected string");

        m_color.update(Prop.w_string());

        return true;
      }
      qcl_prop(TYPE color, NAME color, PROP propColor);


      inline fun borderRadius() const noexcept -> f32 { return m_borderRadius; }
      inline fun setBorderRadius(f32 val) -> void { m_borderRadius = val; update(visDirtyDraw); }
      inline fun propBorderRadius(const ds::value& Prop) -> std::expected<bool, std::string> {
        
        if (Prop.isFloat())
          setBorderRadius(Prop.w_float());
        ef (Prop.isInt())
          setBorderRadius(Prop.w_int());
        else
          return std::unexpected("was expected float or int");

        return true;
      }
      qcl_prop(TYPE f32, NAME borderRadius, READ borderRadius, WRITE setBorderRadius, PROP propBorderRadius);

    public:
      fun draw(renderContext&) -> void override;
      fun doReset(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
