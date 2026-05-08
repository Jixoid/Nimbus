/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "basis.hh"
#include "qcl/types.hh"
#include "qstd/std_layout_flow.hh"



namespace qstd
{

  fun layout_flow::doTiling(renderContext& Context) -> void
  {
    poit<i32> CPos{}, LPad{}; // R,B
    i32 LineHeight{}, LastMargin{};

    // Tiling
    for (auto &X: widgets())
    {
      if (!X->visible())
        continue;

      auto NSize = qcl::size<i32>(
        X->canAutoSizeHorz() ? X->preferedSize().W : X->size().W,
        X->canAutoSizeVert() ? X->preferedSize().H : X->size().H
      );

      NSize.W = max(X->minSize().W, NSize.W);
      NSize.H = max(X->minSize().H, NSize.H);

      if (X->maxSize().W != 0) NSize.W = min(X->maxSize().W, NSize.W);
      if (X->maxSize().H != 0) NSize.H = min(X->maxSize().H, NSize.H);

      
      i32 RequiredWidth = X->margins().left() +NSize.W +X->margins().right() +LPad.X;
      
      if (CPos.X +RequiredWidth > width() && CPos.X > 0)
      {
        CPos.X = 0;
        CPos.Y += LineHeight + LPad.Y;
        LineHeight = 0;
        LastMargin = 0;
      }

      i32 HorzMargin = (X->margins().left() > LastMargin) ? X->margins().left() : LastMargin;
      i32 TargetX = CPos.X + HorzMargin;
      i32 TargetY = CPos.Y + X->margins().top();

      i32 TargetH = NSize.H;
      if (X->anchors().bottom().active() && LineHeight > 0) 
        TargetH = max(NSize.H, LineHeight - X->margins().top() - X->margins().bottom());
      
      if (X->point().Y != TargetY || X->point().X != TargetX)
        X->setPoint({TargetX, TargetY});

      if (X->width() != NSize.W || X->height() != TargetH)
      {
        X->setSize({NSize.W, TargetH});
        
        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );
        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);


      LastMargin = X->margins().right();
      CPos.X = X->point().X + X->width();

      i32 TotalItemHeight = X->height() + X->margins().top() + X->margins().bottom();
      if (TotalItemHeight > LineHeight)
          LineHeight = TotalItemHeight;
    }


    // Others
    fixOverScroll();
  }
  
}
