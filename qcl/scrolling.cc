/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <algorithm>
#include "qcl/basis.hh"
#include "include/core/SkCanvas.h"
#include "include/core/SkFont.h"
#include "include/core/SkPaint.h"
#include "qcl/scrolling.hh"
#include "qcl/object.hh"
#include "qcl/visual.hh"
#include "qcl/application.hh"
#include "qcl/widget.hh"
#include "qcl/types.hh"

#define ef else if
#define el else

using namespace std;



namespace qcl
{  
  
  inline fun __ifScrollableVert(scrolling* Self, f32 Delta) noexcept -> bool {
    return (Delta > 0) ? (Self->clientBound().Y1 +Self->clientPos().Y < 0):(Self->clientBound().Y2 +Self->clientPos().Y -Self->size().H > 0);
  }

  inline fun __ifScrollableHorz(scrolling* Self, f32 Delta) noexcept -> bool {
    return (Delta > 0) ? (Self->clientBound().X1 +Self->clientPos().X < 0):(Self->clientBound().X2 +Self->clientPos().X -Self->size().W > 0);
  }



  fun scrolling::draw_after(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();

    Surface->save();
    Surface->translate(clientPos().X, clientPos().Y);


    #ifdef _QCL_use_debug
      SkFont Font(Context.fontType(), Context.sp(8));
      SkPaint Paint(Context.paint());

      Paint.setStyle(SkPaint::kStroke_Style);
      Paint.setStrokeWidth(1);
      Paint.setColor4f(color::rgba(0,0.3,1));

      l_Bounds: {
        Surface->drawRect({
          (f32)clientBound().X1, (f32)clientBound().Y1,
          (f32)clientBound().X2, (f32)clientBound().Y2,
        }, Paint);
      }


      Paint.setColor4f(color::rgba(0.8,0,1.0) *0.25);
      Paint.setStyle(SkPaint::kFill_Style);

      l_Paddings: {
        Surface->save();
        Surface->clipRect(SkRect::MakeLTRB(
          paddings().left(),
          paddings().top(),
          std::max(width(), clientBound().X2) -paddings().right(),
          std::max(height(), clientBound().Y2) -paddings().bottom()
        ), SkClipOp::kDifference);

        Surface->drawRect(SkRect::MakeXYWH(0,0, width(), height()), Paint);

        Surface->restore();
      }
    #endif


    for (auto &X: widgets()) if (X->visible())
    {
      auto RealRect = rect<i32>::XYWH({X->point()}, X->size());

      if (!RealRect.is_intersects(rect<i32>::XYWH(clientPos() *(-1), size())))
        continue;
    
      
      Surface->save();

      Surface->translate(RealRect.X1, RealRect.Y1);
      Surface->clipRect({0,0, (f32)X->size().W, (f32)X->size().H});

      X->doPaint(Context);

      
      //if (X->Opacity == 1)
      //  Surface->Paint();
      //ef (X->Opacity == 0);
      //el
      //  Surface->PaintA(X->Opacity);
      
      
      #ifdef _QCL_use_debug
      Paint.setStyle(SkPaint::kStroke_Style);
      Paint.setStrokeWidth(1);
      Paint.setColor4f(color::rgba(1,0,0));

      l_Edge: {
        Surface->drawRect(rect<f32>::XYWH({}, X->size()), Paint);
      }

      Paint.setColor4f(color::rgba(0,0.3,1));
      l_Corner: {
        Surface->drawPoints(SkCanvas::kPolygon_PointMode, {{5,0}, {0,0}, {0,5}}, Paint);

        Surface->drawPoints(SkCanvas::kPolygon_PointMode, {{5, (f32)X->height()}, {0, (f32)X->height()}, {0, (f32)X->height() -5}}, Paint);

        Surface->drawPoints(SkCanvas::kPolygon_PointMode, {{(f32)X->width() -5, (f32)X->height()}, {(f32)X->width(), (f32)X->height()}, {(f32)X->width(), (f32)X->height() -5}}, Paint);

        Surface->drawPoints(SkCanvas::kPolygon_PointMode, {{(f32)X->width() -5, 0}, {(f32)X->width(), 0}, {(f32)X->width(), 5}}, Paint);
      }

      #endif

      Surface->restore();

      #ifdef _QCL_use_debug
      Paint.setStyle(SkPaint::kFill_Style);
      Paint.setColor4f(color::rgba(0,1,0) *0.25);

      l_Margins: {
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top(), -X->margins().left(), X->height()), Paint);
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top(), X->width(), -X->margins().top()), Paint);
        
        Surface->drawRect(SkRect::MakeXYWH(X->left()+X->width(), X->top(), X->margins().right(), X->height()), Paint);
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top()+X->height(), X->width(), X->margins().bottom()), Paint);
      }
      #endif
    }

    widget::draw_after(Context);
    
    Surface->restore();


    if (__ifScrollableVert(this, +1) || __ifScrollableVert(this, -1))
      draw_scrollVert(Context);
  
    if (__ifScrollableHorz(this, +1) || __ifScrollableHorz(this, -1))
      draw_scrollHorz(Context);
  }


  fun scrolling::draw_scrollVert(renderContext& Context) -> void
  {
    if (!m_scrollVertVisible)
      return;


    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(4);
    Paint.setColor4f(color::rgba(1,1,1, 0.08));
    
    Surface->drawLine(poit<f32>(width() -5,5), poit<f32>(width() -5, height() -5), Paint);

    f32
      MaxSize = (m_clientBound.Y2 -m_clientBound.Y1),
      Pos = -m_clientPos.Y,
      Height = height() -10;

    Paint.setColor4f(color::rgba(0.50, 0.35, 0.90));
    Surface->drawLine(poit<f32>(width() -5,5 +((Pos /MaxSize) *Height)), poit<f32>(width() -5, 5 +((Pos /MaxSize) *Height) +((Height /MaxSize) *Height)), Paint);
  }

  fun scrolling::draw_scrollHorz(renderContext& Context) -> void
  {
    if (!m_scrollHorzVisible)
      return;


    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    Paint.setStrokeWidth(4);
    Paint.setColor4f(color::rgba(1,1,1, 0.08));
    
    Surface->drawLine(poit<f32>(5, height() -5), poit<f32>(width() -5, height() -5), Paint);

    f32
      MaxSize = (m_clientBound.X2 -m_clientBound.X1),
      Pos = -m_clientPos.X,
      Width = width() -10;

    Paint.setColor4f(color::rgba(0.50, 0.35, 0.90));
    Surface->drawLine(poit<f32>(5 +((Pos /MaxSize) *Width), height() -5), poit<f32>(5 +((Pos /MaxSize) *Width) +((Width /MaxSize) *Width), height() -5), Paint);
  }

  fun scrolling::fixOverScroll() -> void
  {
    if (width() -m_clientPos.X > m_clientBound.X2) m_clientPos.X = width() -m_clientBound.X2;
    if (height()-m_clientPos.Y > m_clientBound.Y2) m_clientPos.Y = height() -m_clientBound.Y2;


    const auto [minX, minY] = tuple{max(0, m_clientBound.X1), max(0, m_clientBound.Y1)};

    m_clientPos = {
      min(minX, m_clientPos.X),
      min(minY, m_clientPos.Y),
    };
  }



  fun scrolling::doMouseScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);
    
    auto CanScrollRecursive = [&](auto&& Self, visual* Ctrl, poit<i32> RelPos) -> bool {
      auto V = dynamic_cast<scrolling*>(Ctrl);
      if (!V)
        return false;

      auto Inner = V->findInput(RelPos);
      if (Inner) {
        poit<i32> InnerRel = RelPos -V->m_clientPos -Inner->point();

        if (Self(Self, Inner, InnerRel))
          return true;
      }
      return __ifScrollableVert(V, Delta);
    };

    bool Delegate{false};
    if (Active) {
      poit<i32> RelPos = Pos -m_clientPos -Active->point();
      Delegate = CanScrollRecursive(CanScrollRecursive, Active, RelPos);
    }


    if (!Delegate && __ifScrollableVert(this, Delta)) {
      widget::doMouseScrollVert(Pos, Delta, State);


      if (__ifScrollableVert(this, Delta)) {
        m_clientPos.Y += Delta*6;

        if (height() -m_clientPos.Y > m_clientBound.Y2)
          m_clientPos.Y = height() -m_clientBound.Y2;

        if (m_clientPos.Y > m_clientBound.Y1)
          m_clientPos.Y = m_clientBound.Y1;

        dyeToRoot();
        CurrentApp->pushMessage(getRoot(), visual::messages::vmPaint);

        if (State == 0)
          setHoverControl(findInput(Pos));
      }

      return;
    }

    if (Active)
      Active->doMouseScrollVert(Pos -m_clientPos -Active->point(), Delta, State);
  }

  fun scrolling::doMouseScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);

    auto CanScrollRecursive = [&](auto&& Self, visual* Ctrl, poit<i32> RelPos) -> bool {
      auto V = dynamic_cast<scrolling*>(Ctrl);
      if (!V)
        return false;

      visual* Inner = V->findInput(RelPos);
      if (Inner)
      {
        poit<i32> InnerRel = RelPos -V->m_clientPos -Inner->point();

        if (Self(Self, Inner, InnerRel))
          return true;
      }
      return __ifScrollableHorz(V, Delta);
    };

    bool Delegate{false};
    if (Active) {
      poit<i32> RelPos = Pos -m_clientPos -Active->point();
      Delegate = CanScrollRecursive(CanScrollRecursive, Active, RelPos);
    }


    if (!Delegate && __ifScrollableHorz(this, Delta)) {
      widget::doMouseScrollHorz(Pos, Delta, State);


      if (__ifScrollableHorz(this, Delta)) {
        m_clientPos.X += Delta*6;

        if (width() -m_clientPos.X > m_clientBound.X2)
          m_clientPos.X = width() -m_clientBound.X2;

        if (m_clientPos.X > m_clientBound.X1)
          m_clientPos.X = m_clientBound.X1;

        dyeToRoot();
        CurrentApp->pushMessage(getRoot(), visual::messages::vmPaint);

        if (State == 0)
          setHoverControl(findInput(Pos));
      }

      return;
    }

    if (Active)
      Active->doMouseScrollHorz(Pos  -m_clientPos -Active->point(), Delta, State);
  }


  fun scrolling::doTouchScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);
    
    auto CanScrollRecursive = [&](auto&& Self, visual* Ctrl, poit<i32> RelPos) -> bool {
      auto V = dynamic_cast<scrolling*>(Ctrl);
      if (!V)
        return false;

      visual* Inner = V->findInput(RelPos);
      if (Inner) {
        poit<i32> InnerRel = RelPos -V->m_clientPos -Inner->point();

        if (Self(Self, Inner, InnerRel))
          return true;
      }
      return __ifScrollableVert(V, Delta);
    };

    bool Delegate{false};
    if (Active) {
      poit<i32> RelPos = Pos -m_clientPos -Active->point();
      Delegate = CanScrollRecursive(CanScrollRecursive, Active, RelPos);
    }


    if (!Delegate && __ifScrollableVert(this, Delta)) {
      widget::doTouchScrollVert(Pos, Delta, State);


      if (__ifScrollableVert(this, Delta)) {
        m_clientPos.Y += Delta*6;

        if (height() -m_clientPos.Y > m_clientBound.Y2)
          m_clientPos.Y = height() -m_clientBound.Y2;

        if (m_clientPos.Y > m_clientBound.Y1)
          m_clientPos.Y = m_clientBound.Y1;

        dyeToRoot();
        CurrentApp->pushMessage(getRoot(), visual::messages::vmPaint);

        if (State == 0)
          setHoverControl(findInput(Pos));
      }

      return;
    }

    if (Active)
      Active->doTouchScrollVert(Pos  -m_clientPos -Active->point(), Delta, State);
  }

  fun scrolling::doTouchScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    Pos -= m_clientPos;
    auto Active = findInput(Pos);

    auto CanScrollRecursive = [&](auto&& Self, visual* Ctrl, poit<i32> RelPos) -> bool {
      auto V = dynamic_cast<scrolling*>(Ctrl);
      if (!V)
        return false;

      visual* Inner = V->findInput(RelPos);
      if (Inner) {
        poit<i32> InnerRel = RelPos -V->m_clientPos -Inner->point();

        if (Self(Self, Inner, InnerRel))
          return true;
      }
      return __ifScrollableHorz(V, Delta);
    };

    bool Delegate{false};
    if (Active) {
      poit<i32> RelPos = Pos -m_clientPos -Active->point();
      Delegate = CanScrollRecursive(CanScrollRecursive, Active, RelPos);
    }


    if (!Delegate && __ifScrollableHorz(this, Delta)) {
      widget::doTouchScrollHorz(Pos, Delta, State);


      if (__ifScrollableHorz(this, Delta)) {
        m_clientPos.X += Delta*6;

        if (width() -m_clientPos.X > m_clientBound.X2)
          m_clientPos.X = width() -m_clientBound.X2;

        if (m_clientPos.X > m_clientBound.X1)
          m_clientPos.X = m_clientBound.X1;

        dyeToRoot();
        CurrentApp->pushMessage(getRoot(), visual::messages::vmPaint);

        if (State == 0)
          setHoverControl(findInput(Pos));
      }

      return;
    }

    if (Active)
      Active->doTouchScrollHorz(Pos -m_clientPos -Active->point(), Delta, State);
  }

}
