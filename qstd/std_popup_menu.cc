/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/application.hh"
#include "qcl/types.hh"
#include "qstd/standard.hh"
#include "include/core/SkFont.h"
#include "include/core/SkPaint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkMaskFilter.h"
#include "include/core/SkBlurTypes.h"



namespace qstd
{

  fun __popupMenu__propItems(const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap, popupMenu::popup_item* Item, string Scope) -> expected<bool, string>
  {
    if (!Prop.isArray()) return std::unexpected("was expected array, in "+Scope);

    for (int i{}; i < Prop.size(); i++) {
      auto X = Prop[i];

      if (X.isString()) {
        auto POP = new popupMenu::popup_item;
        POP->setParent(Item);
        Item->addItem(POP);
        
        POP->setName(X.w_string());
      }
      
      ef (X.isTuple()) {
        auto POP = new popupMenu::popup_item;
        Item->addItem(POP);
        POP->setParent(Item);
        
        if (auto Y = X[ds::etype::etStr]; !Y.isUnDef())
          POP->setName(Y.w_string());
        else
          return std::unexpected("was expected string in "+Scope+"["+to_string(i+1)+"]::string");


        if (auto Y = X[ds::etype::etArr]; !Y.isUnDef()) {
          if (auto Err = __popupMenu__propItems(Y, FuncMap, POP, Scope+"/"+POP->name()); !Err || !*Err)
            return Err;
        }


        if (auto Y = X[ds::etype::etStc]; !Y.isUnDef()) 
          for (int j{}; j < Y.size(); j++) {
            auto Q = Y.get_stc(j);

            if (Q.first == "onClick") {
              if (!Q.second.isString()) [[unlikely]]
              return std::unexpected("was expected string");
              
              auto Seed = FuncMap(Q.second.w_string());
              
              if (!Seed) [[unlikely]]
              return std::unexpected("no function in code");
              
              Seed->ToLoad(POP->onClick);
            }
            else
              return false;          
          }
      }

      else
        return std::unexpected("was expected string or tuple, in "+Scope+"["+to_string(i+1)+"]");
    }

    return true;
  }

  fun popupMenu::propItems(const ds::value& Prop, function<expected<qev_seed,bool>(string_view)> FuncMap) -> expected<bool, string>
  {
    return __popupMenu__propItems(Prop, FuncMap, &m_items, "/");
  }


  fun popupMenu::showPopup(poit<i32> Val) -> void
  {
    Val -= {PR.X1, PR.Y1};

    m_hSel = -1;

    m_nowItems = &m_items;


    if (auto Window = dynamic_cast<window*>(getRoot()))
    {
      setPoint(Val);
      Window->setOverlay(this);
    }
  }

  fun popupMenu::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Font.setSize(Context.sp(14));

    {
      auto rrect = SkRRect::MakeRectXY(SkRect::MakeXYWH(PR.X1*1.2,PR.Y1*1.2, (f32)width()-((PR.X1+PR.X2)*1.2), (f32)height()-((PR.Y1+PR.Y2)*1.2)), 13,13);

      SkPaint shadowPaint;
      shadowPaint.setColor(SK_ColorBLACK);
      shadowPaint.setAlphaf(0.5);
      shadowPaint.setAntiAlias(true);

      float blurSigma = 4.0;
      shadowPaint.setMaskFilter(SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, blurSigma));

