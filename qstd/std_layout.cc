/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "include/core/SkRRect.h"
#include "qcl/types.hh"
#include "qstd/std_layout.hh"



namespace qstd
{

  fun layout::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setColor4f(m_color);
    for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
    
    Surface->drawRRect(SkRRect::MakeRectXY({0,0, (f32)size().W, (f32)size().H}, m_borderRadius, m_borderRadius), Paint);
  }

  fun layout::draw_scrollVert(renderContext& Context) -> void
  {
    if (!scrollVertVisible())
      return;

    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());


    auto rrect = SkRRect::MakeRectXY({0,0, (f32)width(), (f32)height()-1}, 14,14);
    Surface->save();
    Surface->clipRRect(rrect);


    Paint.setStrokeWidth(6);

    Paint.setColor4f(Monet.get(monetRole::SurfaceDim) *0.4);
    Surface->drawLine({(f32)size().W -3,0}, {(f32)size().W -3, (f32)size().H}, Paint);

    f32
      MaxSize = (clientBound().Y2-clientBound().Y1),
      Pos = -clientPos().Y,
      Height = height();

    Paint.setColor4f(Monet.get(monetRole::Primary));
    Surface->drawLine(
      {(f32)size().W -3, ((Pos /MaxSize) *Height)},
      {(f32)size().W -3, ((Pos /MaxSize) *Height) +((Height /MaxSize) *Height)},
      Paint
    );
    

    Surface->restore();
  }

  fun layout::draw_scrollHorz(renderContext& Context) -> void
  {
    if (!scrollHorzVisible())
      return;

    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());


    auto rrect = SkRRect::MakeRectXY({0,0, (f32)width(), (f32)height()-1}, 14,14);
    Surface->save();
    Surface->clipRRect(rrect);


    Paint.setStrokeWidth(6);

    Paint.setColor4f(Monet.get(monetRole::SurfaceDim) *0.4);
    Surface->drawLine({0, (f32)size().H -3}, {(f32)size().W, (f32)size().H -3}, Paint);

    f32
      MaxSize = (clientBound().X2-clientBound().X1),
      Pos = -clientPos().X,
      Width = width();

    Paint.setColor4f(Monet.get(monetRole::Primary));
    Surface->drawLine(
      {((Pos /MaxSize) *Width), (f32)size().H -3},
      {((Pos /MaxSize) *Width) +((Width /MaxSize) *Width), (f32)size().H -3},
      Paint
    );


    Surface->restore();
  }

  fun layout::doReset(renderContext& Ctx) -> void
  {
    m_color.update();

    view::doReset(Ctx);
  }
  
}
