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
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"



namespace qstd
{

  fun check::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    color Theme;

    if ((stateFlags() & (sfHover)) != 0)
      Theme = Monet.get(monetRole::PrimaryContainer);
    else
      Theme = Monet.get(monetRole::SurfaceOutline);
      
    
    Paint.setColor4f(Theme);
    Paint.setStrokeWidth(2);
    Paint.setStyle(SkPaint::kStroke_Style);

    Surface->drawRoundRect(SkRect::MakeXYWH(1,1, 20, 20), 7, 7, Paint);

    if (m_checked) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRoundRect(SkRect::MakeXYWH(11-5, 11-5, 10, 10), 3, 3, Paint);
    }


    Font.setSize(15);


    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::OnSurface));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), 27, Pos, Font, Paint);
  }

  fun check::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun check::doClick() -> void
  {
    widget::doClick();

    doChanged(!m_checked);
  }

  fun check::doChanged(bool Status) -> void
  {
    m_checked = Status;

    onChanged(this, Status);

    update(visDirtyDraw);
  }

  fun check::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());
    
    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);
    
    setPreferedSize({(i32)TSize.width() +30 +8, max<i32>(TSize.height() +8, 22)});
  }
  
}
