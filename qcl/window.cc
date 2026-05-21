/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "include/core/SkPaint.h"
#include "include/core/SkRect.h"
#include "qcl/types.hh"
#include "qcl/application.hh"
#include "qcl/widget.hh"
#include "qcl/view.hh"
#include "qcl/window.hh"
#include "qcl/platform.hh"

#define el else
#define ef else if

using namespace std;



namespace qcl
{
  window::window(): view(), m_ohid(platform::API->Window.new_())
  {
    CurrentApp->regWindow(this);

    setSize({800,600});
  }

  window::~window()
  {
    if (m_destroy_ohid)
    {
      CurrentApp->delWindow(this);

      platform::API->Window.dis_(m_ohid);
    }
  }
  

  fun window::show() -> void
  {
    platform::API->Window.show(ohid());
  }

  fun window::hide() -> void
  {
    platform::API->Window.hide(ohid());
  }


  fun window::setSize(qcl::size<i32> Val) -> void
  {
    platform::API->Window.setSize(ohid(), Val);
    //m_surface.setSize(Val.W, Val.H);

    widget::setSize(Val);
  }



  fun window::findInput(poit<i32> Pos) const -> visual*
  {
    if (m_overlay)
    {
      if (
        Pos.X >= m_overlay->point().X &&
        Pos.Y >= m_overlay->point().Y &&
        
        Pos.X <= m_overlay->endPoit().X &&
        Pos.Y <= m_overlay->endPoit().Y
      )
        return m_overlay;

      return nil;
    }

    el
      return view::findInput(Pos);
  }

  fun window::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    view::doMouseDown(Pos, Button, State);

    if (Button == shiftStates::ssLeft && hoverControl() == nil)
    {
      m_overlay = nil;

      CurrentApp->pushMessage(this, visual::messages::vmPaint);
    }
  }

  fun window::draw_after(renderContext& Context) -> void
  {
    view::draw_after(Context);
    
    if (m_overlay)
    {
      m_overlay->doPaint_prepare(Context);
      
      auto Surface = Context.canvas();

      if (m_overlayMode == womShadow)
      {
        SkPaint Paint(color::black() *0.5);
        Surface->drawRect(SkRect::MakeXYWH(0,0, width(),height()), Paint);
      }


      Surface->save();

      Surface->translate(m_overlay->left(), m_overlay->top());
      Surface->clipRect(rect<f32>::XYWH({}, m_overlay->size()));

      m_overlay->doPaint(Context);

      Surface->restore();
    }
  }



  fun window::handlerWindowStateChanged(windowStates State) -> void
  {
    m_windowState = State;

    doWindowStateChanged(State);
  }


  fun __DeleteSelf(u0 p1) -> void
  {
    delete (window*)(p1);
  }


  fun window::handlerWindowClose() -> void
  {
    if (doWindowClose())
      CurrentApp->pushTask(&__DeleteSelf, (u0)(this));
  }
  

  fun window::doWindowStateChanged(windowStates State) -> void
  {
    onWindowStateChanged(this, State);
  }

  fun window::doWindowClose() -> bool
  {
    return (onWindowClose ? onWindowClose(this) : true);
  }

  fun window::doCreate() -> void
  {
    onCreate(this);
  }

  fun window::doDestroy() -> void
  {
    onDestroy(this);
  }

}
