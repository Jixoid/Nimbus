/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/

#include "qstd/std_multiprogbar.hh"
#include "include/core/SkPaint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"

using namespace qcl;


namespace qstd
{

  fun multi_progbar::draw(renderContext& Ctx) -> void
  {
    auto Canvas = Ctx.canvas();


    f32 totalValue{};
    for (auto &X: m_values) totalValue += X.first;

    f32 totalGapWidth = (m_values.size() -1) *3;
    
    f32 usableWidth = (f32)width() - totalGapWidth;
    f32 Scl = (totalValue > 0) ? (usableWidth / totalValue) : 0;

    
    Canvas->save();
    Canvas->clipRRect(SkRRect::MakeRectXY(SkRect::MakeLTRB(0, 0, width()-1, height()-1), height()/2.0, height()/2.0), true);
    
    f32 Off{};
    for (auto &X: m_values) {
      SkPaint Paint(Ctx.paint());
      Paint.setStyle(SkPaint::kFill_Style);
      Paint.setColor4f(X.second);

      Canvas->drawRoundRect(
        SkRect::MakeLTRB((u32)Off, 0, (u32)(Off+(X.first*Scl)), height()-1),
        5,5,
        Paint
      );

      Off += (X.first*Scl) +3;
    }

    Canvas->restore();
  }

}
