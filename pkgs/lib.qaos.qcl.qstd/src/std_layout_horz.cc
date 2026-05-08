/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "basis.hh"
#include "qcl/types.hh"
#include "qstd/std_layout_horz.hh"



namespace qstd
{

  fun layout_horz::doTiling(renderContext& Context) -> void
  {
    i32
      StartPos = 0,
      LastMargin = 0;

    // Tiling
    for (auto &X: widgets())
    {
      StartPos += (X->margins().left() > LastMargin) ? (X->margins().left()):(LastMargin);

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
        SPos = X->margins().top(),
        EPos = SPos +NSize.H;


      // End Fixed
      if (X->anchors().bottom().active()) EPos = height() -X->margins().bottom();
      if (!X->anchors().top().active() && X->anchors().bottom().active()) SPos = EPos -NSize.H;

      if (X->point().Y != SPos || X->point().X != StartPos)
        X->setPoint({StartPos, SPos});

      if (X->height() != (EPos -SPos) || X->width() != NSize.W)
      {
        X->setSize({NSize.W, EPos -SPos});

        X->setWidth( max(X->minSize().W, X->width()) );
        X->setHeight( max(X->minSize().H, X->height()) );

        if (X->maxSize().W != 0) X->setWidth( min(X->maxSize().W, X->width()) );
        if (X->maxSize().H != 0) X->setHeight( min(X->maxSize().H, X->height()) );
      }


      X->doPaint_prepare(Context);

      // Tiling
      LastMargin = X->margins().right();
      StartPos += X->width();
    }


    // Others
    fixOverScroll();
  }
  
}
