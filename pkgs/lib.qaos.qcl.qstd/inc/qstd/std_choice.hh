/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "ds/ds.hh"
#include "qstd/std_tabs.hh"

using namespace qcl;



namespace qstd
{

  struct qcl_object choice: qstd::tabs
  {
    private:
      u32 m_threshold{0};

    public:
      inline fun threshold() const noexcept -> u32 { return m_threshold; }
      inline fun setThreshold(u32 val) -> void { m_threshold = val; update(visDirtyDraw); }
      inline fun propThreshold(const ds::value& Prop) -> expected<bool, string> {
        if (!Prop.isInt())
          return std::unexpected("was expected int");
      
        setThreshold(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u32, NAME threshold, READ threshold, WRITE setThreshold, PROP propThreshold);

    public:
      fun draw(renderContext&) -> void override;

      fun doMouseMove(poit<f32> Pos, shiftStateSet State) -> void override;
      fun doClickEx(poit<f32> Pos) -> void override;

      fun calcAutoSize(renderContext&) -> void override;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
