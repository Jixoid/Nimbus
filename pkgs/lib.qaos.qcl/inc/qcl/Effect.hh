/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "DS.hh"
#include "qcl/Types.hh"
#include <expected>
#include <string_view>

using namespace std;



namespace qcl
{

  struct effect
  {
    public:
      inline effect() {}
      inline virtual ~effect() {}

    private:
      bool m_enabled{true};

    public:
      inline fun enabled() const noexcept -> bool {return m_enabled;}
      inline fun setEnabled(bool Val) noexcept -> void {m_enabled = Val;}

    public:
      virtual fun reset() -> void {};

      virtual fun modify(SkPaint&) -> void = 0;
      
      //virtual fun begin(renderContext& Ctx, const rect_f32 Rect) -> bool { return false; }
      //virtual fun end(renderContext& Ctx) -> void {}

      virtual fun loadProp(string_view Name, const ds::value& Prop) -> std::expected<bool, std::string>;
  };

}

