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
#include "include/core/SkRRect.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontMetrics.h"



namespace qstd
{

  fun tabs::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(2);
    Font.setSize(14);


    // Background
    SkRRect RRect;
    RRect.setRectXY(SkRect::MakeXYWH(0, 0, (f32)size().W, (f32)size().H), 18, 18);
    Surface->save();
    Surface->clipRRect(RRect, true);


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawRect(SkRect::MakeXYWH(0,0, (f32)size().W, (f32)size().H), Paint); 
    
    // Ensure m_optTabs is up to date
    if (m_optTabs.size() != m_tabs.size()) {
      m_optTabs.clear();
      for(const auto& t : m_tabs) {
        SkRect b;
        Font.measureText(t.c_str(), t.size(), SkTextEncoding::kUTF8, &b);
        m_optTabs.push_back({b.width(), b.height()});
      }
    }


    u32 XOff = 0;

    for (u32 i = 0; i < m_tabs.size(); i++)
    {
      if (i >= m_optTabs.size()) break; // Safety
      
      auto &X = m_tabs[i];
      XOff += 16;

      SkFontMetrics metrics;
      Font.getMetrics(&metrics);


      if (m_tabID == i)
      {
        Surface->save();

        SkRect TabRect = SkRect::MakeLTRB(
          (f32)XOff -6, 
          ((f32)size().H/2) -(m_optTabs[i].H/2) -6, 
          XOff +m_optTabs[i].W +6, 
          ((f32)size().H/2) +(m_optTabs[i].H/2) +6
        );

        SkRRect TabRRect;
        TabRRect.setRectXY(TabRect, 18, 18);
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

        Surface->restore(); // popSets
      }

      el {
        f32 Pos = (height() / 2.0f) - ((metrics.fAscent + metrics.fDescent) / 2.0f);
        
        Paint = SkPaint(Context.paint());
        Paint.setColor4f(Monet.get(monetRole::OnSurface));
        Surface->drawString(X.c_str(), (f32)XOff, Pos, Font, Paint);
      }

      
      XOff += m_optTabs[i].W;
    }


    Surface->restore(); // clip_Reset (Main Clip)
  }

  fun tabs::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    m_hTabID = -1;
    update(visDirtyDraw);
  }

  fun tabs::recalcTabsSize(renderContext& Context) -> void
  {
    auto Font(Context.font());
    Font.setSize(14);
    
    
    m_optTabs.clear();
    m_optTabs.reserve(m_tabs.size());
    
    for (auto &X: m_tabs)
    {
      SkRect TSize;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &TSize);

      m_optTabs.push_back({TSize.width(), TSize.height()});
    }
  }

  fun tabs::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    u32 XOff = 0;

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

  fun tabs::doClickEx(poit<f32> Pos) -> void
  {
    u32 XOff = 0;

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

  fun tabs::doChanged(i32 nTabID) -> void
  {
    m_tabID = nTabID;

    onChanged(this, nTabID);


    update(visDirtyDraw);
  }

  fun tabs::calcAutoSize(renderContext& Context) -> void
  {
    recalcTabsSize(Context);

    SkFont Font(Context.font());
    Font.setSize(14);
    
    auto XOff = 16;
    for (const auto& X : m_tabs) {
      SkRect b;
      Font.measureText(X.c_str(), X.size(), SkTextEncoding::kUTF8, &b);
      XOff += b.width() + 16;
    }
    
    setPreferedSize({XOff, 32});
  }
  
}
