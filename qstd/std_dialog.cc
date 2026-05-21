/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/types.hh"
#include "qstd/std_dialog.hh"
#include "include/core/SkMaskFilter.h"
#include "include/core/SkBlurTypes.h"



namespace qstd
{

  fun dialog_titleBar::setButtons() -> void
  {
    m_btnClose->setParent(this);
    m_btnClose->setButtonType(dialog_titleButtonType::dtbtClose);
    m_btnClose->anchors().top().set(true, this, widgetAnchorSide::wasBeg);
    m_btnClose->anchors().right().set(true, this, widgetAnchorSide::wasEnd);
    m_btnClose->anchors().bottom().set(true, this, widgetAnchorSide::wasEnd);
    m_btnClose->margins().set({0,5,5,5});
  }

  fun dialog::setTitleBar() -> void
  {
    m_titleBar->setParent(this);
    m_titleBar->anchors().top().set(true, this, widgetAnchorSide::wasBeg);
    m_titleBar->anchors().left().set(true, this, widgetAnchorSide::wasBeg);
    m_titleBar->anchors().right().set(true, this, widgetAnchorSide::wasEnd);
    m_titleBar->margins().set({12,12,12,0});
  }

  fun dialog::showModal() -> void
  {
    if (auto win = dynamic_cast<window*>(getRoot()))
    {
      setPoint({(win->width()-width())/2, (win->height()-height())/2});

      win->setOverlay(this, windowOverlayMode::womShadow);
    }
  }

  fun dialog::draw(renderContext& Context) -> void
  {
    constexpr auto borderRadius = 16;


    auto Surface = Context.canvas();
    auto Paint = SkPaint(Context.paint());

    rect<i32> PR = {12,12,12,12};
    {
      auto rrect = SkRRect::MakeRectXY(SkRect::MakeLTRB(PR.X1,PR.Y1, (f32)width()-PR.X2, (f32)height()-PR.Y2), borderRadius,borderRadius);

      SkPaint shadowPaint;
      shadowPaint.setColor(SK_ColorBLACK);
      shadowPaint.setAlphaf(0.5);
      shadowPaint.setAntiAlias(true);

      float blurSigma = 4.0;
      shadowPaint.setMaskFilter(SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, blurSigma));

      Surface->drawRRect(rrect, shadowPaint);
    }


    Paint.setColor4f(Monet.get(monetRole::SurfaceContainer));
    Surface->drawRRect(SkRRect::MakeRectXY(SkRect::MakeLTRB(PR.X1, PR.Y1, width()-PR.X2, height()-PR.Y2), borderRadius, borderRadius), Paint);
  }
  
}
