/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/types.hh"
#include "qstd/std_layout_vert.hh"



namespace qstd
{

  fun layout_vert::doTiling(renderContext& Context) -> void
  {
    i32
      StartPos{},
      LastMargin = paddings().top();

    // Tiling
    for (auto &X: widgets())
    {
      StartPos += (X->margins().top() > LastMargin) ? (X->margins().top()):(LastMargin);

      // End Pos
      auto NSize = qcl::size<i32>(
        X->canAutoSizeHorz() ? X->preferedSize().W : X->size().W,
        X->canAutoSizeVert() ? X->preferedSize().H : X->size().H
      );

      NSize.W = max(X->minSize().W, NSize.W);
      NSize.H = max(X->minSize().H, NSize.H);

      if (X->maxSize().W != 0) NSize.W = min(X->maxSize().W, NSize.W);
      if (X->maxSize().H != 0) NSize.H = min(X->maxSize().H, NSize.H);

      i32
        SPos = std::max(paddings().left(), X->margins().left()),
        EPos = std::min(SPos +NSize.W, width() -paddings().right());


      // End Fixed
      if (X->anchors().right().active()) EPos = width() -X->margins().right() -paddings().right();
      if (!X->anchors().left().active() && X->anchors().right().active()) SPos = EPos -NSize.W;

      if (X->point().X != SPos || X->point().Y != StartPos)
        X->setPoint(poit<i32>(SPos, StartPos));

      if (X->width() != (EPos -SPos) || X->height() != NSize.H) {
        X->setSize(qcl::size<i32>(EPos -SPos, NSize.H));

        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );

        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);

      // Tiling
      LastMargin = X->margins().bottom();
      StartPos += X->height();
    }


    // Others
    fixOverScroll();
  }
  
}
