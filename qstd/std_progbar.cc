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
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"



namespace qstd
{

  fun progbar::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);

    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), 14, 14);

    Surface->save();
    Surface->clipRRect(RRect, true);

    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);

    Surface->drawRect(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), Paint);
    
    
    f32 ProgressW = ((f32)size().W / m_max * m_value);
    if (ProgressW > 0) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

      Surface->drawRRect(SkRRect::MakeRectXY(SkRect::MakeXYWH(0, 0, ProgressW, (f32)size().H), 5,5), Paint);
    }
    
    Surface->restore(); // Restore main clip


    // Text Drawing
    Font.setSize(15);

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);

    f32 TX = ((size().W -TSize.width())/2);

    SkRRect ProgRRect;
    ProgRRect.setRectXY(SkRect::MakeLTRB(ProgressW,0, (f32)width(), (f32)height()), 14, 14);
    
    Surface->save();
    Surface->clipRRect(ProgRRect, true);
    
    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), TX, Pos, Font, Paint);

    Surface->restore();

    
    if (ProgressW > 0) {
      SkRRect ProgRRect;
      ProgRRect.setRectXY(SkRect::MakeXYWH(0,0, ProgressW, (f32)size().H), 14, 14);
      
      Surface->save();
      Surface->clipRRect(ProgRRect, true);

      if (!m_font)
        m_font = Context.fontType(0.7, 0, -0.4,0);
      
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::OnPrimaryContainer));
      Surface->drawString(m_text.c_str(), TX, Pos, SkFont(m_font, 15), Paint);
      
      Surface->restore();
    }
  }
  
}
