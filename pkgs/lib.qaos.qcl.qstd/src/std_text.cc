/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "basis.hh"
#include "qstd/standard.hh"
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"



namespace qstd
{

  fun text::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());
    SkFont Font(m_font ? m_font : Context.fontType(), m_fontSize);  

    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);

    poit<f32> Pos;

    // --- Yatay Hizalama (Horizontal Alignment) ---
    switch (m_alignHorz)
    {
      case taLeft:
        Pos.X = 2 - TSize.fLeft; 
        break;

      case taCenter:
        Pos.X = (width() - TSize.width()) / 2.0f - TSize.fLeft;
        break;

      case taRight:
        Pos.X = width() - TSize.width() - 2 - TSize.fLeft;
        break;
    }

    // --- Dikey Hizalama (Vertical Alignment) ---
    float textHeight = metrics.fDescent - metrics.fAscent;

    switch (m_alignVert)
    {
      case taTop:
        Pos.Y = -metrics.fAscent + 2; 
        break;

      case taCenter:
        Pos.Y = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
        break;

      case taBottom:
        Pos.Y = height() - metrics.fDescent - 2;
        break;
    }

    Paint.setColor4f(m_color);
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawString(m_text.c_str(), Pos.X, Pos.Y, Font, Paint);
  }

  fun text::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(m_font ? m_font : Context.fontType(), m_fontSize);
    
    SkFontMetrics metrics;
    Font.getMetrics(&metrics);

    SkRect TSize;
    Font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8, &TSize);
    
    setPreferedSize({(i32)TSize.width() +(i32)Context.dp(4), (i32)(metrics.fDescent - metrics.fAscent +Context.dp(4))});
  }

  fun text::doReset(renderContext& Ctx) -> void
  {
    m_color.update();
    m_font = Ctx.fontType(m_fontGrad, m_fontWght, m_fontWdth,0);

    widget::doReset(Ctx);
  }
  
}
