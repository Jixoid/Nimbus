/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <algorithm>
#include "qstd/monet.hh"
#include "qstd/std_interval.hh"
#include "include/core/SkPaint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"

using namespace qcl;



namespace qstd
{

  fun interval::draw(renderContext& Context) -> void
  {
    auto Surface = Context.canvas();
    SkPaint Paint(Context.paint());

    f32 W = (f32)size().W;
    f32 H = (f32)size().H;

    // Hover durumuna göre görsel parametreler
    bool isHover = (stateFlags() & (sfHover)) != 0;
    f32 handleHalfW = isHover ? 3.0f : 2.0f;
    f32 gap = isHover ? 7.0f : 6.0f;
    f32 trackPadding = 4.0f; // Üstten ve alttan bırakılan boşluk

    // Pozisyon Hesaplamaları (m_pos1 ve m_pos2)
    // m_pos1'in m_pos2'den küçük olduğunu varsayıyoruz (yoksa std::min/max ile sarmalanabilir)
    f32 usableW = W - 4.0f;
    f32 Pos1 = (usableW / m_max * m_pos1) + 2.0f;
    f32 Pos2 = (usableW / m_max * m_pos2) + 2.0f;

    // --- 1. SOL BOŞ KISIM (Start -> Pos1) ---
    SkVector RadiiLeft[4] = { {14,14}, {4,4}, {4,4}, {14,14} };
    SkRRect RRLeft;
    RRLeft.setRectRadii(SkRect::MakeLTRB(1, trackPadding, Pos1 - gap, H - trackPadding), RadiiLeft);
    
    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) * 0.5f);
    Paint.setStyle(SkPaint::kFill_Style);
    Surface->drawRRect(RRLeft, Paint);

    // --- 2. ORTA DOLU KISIM (Pos1 -> Pos2) ---
    // Her iki tarafı da düz kesim (veya az yuvarlak) olan dolgu alanı
    SkRect RectMiddle = SkRect::MakeLTRB(Pos1 + gap, trackPadding, Pos2 - gap, H - trackPadding);
    
    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::SecondaryContainer));
    for (auto &X : effects()) if (X->enabled()) X->modify(Paint);
    
    // Eğer mesafe gap'ten darsa çizme veya koruma ekle
    if (RectMiddle.width() > 0) {
      Surface->drawRoundRect(RectMiddle, 4,4, Paint);
    }

    // --- 3. SAĞ BOŞ KISIM (Pos2 -> End) ---
    SkVector RadiiRight[4] = { {4,4}, {14,14}, {14,14}, {4,4} };
    SkRRect RRRight;
    RRRight.setRectRadii(SkRect::MakeLTRB(Pos2 + gap, trackPadding, W - 1, H - trackPadding), RadiiRight);
    
    Paint = SkPaint(Context.paint());
    Paint.setColor4f(Monet.get(monetRole::SurfaceContainerHigh) * 0.5f);
    Surface->drawRRect(RRRight, Paint);

    // --- 4. HANDLE'LAR (Pos1 ve Pos2 üzerine) ---
    auto drawHandle = [&](f32 p) {
      Paint = SkPaint(Context.paint());
      Paint.setColor4f(Monet.get(monetRole::PrimaryContainer));
      for (auto &X : effects()) if (X->enabled()) X->modify(Paint);
      Surface->drawRoundRect(SkRect::MakeLTRB(p - handleHalfW, 1, p + handleHalfW, H - 1), 4, 4, Paint);
    };

    drawHandle(Pos1);
    drawHandle(Pos2);
  }

  fun interval::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun interval::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    if (Button == shiftStates::ssLeft) {
      f32 clickValue = ((f32)Pos.X / (f32)size().W) * m_max;
      clickValue = std::clamp<f32>(clickValue, 0, (f32)m_max);

      f32 dist1 = std::abs(clickValue -m_pos1);
      f32 dist2 = std::abs(clickValue -m_pos2);

      (dist1 <= dist2) ? doChanged(clickValue, m_pos2) : doChanged(m_pos1, clickValue);
    }

    widget::doMouseDown(Pos, Button, State);
  }

  fun interval::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    if (State == shiftStates::ssLeft) {
      f32 clickValue = ((f32)Pos.X / (f32)size().W) * m_max;
      clickValue = std::clamp<f32>(clickValue, 0, (f32)m_max);

      f32 dist1 = std::abs(clickValue -m_pos1);
      f32 dist2 = std::abs(clickValue -m_pos2);

      (dist1 <= dist2) ? doChanged(clickValue, m_pos2) : doChanged(m_pos1, clickValue);
    }

    widget::doMouseMove(Pos, State);
  }

  fun interval::doChanged(u32 nPos1, u32 nPos2) -> void
  {
    m_pos1 = nPos1;
    m_pos2 = nPos2;

    onChanged(this, nPos1, nPos2);

    update(visDirtyDraw);
  }

}