      Surface->drawRRect(rrect, shadowPaint);
    }


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh));
    Paint.setStyle(SkPaint::kFill_Style);
    
    Surface->drawRoundRect(SkRect::MakeXYWH(PR.X1,PR.Y1, (f32)width()-PR.X1-PR.X2, (f32)height()-PR.Y1-PR.Y2), 13, 13, Paint);


    u32 s = PR.Y1;

    if (m_nowItems != &m_items)
    {
      SkRect TSize;
      string NameStr(m_nowItems->name());
      Font.measureText(NameStr.c_str(), NameStr.size(), SkTextEncoding::kUTF8, &TSize);

      
      Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
      Paint.setStyle(SkPaint::kFill_Style);
      SkVector radii[4] = {{13,13},{13,13}, {0,0},{0,0}};
      SkRRect RRect;
      RRect.setRectRadii(SkRect::MakeXYWH(PR.X1,PR.Y1, (f32)width()-PR.X1-PR.X2, (f32)PR.Y1+TSize.height()+16), radii);

      Surface->drawRRect(RRect, Paint);

      
      Paint.setColor4f(Monet.get(monetRole::OnSurfaceContainerHigh));
      Paint.setStyle(SkPaint::kStroke_Style);

      f32 x1 = PR.X1+ 17, y1 = (f32)s+11;
      f32 x2 = PR.X1+ 13, y2 = (f32)s+15;
      Surface->drawLine(x1, y1, x2, y2, Paint);
      
      f32 x3 = PR.X1 +17, y3 = (f32)s+19;
      Surface->drawLine(x2, y2, x3, y3, Paint);

      Paint.setStyle(SkPaint::kFill_Style);
      Surface->drawString(
        NameStr.c_str(), 
        ((f32)size().W -TSize.width())/2, 
        (s) +((30 -TSize.height())/2) - TSize.fTop, // Approximate baseline
        Font, 
        Paint
      );

      s += 30;
    }


    f32 itemHeight = Font.getSize()*1.8;
    f32 separatorHeight = 12.0;

    s += 2;

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      auto item = m_nowItems->items()[i];
      string ItemNameStr(item->name());

      if (ItemNameStr == "-")
      {
        Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
        Paint.setStyle(SkPaint::kStroke_Style);
        Paint.setStrokeWidth(1.0f);
        
        f32 lineY = s + (separatorHeight / 2.0f);
        Surface->drawLine(PR.X1, lineY, (f32)width()-PR.X2, lineY, Paint);

        s += separatorHeight;
        continue;
      }

      SkRect TSize;
      Font.measureText(ItemNameStr.c_str(), ItemNameStr.size(), SkTextEncoding::kUTF8, &TSize);
      
      SkRect rowRect = SkRect::MakeXYWH(PR.X1+2, s, (f32)width()-PR.X1-PR.X2-4, itemHeight);

      if (i == m_hSel)
      {
        Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
        Paint.setStyle(SkPaint::kFill_Style);
        for (auto &X : effects()) if (X->enabled()) X->modify(Paint);
        
        Surface->drawRRect(SkRRect::MakeRectXY(rowRect.makeInset(2, 2), 13, 13), Paint);
      }

      if (i == m_hSel)
        Paint = SkPaint(Monet.get(monetRole::OnPrimaryContainer));
      else
        Paint = SkPaint(Monet.get(monetRole::OnSurfaceContainerHigh));
      
      Paint.setAntiAlias(true);


      f32 textY = s + (itemHeight /2.0) -(TSize.fTop +TSize.fBottom) /2.0;

      Surface->drawString(ItemNameStr.c_str(), rowRect.fLeft +8, textY, Font, Paint);


      if (!item->items().empty())
      {
        Paint.setStyle(SkPaint::kStroke_Style);
        Paint.setStrokeWidth(1);
        
        f32 arrowX = (f32)width() -PR.X2 -13.0;
        f32 arrowY = s + (itemHeight / 2.0);
        f32 size = 4.0;

        Surface->drawLine({arrowX -size, arrowY -size}, {arrowX, arrowY}, Paint);
        Surface->drawLine({arrowX, arrowY}, {arrowX -size, arrowY +size}, Paint);
      }

      s += itemHeight; // Bir sonraki öğeye geç
    }
    s += PR.Y2 -2;


    if (s+4 != height())
    {
      pair<u32, popupMenu*> *Data = new pair<u32, popupMenu*>;

      Data->first = s+4;
      Data->second = this;


      CurrentApp->pushTask(
        [](u0 Data)
        {
          pair<u32, popupMenu*> *Pair = (pair<u32, popupMenu*>*)Data;

          Pair->second->setSize({Pair->second->width(), (i32)Pair->first});

          CurrentApp->pushMessage(Pair->second->getRoot(), messages::vmPaint);

          delete Pair;
        },
        (u0)Data
      );
    }
  }

  fun popupMenu::doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    Pos -= {(f32)PR.X1, (f32)PR.Y1};

    const i32 k = 14*1.8;
    u32 s = 3;
    i32 NHSel = 0;

    if (Pos.X < 0 || Pos.Y < 0 || Pos.X > width()-PR.X1-PR.X2 || Pos.Y > height()-PR.Y1-PR.Y1)
    {
      NHSel = -1;
      goto _l_Sec;
    }
    
    if (m_nowItems != &m_items)
    {
      s += 30;
      
      if (Pos.Y <= s)
      {
        NHSel = -2;
        goto _l_Sec;
      }
    }

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      if (m_nowItems->items()[i]->name() == "-")
        s += 12;
      else
        s += k;

      if (s >= Pos.Y)
        break;

      NHSel += 1;
    }


    _l_Sec:
    if (NHSel == -2)
    {
      if (m_nowItems->items()[0]->parent() == nil)
        return;

      m_nowItems = m_nowItems->parent();

      doMouseMove(Pos, 0);

      update(visDirtyDraw);
      return;
    }


    if (Pos.X < 0 || Pos.X > width())
      goto _l_Close;

    if (m_nowItems->items().size() <= NHSel)
      return;

    if (m_nowItems->items()[NHSel]->name() == "-")
      return;



    if (m_nowItems->items()[NHSel]->items().empty())
    {
      if (auto win = dynamic_cast<window*>(getRoot()))
      {
        win->setOverlay(nil);
      }
      
      m_nowItems->items()[NHSel]->onClick(this);
      
      update();
    }
    else
    {
      m_nowItems->items()[NHSel]->setParent(m_nowItems);
      m_nowItems = m_nowItems->items()[NHSel];

      doMouseMove(Pos, 0);

      update(visDirtyDraw);
    }


    return;

    _l_Close:
    if (auto win = dynamic_cast<window*>(getRoot()))
    {
      win->setOverlay(nil);
      update();
    }
  }

  fun popupMenu::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    Pos -= {(f32)PR.X1, (f32)PR.Y1};


    const i32 k = 14*1.8;
    u32 s = 3;
    i32 NHSel = 0;

    if (Pos.X < 0 || Pos.Y < 0 || Pos.X > width()-PR.X1-PR.X2 || Pos.Y > height()-PR.Y1-PR.Y1)
    {
      NHSel = -1;
      goto _l_Sec;
    }

    if (m_nowItems != &m_items)
    {
      s += 30;
      
      if (Pos.Y <= s)
      {
        NHSel = -2;
        goto _l_Sec;
      }
    }

    for (u32 i = 0; i < m_nowItems->items().size(); i++)
    {
      if (m_nowItems->items()[i]->name() == "-")
        s += 12;
      else
        s += k;

      if (s >= Pos.Y)
        break;

      NHSel += 1;
    }
    

    _l_Sec:
    if (NHSel != m_hSel)
    {
      m_hSel = NHSel;

      update(visDirtyDraw);
    }

    contextMenu::doMouseMove(Pos, State);
  }

  fun popupMenu::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    contextMenu::handlerStateChanged(State);

    if ((State & stateFlags::sfHover) == 0)
      m_hSel = -1;


    update(visDirtyDraw);
  }
  
}
