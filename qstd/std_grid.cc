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



namespace qstd
{

  fun grid::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());
    
    Paint.setColor4f(Monet.get(monetRole::Surface));
    Paint.setStrokeWidth(1);
    
    for (int i = 1; i < m_rows.size()+1; i++)
      Surface->drawLine(0, (f32)i*m_rowHeight, (f32)size().W, (f32)i*m_rowHeight, Paint);
    
    Font.setSize(14);


    // Satırları Çiz
    int SX = 0;
    for (int C = 0; C < m_cols.size(); C++)
    {
      grid_column &Col = m_cols[C];
 
      Paint.setColor4f(Monet.get(monetRole::OnPrimary));
      Paint.setStyle(SkPaint::kFill_Style);

      Surface->drawString(Col.Title.c_str(), (f32)SX+5, (f32)0+7 + 10, Font, Paint); 


      // Items
      int SY = m_rowHeight;
      for (auto &R: m_rows)
      {
        if (C >= R.size())
          break;
        
        
        switch (Col.Type)
        {
          case grid_col_type::gctText:
          {
            Paint.setColor4f(Monet.get(monetRole::OnPrimary));
            Paint.setStyle(SkPaint::kFill_Style);
            Surface->drawString(R[C].c_str(), (f32)SX+5, (f32)SY+7 + 10, Font, Paint);
            break;
          }

          case grid_col_type::gctCheckbox:
          {
            // Draw Checkbox Inline
            Paint.setStrokeWidth(2);
            
            f32 CX = (f32)SX+4;
            f32 CY = (f32)SY+4;

            // Box
            Paint.setColor4f(Monet.get(monetRole::Surface));
            Paint.setStyle(SkPaint::kStroke_Style);
            for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

            Surface->drawRoundRect(SkRect::MakeXYWH(CX+1, CY+1, 14, 14), 5, 5, Paint);

            if (R[C] == "1")
            {
              // Checked
              Paint.setColor4f(Monet.get(monetRole::Primary));
              Paint.setStyle(SkPaint::kFill_Style);
              for (auto &X: effects()) if (X->enabled()) X->modify(Paint);

              Surface->drawRoundRect(SkRect::MakeXYWH(CX+8-4, CY+8-4, 8, 8), 3, 3, Paint);
            }
            break;
          }

          default:;
        }

        SY += m_rowHeight;
      }
      SX += Col.Width;
    }

  }

  fun grid::doReset(renderContext& Ctx) -> void
  {
    m_lineColor.update();

    widget::doReset(Ctx);
  }

  fun grid::doClickEx(poit<f32> Pos) -> void
  {
    i32 nRow{-1}, nCol{-1};

    i32 Cac{};
    for (i32 i{}; i < m_cols.size(); i++)
      if ((Cac += m_cols[i].Width) > Pos.X)
      {
        nCol = i;
        break;
      }

    nRow = ((Pos.Y) /m_rowHeight) -1;

    if (nRow >= m_rows.size())
      return;

    doCellClick(nRow, nCol);    
  }

  fun grid::doCellClick(i32 Row, i32 Col) -> void
  {
    onCellClick(this, Row, Col);
  }

  fun grid::calcAutoSize(renderContext& Context) -> void
  {
    setPreferedSize({100,100});
  }
  
}
