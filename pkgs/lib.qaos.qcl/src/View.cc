/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "include/core/SkCanvas.h"
#include "include/core/SkFont.h"
#include "include/core/SkPaint.h"
#include "qcl/Object.hh"
#include "qcl/Visual.hh"
#include <stdexcept>
#define ef else if
#define el else

#include <vector>
#include <ranges>
#include <algorithm>

#include "Basis.h"

#include "qcl/Platform.hh"
#include "qcl/Widget.hh"
#include "qcl/Types.hh"
#include "qcl/View.hh"

using namespace std;



namespace qcl
{
  fun view::childs() const noexcept -> const vector<object*>
  {
    vector<object*> ret;

    ret.reserve(m_objects.size() + m_widgets.size());

    ret.append_range(m_objects);
    ret.append_range(m_widgets);

    return ret;
  }

  fun view::setHoverControl(visual* Ctrl) -> void
  {
    if (m_hoverControl == Ctrl)
      return;

    if (Ctrl)
      Ctrl->handlerStateChanged(
        visual::stateFlags::sfHover |
        (m_focusControl == Ctrl ? visual::stateFlags::sfFocus:0)
      );

    if (m_hoverControl != Nil)
      m_hoverControl->handlerStateChanged(
        (m_focusControl == m_hoverControl ? visual::stateFlags::sfFocus:0)
      ); 


    m_hoverControl = Ctrl;
  }

  fun view::setFocusControl(visual* Ctrl) -> void
  {
    if (m_focusControl == Ctrl)
      return;

    if (Ctrl)
      Ctrl->handlerStateChanged(
        visual::stateFlags::sfFocus |
        (m_hoverControl == Ctrl ? visual::stateFlags::sfHover:0)
      );

    if (m_focusControl != Nil)
      m_focusControl->handlerStateChanged(
        (m_hoverControl == m_focusControl ? visual::stateFlags::sfHover:0)
      ); 


    m_focusControl = Ctrl;
  }


  fun view::addChild(object* Ctrl) -> void
  {
    if (Ctrl->m_parent)
      throw runtime_error("view::addChild(object*), object already has a parent");


    const auto Childs = childs();
    auto it = std::find(Childs.begin(), Childs.end(), Ctrl);

    if (it != Childs.end())
      throw runtime_error("view::addChild(object*), object already has been acquied");
    

    Ctrl->m_parent = this;
    
    if (auto X = dynamic_cast<widget*>(Ctrl); X)
    {
      m_widgets.push_back(X);
      addFlag(visDirtyDraw | visDirtyAutoSize | vieDirtyTiling);
    }

    else {
      m_objects.push_back(Ctrl);
    }
  }

  fun view::remChild(object* Ctrl) -> void
  {
    delete relChild(Ctrl);
  }

  fun view::relChild(object* Ctrl) -> object*
  {
    if (auto X = dynamic_cast<widget*>(Ctrl); X)
    {
      auto it = std::find(m_widgets.begin(), m_widgets.end(), Ctrl);

      if (it == m_widgets.end())
        throw runtime_error("view::relChild(object*), this object not in child list");
      
      Ctrl->m_parent = Nil;

      if (m_hoverControl == Ctrl)
      {
        m_hoverControl->handlerStateChanged((m_focusControl == m_hoverControl ? visual::stateFlags::sfFocus:0));
        m_hoverControl = Nil;
      }

      if (m_focusControl == Ctrl)
      {
        m_focusControl->handlerStateChanged((m_hoverControl == m_focusControl ? visual::stateFlags::sfHover:0));
        m_focusControl = Nil;
      }

      object* Ret = *it;
      m_widgets.erase(it);
        
      doResize();

      return Ret;
    }

    else
    {
      auto it = std::find(m_objects.begin(), m_objects.end(), Ctrl);

      if (it == m_objects.end())
        throw runtime_error("view::relChild(object*), this object not in child list");
      
      Ctrl->m_parent = Nil;

      object* Ret = *it;
      m_objects.erase(it);
      return Ret;
    }
  }

  

