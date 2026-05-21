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

  fun choice::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    Paint.setStrokeWidth(2);
    Font.setSize(14);


    // Background
    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);

    Surface->drawRoundRect(SkRect::MakeLTRB(0, (f32)m_threshold, (f32)size().W, (f32)size().H -m_threshold), 18, 18, Paint);

    // Measure Loop if needed
    if (m_optTabs.size() != m_tabs.size()) {
      m_optTabs.clear();
      for(const auto& t : m_tabs) {
        SkRect b;
        Font.measureText(t.c_str(), t.size(), SkTextEncoding::kUTF8, &b);
        m_optTabs.push_back({b.width(), b.height()});
      }
    }


    i32 XOff = -10;

    for (u32 i = 0; i < m_tabs.size(); i++)
    {
      if (i >= m_optTabs.size()) break;

      auto &X = m_tabs[i];

      XOff += 16;


      SkFontMetrics metrics;
      Font.getMetrics(&metrics);

      if (m_tabID == i)
      {
        SkRect TabRect = SkRect::MakeLTRB(
          (f32)XOff -6, 
          0, 
          XOff +m_optTabs[i].W +6, 
          (f32)size().H
        );

        // Clip
        SkRRect TabRRect;
        TabRRect.setRectXY(TabRect, 18, 18);
        Surface->save();
        Surface->clipRRect(TabRRect, true);


        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
        Paint.setStyle(SkPaint::kFill_Style);
        for (auto &X: effects()) if (X->enabled()) X->modify(Paint);
        
        Surface->drawRect(TabRect, Paint);

        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);

        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnPrimaryContainer));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);

        Surface->restore();
      }

      el {
        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
        
        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnSurface));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);
      }

      XOff += m_optTabs[i].W;
    }

  }

  fun choice::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    i32 XOff = -10;

    i32 NHTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NHTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NHTabID != m_hTabID)
    {
      m_hTabID = NHTabID;

      update(visDirtyDraw);
    }


    widget::doMouseMove(Pos, State);
  }

  fun choice::doClickEx(poit<f32> Pos) -> void
  {
    i32 XOff = -10;

    i32 NTabID = -1;

    for (u32 i = 0; i < m_optTabs.size(); i++)
    {
      XOff += 16;

      if (
        Pos.X > XOff -3 &&
        Pos.X < XOff +m_optTabs[i].W +3
      )
      {
        NTabID = i;
        break;
      }

      XOff += m_optTabs[i].W;
    }


    if (NTabID != -1 && NTabID != m_tabID)
      doChanged(NTabID);


    widget::doClickEx(Pos);
  }

  fun choice::calcAutoSize(renderContext& Context) -> void
  {
    SkFont Font(Context.font());
    Font.setSize(14);
    
    auto XOff = -3;
    for (const auto& X: m_tabs) {
      SkRect b;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &b);
      XOff += b.width() + 16; 
    }
    
    setPreferedSize({XOff, 28});
  }
  
}
