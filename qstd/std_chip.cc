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



namespace qstd
{

  fun chip::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());
    
    Paint.setStrokeWidth(2);
    Paint.setColor4f(m_color);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

    Paint.setStyle(SkPaint::kStroke_Style);
    
    f32 Radius = m_borderRadius;
    if (Radius < 0)
      Radius = (size().W > height() ? width():size().H);
       
    Surface->drawRoundRect(SkRect::MakeLTRB(1,1, (f32)width()-1, (f32)height()-1), Radius, Radius, Paint);
  }

  fun chip::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }
  
}
