/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "basis.hh"
#include "qcl/types.hh"
#include "qstd/standard.hh"



namespace qstd
{

  fun form::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());
    
    Paint.setColor4f(m_color);

    Surface->drawRect({0,0, (f32)size().W, (f32)size().H}, Paint);
  }

  fun form::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }
  
}
