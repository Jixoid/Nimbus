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
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"



namespace qstd
{

  fun toggle::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    monetRole Theme  = (stateFlags() & (sfHover)) != 0 ? monetRole::PrimaryContainer : monetRole::SurfaceOutline;
    monetRole Theme2 = (stateFlags() & (sfHover)) != 0 ? monetRole::PrimaryContainer : monetRole::PrimaryContainer;


    Paint.setStrokeWidth(2);

    if (!m_checked) {
      Paint.setColor4f(Monet.get(Theme));
      Paint.setStyle(SkPaint::kStroke_Style);

      Surface->drawRoundRect(SkRect::MakeXYWH((f32)size().W -50, 1, 49, 24), 12, 12, Paint);
      
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(Theme2));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawCircle((f32)size().W -38, 13, 7, Paint); 
    }
    else {
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      Paint.setStyle(SkPaint::kStrokeAndFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeXYWH((f32)size().W -50, 1, 49, 24), 12, 12, Paint);

      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainerLow) *0.9);
      Surface->drawCircle((f32)size().W -13, 13, 9, Paint);
    }


    Font.setSize(15);

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    Paint = SkPaint(Context.paint());
    Paint.setStyle(SkPaint::kFill_Style);
    Paint.setColor4f(Monet.get(monetRole::OnSurface));
    Surface->drawString(m_text.c_str(), 0, Pos, Font, Paint);
  }

  fun toggle::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun toggle::doClick() -> void
  {
    widget::doClick();
    
    doChanged(!m_checked);
  }

  fun toggle::doChanged(bool Status) -> void
  {
    m_checked = Status;

    onChanged(this, Status);


    update(visDirtyDraw);
  }

  fun toggle::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
    
    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);

    setPreferedSize({(i32)TSize.width() +8 +70, max<i32>(Pos +8, 26)});
  }
  
}
