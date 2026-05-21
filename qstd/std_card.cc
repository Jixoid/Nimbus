/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/types.hh"
#include "qstd/standard.hh"
#include "include/core/SkRRect.h"



namespace qstd
{

  fun card::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    f32 Radius = m_borderRadius;
    if (Radius < 0)
      Radius = (size().W > height() ? width():size().H);
    
    // Clip
    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0,0, (f32)width(), (f32)height()), Radius, Radius);

    Paint.setColor4f(m_color);
    Paint.setStyle(SkPaint::kFill_Style);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
    
    Surface->drawRRect(RRect, Paint);
  }

  fun card::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }
  
}
