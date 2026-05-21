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

  fun slider::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);

    // Common Calc
    f32 W = (f32)size().W;
    f32 H = (f32)size().H;
    f32 ValPos = ((W - 4.0f) / m_max * m_value) + 2.0f;

    if ((stateFlags() & (sfHover)) != 0)
    {
      // Filled (Left)
      SkVector RadiiLeft[4] = { {14,14}, {4,4}, {4,4}, {14,14} }; // TL, TR, BR, BL
      SkRRect RRectLeft;
      RRectLeft.setRectRadii(SkRect::MakeLTRB(1, 4, ValPos-7, H-4), RadiiLeft);
      
      Paint.setColor4f(Monet.get(monetRole::SecondaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(RRectLeft, Paint);


      // Handle (Middle)
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeLTRB(ValPos-3, 1, ValPos+3, H-1), 4, 4, Paint);


      // Empty Part (Right)-
      SkVector RadiiRight[4] = {{4,4}, {14,14}, {14,14}, {4,4}};
      SkRRect RRectRight;
      RRectRight.setRectRadii(SkRect::MakeLTRB(ValPos+7, 4, W-1, H-4), RadiiRight);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) *0.5);
      
      Surface->drawRRect(RRectRight, Paint);
    }
      
    else
    {
      // Filled (Left)
      SkVector RadiiLeft[4] = {{14,14}, {4,4}, {4,4}, {14,14}};
      SkRRect RRectLeft;
      RRectLeft.setRectRadii(SkRect::MakeLTRB(1, 4, ValPos-6, H-4), RadiiLeft);

      
      Paint.setColor4f(Monet.get(monetRole::SecondaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(RRectLeft, Paint);


      // Handle (Middle)
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeLTRB(ValPos-2, 1, ValPos+2, H-1), 4, 4, Paint);


      // Empty Part (Right)
      SkVector RadiiRight[4] = {{4,4}, {14,14}, {14,14}, {4,4}};
      SkRRect RRectRight;
      RRectRight.setRectRadii(SkRect::MakeLTRB(ValPos+6, 4, W-1, H-4), RadiiRight);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) *0.5);
      
      Surface->drawRRect(RRectRight, Paint);
    }
  }

  fun slider::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun slider::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (Button == shiftStates::ssLeft)
    {
      i32 Step = min<i32>(size().W, std::max<i32>(0, (i32)Pos.X));
      
      doChanged((f32)Step /(f32)size().W *m_max);
    }

    widget::doMouseDown(Pos, Button, State);
  }

  fun slider::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    if ((State & shiftStates::ssLeft) != 0)
    {
      i32 Step = min<i32>(size().W, std::max<i32>(0, Pos.X));

      doChanged((f32)Step /(f32)size().W *m_max);
    }

    widget::doMouseMove(Pos, State);
  }

  fun slider::doChanged(u32 nValue) -> void
  {
    m_value = nValue;

    onChanged(this, nValue);


    update(visDirtyDraw);
  }
  
}