  fun view::draw_after(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkFont Font(Context.font());
    SkPaint Paint(Context.paint());

    Font.setSize(8);


    #ifdef _QCL_use_debug
      Paint.setColor4f(color::rgba(0,0.3,1));
      Paint.setStrokeWidth(1);
      Paint.setStyle(SkPaint::kStroke_Style);

      l_Bounds: {
        Surface->drawRect({0,0,(f32)width(), (f32)height()}, Paint);
      }


      Paint.setColor4f(color::rgba(0.8,0,1.0).copy_mul_A(0.25));
      Paint.setStyle(SkPaint::kFill_Style);

      l_Paddings: {
        Surface->save();
        Surface->clipRect(SkRect::MakeLTRB(
          paddings().left(),
          paddings().top(),
          width() -paddings().right(),
          height() -paddings().bottom()
        ), SkClipOp::kDifference);

        Surface->drawRect(SkRect::MakeXYWH(0,0, width(), height()), Paint);

        Surface->restore();
      }
    #endif


    for (auto &X: m_widgets)
    {
      if (!X->m_visible)
        continue;

      auto RealRect = rect<i32>::XYWH({X->point()}, X->size());

      if (!RealRect.is_intersects(rect<i32>::XYWH({}, size())))
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
      Paint.setColor4f(color::rgba(0,1,0).copy_mul_A(0.25));

      l_Margins: {
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top(), -X->margins().left(), X->height()), Paint);
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top(), X->width(), -X->margins().top()), Paint);
        
        Surface->drawRect(SkRect::MakeXYWH(X->left()+X->width(), X->top(), X->margins().right(), X->height()), Paint);
        Surface->drawRect(SkRect::MakeXYWH(X->left(), X->top()+X->height(), X->width(), X->margins().bottom()), Paint);
      }
      #endif
    }

    widget::draw_after(Context);
  }

  fun view::findInput(poit<i32> Pos) const -> visual*
  {
    for (auto &X: views::reverse(m_widgets))
      if (
        (X->m_visible & X->m_enabled & !X->m_transparent) &&

        Pos.X >= X->m_poit.X &
        Pos.Y >= X->m_poit.Y &
        
        Pos.X <= X->m_endPoit.X &
        Pos.Y <= X->m_endPoit.Y
      )
        return X;

    return Nil;
  }



  fun view::f_tilingScanH(vector<widget*> &HList, widget *This) -> void
  {
    if (This->_TC_Visiting) {
      string Msg = "a circular depency is caught";
      platform::qcl_error(Msg.c_str()); 
    }


    This->_TC_Visiting = true;

    if (This->anchors().left().active() && This->anchors().left().control() != Nil && This->anchors().left().control() != this)
      f_tilingScanH(HList, This->anchors().left().control());

    if (This->anchors().right().active() && This->anchors().right().control() != Nil && This->anchors().right().control() != this)
      f_tilingScanH(HList, This->anchors().right().control());

    This->_TC_Visiting = false;


    if (auto fnd = std::find(HList.begin(), HList.end(), This); fnd == HList.end())
      HList.push_back(This);
  }

  fun view::f_tilingScanV(vector<widget*> &VList, widget *This) -> void
  {
    if (This->_TC_Visiting) {
      string Msg = "a circular depency is caught";
      platform::qcl_error(Msg.c_str()); 
    }


    This->_TC_Visiting = true;
    
    if (This->anchors().top().active() && This->anchors().top().control() != Nil && This->anchors().top().control() != this)
      f_tilingScanV(VList, This->anchors().top().control());

    if (This->anchors().bottom().active() && This->anchors().bottom().control() != Nil && This->anchors().bottom().control() != this)
      f_tilingScanV(VList, This->anchors().bottom().control());

    This->_TC_Visiting = false;


    if (auto fnd = std::find(VList.begin(), VList.end(), This); fnd == VList.end())
      VList.push_back(This);
  }



  fun view::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    if (!(State & visual::stateFlags::sfHover)) setHoverControl(Nil);
    if (!(State & visual::stateFlags::sfFocus)) setFocusControl(Nil);

    widget::handlerStateChanged(State);
  }

  fun view::calcAutoSize(renderContext&) -> void
  {
    qcl::size<i32> MaxBare{}, MaxPadd{};

    for (auto &X: m_widgets) if (X->m_visible) {

      MaxBare.W = std::max(MaxBare.W, X->endPoit().X);
      MaxBare.H = std::max(MaxBare.H, X->endPoit().Y);

      MaxPadd.W = std::max(MaxPadd.W, MaxBare.W +X->margins().right());
      MaxPadd.H = std::max(MaxPadd.H, MaxBare.H +X->margins().bottom());
    }

    setPreferedSize({
      std::max(MaxBare.W +paddings().right(),  MaxPadd.W), 
      std::max(MaxBare.H +paddings().bottom(), MaxPadd.H) 
    });
  }


  fun view::doReset(renderContext& Context) -> void
  {
    for (auto &X: m_widgets)
      X->doReset(Context);

    widget::doReset(Context);
  }

  fun view::doTiling(renderContext& Context) -> void
  {
    // Pass 1: Horz & Vert Scan
    vector<widget*> CList;

    for (auto &X: m_widgets) X->_TC_Visiting = X->_TC_Visited = false;
    for (auto &X: m_widgets) f_tilingScanH(CList, X);

    for (auto &X: m_widgets) X->_TC_Visiting = X->_TC_Visited = false;
    for (auto &X: m_widgets) f_tilingScanV(CList, X);

    
    // Pass 2: Horz & Vert Relocation
    for (auto &X: CList)
    {
      auto NSize = qcl::size<i32>(
        (X->m_autoSizeHorz ? X->m_preferedSize.W : X->m_size.W),
        (X->m_autoSizeVert ? X->m_preferedSize.H : X->m_size.H)
      );

      NSize.W = max(X->m_minSize.W, NSize.W);
      NSize.H = max(X->m_minSize.H, NSize.H);

      if (X->m_maxSize.W != 0) NSize.W = min(X->m_maxSize.W, NSize.W);
      if (X->m_maxSize.H != 0) NSize.H = min(X->m_maxSize.H, NSize.H);

      // Start Pos
      poit<i32> SPos = X->m_poit;

      if (X->m_anchors.left().active()) {
        const auto Ctrl = X->m_anchors.left().control();

        if (Ctrl == this) {
          if (X->m_anchors.left().side() == widgetAnchorSide::wasCen)
            SPos.X = m_paddings.left() + ((m_size.W - m_paddings.left() - m_paddings.right() - NSize.W) / 2);
          else
            SPos.X = m_paddings.left();
        }
        ef (Ctrl == Nil);
        el switch (X->m_anchors.left().side()) {
          case widgetAnchorSide::wasBeg: SPos.X = Ctrl->point().X; break;
          case widgetAnchorSide::wasEnd: SPos.X = (Ctrl->m_visible) ? (Ctrl->m_endPoit.X):(Ctrl->m_poit.X); break;
          case widgetAnchorSide::wasCen: SPos.X = Ctrl->point().X +((Ctrl->width()-NSize.W)/2); break;
        }

        if (Ctrl)
          SPos.X += X->m_margins.left();
      }

      if (X->m_anchors.top().active()) {
        const auto Ctrl = X->m_anchors.top().control();

        if (Ctrl == this) {
          if (X->m_anchors.top().side() == widgetAnchorSide::wasCen)
            SPos.Y = m_paddings.top() + ((m_size.H - m_paddings.top() - m_paddings.bottom() - NSize.H) / 2);
          else
            SPos.Y = m_paddings.top();
        }
        ef (Ctrl == Nil);
        el switch (X->m_anchors.top().side()) {
          case widgetAnchorSide::wasBeg: SPos.Y = Ctrl->m_poit.Y; break;
          case widgetAnchorSide::wasEnd: SPos.Y = (Ctrl->m_visible) ? (Ctrl->m_endPoit.Y):(Ctrl->m_poit.Y); break;
          case widgetAnchorSide::wasCen: SPos.Y = Ctrl->point().Y +((Ctrl->height()-NSize.H)/2); break;
        }

        if (Ctrl)
          SPos.Y += X->m_margins.top();
      }


      // End Pos
      poit<i32> EPos = SPos + NSize;


      if (X->m_anchors.right().active()) {
        const auto Ctrl = X->m_anchors.right().control();

        if (Ctrl == this) {
          if (X->m_anchors.right().side() == widgetAnchorSide::wasCen)
            EPos.X = m_size.W - m_paddings.right() - ((m_size.W - m_paddings.left() - m_paddings.right() - NSize.W) / 2);
          else
            EPos.X = m_size.W - m_paddings.right();
        }
        ef (Ctrl == Nil);
        el switch (X->m_anchors.right().side()) {
          case widgetAnchorSide::wasBeg: EPos.X = (Ctrl->m_visible) ? (Ctrl->m_poit.X):(Ctrl->m_endPoit.X); break;
          case widgetAnchorSide::wasEnd: EPos.X = Ctrl->m_endPoit.X; break;
          case widgetAnchorSide::wasCen: EPos.X = Ctrl->point().X +Ctrl->size().W -((Ctrl->width()-NSize.W)/2); break;
        }

        if (Ctrl)
          EPos.X -= X->m_margins.right();
      }

      if (X->m_anchors.bottom().active()) {
        const auto Ctrl = X->m_anchors.bottom().control();

        if (Ctrl == this) {
          if (X->m_anchors.bottom().side() == widgetAnchorSide::wasCen)
            EPos.Y = m_size.H - m_paddings.bottom() - ((m_size.H - m_paddings.top() - m_paddings.bottom() - NSize.H) / 2);
          else
            EPos.Y = m_size.H - m_paddings.bottom();
        }
        ef (Ctrl == Nil);
        el switch (X->m_anchors.bottom().side()) {
          case widgetAnchorSide::wasBeg: EPos.Y = (Ctrl->m_visible) ? (Ctrl->m_poit.Y):(Ctrl->m_endPoit.Y); break;
          case widgetAnchorSide::wasEnd: EPos.Y = Ctrl->m_endPoit.Y; break;
          case widgetAnchorSide::wasCen: EPos.Y = Ctrl->point().Y +Ctrl->size().H -((Ctrl->height()-NSize.H)/2); break;
        }

        if (Ctrl)
          EPos.Y -= X->m_margins.bottom();
      }


      // End Fixed
      if (!X->m_anchors.left().active() && X->m_anchors.right().active()) SPos.X = EPos.X -NSize.W;
      if (!X->m_anchors.top().active()  && X->m_anchors.bottom().active()) SPos.Y = EPos.Y -NSize.H;


      // Set Attrs
      if (X->point() != SPos)
        X->setPoint(SPos);

      if (X->width() != (EPos.X -SPos.X) || X->height() != (EPos.Y -SPos.Y)) {
        qcl::size<i32> Temp = (EPos-SPos);

        Temp.W = max(X->minSize().W, Temp.W);
        Temp.H = max(X->minSize().H, Temp.H);

        if (X->maxSize().W != 0) Temp.W = min(X->maxSize().W, Temp.W);
        if (X->maxSize().H != 0) Temp.H = min(X->maxSize().H, Temp.H);

        X->setSize(Temp);
      }

      X->doPaint_prepare(Context);
    }
  }
  
  fun view::doPaint_prepare(renderContext& Context) -> void
  {
    // Pass 1: Prepare & Calc Prefered Size
    for (auto &X: m_widgets)
    {
      if (!X->m_visible)
        continue;

      X->doPaint_prepare(Context);

      if ((X->m_autoSizeVert || X->m_autoSizeHorz) && X->hasFlagR(visDirtyAutoSize))
        X->calcAutoSize(Context);
    }


    // Pass 2: Tiling
    if (hasFlagR(vieDirtyTiling))
      doTiling(Context);


    // Pass 3: Final
    if (hasFlagR(visDirtyResize))
      doResize();

    if (hasFlagR(visDirtyRebound))
      m_endPoit = m_poit +m_size;
  }

  fun view::doPaint(renderContext& Context) -> void
  {
    for (auto &X: m_widgets)
    {
      if (!X->m_visible)
        continue;

      if (X->m_dirtyFlags & visDirtyDraw)
        X->doPaint(Context);
    }


    widget::doPaint(Context);
  }

  fun view::doResize() -> void
  {
    addFlag(vieDirtyTiling);

    widget::doResize();
  }

  fun view::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (State == 0)
      setHoverControl(findInput(Pos));

    if (Button == shiftStates::ssLeft)
      setFocusControl(m_hoverControl);


    if (!m_hoverControl)
      widget::doMouseDown(Pos, Button, State);
    else
      m_hoverControl->doMouseDown(Pos -m_hoverControl->m_poit, Button, State);
  }

  fun view::doMouseUp(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (!m_hoverControl)
      widget::doMouseUp(Pos, Button, State);
    else {
      m_hoverControl->doMouseUp(Pos -m_hoverControl->m_poit, Button, State);
      
      if (State == 0)
        setHoverControl(findInput(Pos));
    }
  }

  fun view::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    if (State == 0)
      setHoverControl(findInput(Pos));


    if (!m_hoverControl)
      widget::doMouseMove(Pos, State);
    else
      m_hoverControl->doMouseMove(Pos -m_hoverControl->m_poit, State);
  }

  fun view::doMouseScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);
    
    if (!Active)
      widget::doMouseScrollVert(Pos, Delta, State);
    else
      Active->doMouseScrollVert(Pos -Active->m_poit, Delta, State);
  }

  fun view::doMouseScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);

    if (!Active)
      widget::doMouseScrollHorz(Pos, Delta, State);
    else
      Active->doMouseScrollHorz(Pos -Active->m_poit, Delta, State);
  }


  fun view::doTouchScrollVert(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);
   
    if (!Active)
      widget::doTouchScrollVert(Pos, Delta, State);
    else
      Active->doTouchScrollVert(Pos -Active->m_poit, Delta, State);
  }

  fun view::doTouchScrollHorz(poit<f32> Pos, f32 Delta, shiftStateSet State) -> void
  {
    auto Active = findInput(Pos);
   
    if (!Active)
      widget::doTouchScrollHorz(Pos, Delta, State);
    else
      Active->doTouchScrollHorz(Pos -Active->m_poit, Delta, State);
  }

  fun view::doTouchPinch(poit<f32> Delta, f32 Scale, shiftStateSet State) -> void
  {
    if (!m_hoverControl)
      widget::doTouchPinch(Delta, Scale, State);
    else
      m_hoverControl->doTouchPinch(Delta, Scale, State);
  }

  fun view::doTouchRotate(poit<f32> Delta, f32 Rotate, shiftStateSet State) -> void
  {
    if (!m_hoverControl)
      widget::doTouchRotate(Delta, Rotate, State);
    else
      m_hoverControl->doTouchRotate(Delta, Rotate, State);
  }
  

  fun view::doKeyDown(u64 Key, u32 KeyCode, shiftStateSet State) -> void
  {
    if (m_focusControl) m_focusControl->doKeyDown(Key, KeyCode, State);
    ef (m_hoverControl) m_hoverControl->doKeyDown(Key, KeyCode, State);

    else widget::doKeyDown(Key, KeyCode, State);
  }

  fun view::doKeyUp(u64 Key, u32 KeyCode, shiftStateSet State) -> void
  {
    if (m_focusControl) m_focusControl->handlerKeyUp(Key, KeyCode, State);
    ef (m_hoverControl) m_hoverControl->handlerKeyUp(Key, KeyCode, State);

    else widget::doKeyUp(Key, KeyCode, State);
  }

}
