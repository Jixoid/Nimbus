/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/

#include "Basis.hh"

#include "include/core/SkRect.h"
#include "qcl/Types.hh"
#include "qstd/Standard.hh"


namespace
{

  fun utf8_backspace_at(string &s, u0 &pos) -> void
  {
    if (s.empty() || pos == 0)
      return;

    u0 i = pos - 1;
    
    // Continuation byte (10xxxxxx) yakaladıkça geri git
    while (i > 0 && ((s[i] & 0xC0) == 0x80))
      --i;

    s.erase(i, pos - i);
    pos = i;
  }


  fun utf8_length(string s) -> u32
  {
    u0 count = 0;
    for (u8 c: s)
      if ((c & 0xC0) != 0x80) // UTF-8 continuation bytes'i atla
        ++count;

    return count;
  }

  fun utf8_length(const char *s) -> u32
  {
    u0 count = 0;

    u0 i = 0;
    while (s[i] != 0)
    {
      if ((s[i] & 0xC0) != 0x80) // UTF-8 continuation bytes'i atla
        ++count;

      ++i;
    }


    return count;
  }

  fun utf8_char_length(char C) -> i8
  {
    if ((C & 0x80) == 0x00) return 1;  // 0xxxxxxx
    ef ((C & 0xE0) == 0xC0) return 2;  // 110xxxxx
    ef ((C & 0xF0) == 0xE0) return 3;  // 1110xxxx
    ef ((C & 0xF8) == 0xF0) return 4;  // 11110xxx
    
    el
      return -1; // geçersiz UTF-8 başlangıcı
  }

  fun utf8_char_length_from_end(const char* str, u0 len) -> i8
  {
    if (len == 0)
      return 0;


    i8 i = 1;
    while (true)
    {
      if (i > len) return i;

      if ((*(str -i) & 0xC0) != 0x80) // UTF-8 continuation bytes'i atla
        break;

      ++i;
    }

    return i;
  }

}

namespace qstd
{

  void edit::draw(renderContext& Context) {
    auto* canvas = Context.canvas();
    SkFont font(Context.font());
    SkPaint paint(Context.paint());

    // Theme
    SkColor4f themeColor;
    if (stateFlags() & sfFocus)
      themeColor = Monet.get(monetRole::PrimaryContainer);
    ef (stateFlags() & sfHover)
      themeColor = Monet.get(monetRole::SurfaceOutline);
    el
      themeColor = Monet.get(monetRole::PrimaryVariant) *0.4;


    // 2. Kenarlık ve Başlık (Label) Çizimi
    paint.setStrokeWidth(2.0f);

    if (!m_title.empty()) {
        font.setSize(12.0f);
        SkRect titleBounds;
        font.measureText(m_title.c_str(), m_title.size(), SkTextEncoding::kUTF8, &titleBounds);

        // Başlığı çiz (OnSurface rengiyle)
        paint.setStyle(SkPaint::kFill_Style);
        paint.setColor4f(Monet.get(monetRole::OnSurface));
        
        SkFontMetrics metrics;
        font.getMetrics(&metrics);
        // Başlık metnini dikeyde yukarı hizala
        float titleY = -metrics.fAscent - 2.0f; 
        canvas->drawString(m_title.c_str(), 14.0f, titleY, font, paint);

        // Kenarlığı çiz (Başlığın olduğu yeri keserek)
        paint.setStyle(SkPaint::kStroke_Style);
        paint.setColor4f(themeColor);

        canvas->save();
        // Başlığın geçtiği yeri "clip" ederek orayı boş bırakıyoruz (Material Design stili)
        SkRect titleClip = SkRect::MakeXYWH(10.0f, -5.0f, titleBounds.width() + 8.0f, 15.0f);
        canvas->clipRect(titleClip, SkClipOp::kDifference, true);
        
        canvas->drawRoundRect(SkRect::MakeLTRB(1, 5, width() - 1, height() - 1), 
                              m_borderRadius, m_borderRadius, paint);
        canvas->restore();
    } else {
        // Başlık yoksa düz kenarlık
        paint.setStyle(SkPaint::kStroke_Style);
        paint.setColor4f(themeColor);
        canvas->drawRoundRect(SkRect::MakeLTRB(1, 5, width() - 1, height() - 1), 
                              m_borderRadius, m_borderRadius, paint);
    }


    font.setSize(15.0f);

    float padding = 12.0f;
    float viewWidth = width() - (padding * 2);
    
    // İmlecin pozisyonunu ölç (Scroll hesaplamak için)
    float cursorOffset = font.measureText(m_text.c_str(), m_pos, SkTextEncoding::kUTF8);

    // --- 3. Scroll Hesaplama (Auto-Scroll) ---
    if (cursorOffset < m_scrollX) {
      m_scrollX = cursorOffset; // İmleç soldan çıktıysa
    } else if (cursorOffset > m_scrollX + viewWidth) {
      m_scrollX = cursorOffset - viewWidth; // İmleç sağdan çıktıysa
    }
    
    // Metin çok kısaysa scroll'u sıfırla
    float totalWidth = font.measureText(m_text.c_str(), m_text.size(), SkTextEncoding::kUTF8);
    if (totalWidth < viewWidth) m_scrollX = 0;

    // --- 4. Metin ve Seçim Çizimi (Clipping ile) ---
    canvas->save();
    // Metnin kutu dışına taşmaması için maske uygula
    canvas->clipRect(SkRect::MakeLTRB(padding-1, 0, width() - padding, height()), SkClipOp::kIntersect);
    // Koordinat sistemini scroll kadar kaydır
    canvas->translate(padding - m_scrollX, 0);

    SkFontMetrics textMetrics;
    font.getMetrics(&textMetrics);
    float textY = (height() / 2.0f) - ((textMetrics.fAscent + textMetrics.fDescent) / 2.0f);
    textY += (m_title.empty() ? 2.0f : 4.0f);

    // A. Seçim Alanı (Selection)
    if ((stateFlags() & sfFocus) && !(m_pos2 == -1 || m_pos2 == m_pos)) {
      int x1 = font.measureText(m_text.c_str(), m_pos, SkTextEncoding::kUTF8);
      int x2 = font.measureText(m_text.c_str(), m_pos2, SkTextEncoding::kUTF8);
      
      paint.setColor4f(Monet.get(monetRole::PrimaryFixed));
      paint.setStyle(SkPaint::kFill_Style);
      canvas->drawRoundRect(
        SkRect::MakeLTRB(
          std::min(x1, x2), textY + textMetrics.fAscent, 
          std::max(x1, x2), textY + textMetrics.fDescent
        ), 2.5, 2.5, paint
      );
    }

    // B. Ana Metin
    paint.setStyle(SkPaint::kFill_Style);
    paint.setColor4f(Monet.get(monetRole::OnSurface));
    canvas->drawString(m_text.c_str(), 0, textY, font, paint);

    // C. İmleç Çizgisi
    if ((stateFlags() & sfFocus) && (m_pos2 == -1 || m_pos2 == m_pos)) {
      paint.setColor4f(themeColor);
      paint.setStrokeWidth(1.5f);
      paint.setStyle(SkPaint::kStroke_Style);
      canvas->drawLine(
        (int)cursorOffset, textY + textMetrics.fAscent, 
        (int)cursorOffset, textY + textMetrics.fDescent,
        paint
      );
    }

    canvas->restore(); // Kaydırmayı ve maskeyi geri al


    if (!m_font) [[unlikely]]
      m_font = Context.fontType();
  }

  fun edit::handlerStateChanged(visual::stateFlagSet State) -> void
  {
    widget::handlerStateChanged(State);

    update(visDirtyDraw);
  }

  fun edit::doMouseDown(poit<f32> Pos, shiftStateSet Button, shiftStateSet State) -> void
  {
    m_pos2 = -1;

    SkFont font(m_font); // Context'ten fontu al
    font.setSize(15.0f); // Draw fonksiyonundaki ile aynı boyut olmalı

    float startX = 12.0f; // Metnin çizilmeye başladığı X ofseti
    float localX = Pos.X -startX +m_scrollX;

    // Eğer tıklama metnin solundaysa imleci başa al
    if (localX <= 0) {
        m_pos = 0;
        update();
        return;
    }

    size_t textLen = m_text.size();
    const char* ptr = m_text.c_str();
    
    u32 lastPos = 0;
    float lastWidth = 0;
    bool found = false;

    // 2. Metni karakter karakter gezerek genişlik kontrolü yap
    for (u32 i = 0; i < textLen; ) {
        // Mevcut UTF-8 karakterinin uzunluğunu al
        i8 charLen = utf8_char_length(ptr[i]);
        if (charLen <= 0) charLen = 1; // Hatalı karakter koruması

        u32 nextPos = i + charLen;
        
        // Bu karaktere kadar olan toplam genişliği ölç (Advance Width)
        float currentWidth = font.measureText(ptr, nextPos, SkTextEncoding::kUTF8);

        // Tıklanan nokta bu karakterin sınırları içinde mi?
        if (localX <= currentWidth) {
            // Karakterin orta noktasını bul (Görsel doğruluk için)
            float charMidPoint = lastWidth + (currentWidth - lastWidth) / 2.0f;
            
            if (localX < charMidPoint)
                m_pos = i;       // Karakterin soluna tıklandı
            else
                m_pos = nextPos; // Karakterin sağına tıklandı
            
            found = true;
            break;
        }

        lastWidth = currentWidth;
        lastPos = nextPos;
        i = nextPos;
    }

    // 3. Eğer döngü bittiyse ve bulunamadıysa metnin en sonuna tıklanmıştır
    if (!found) {
        m_pos = (u32)textLen;
    }

    // Fokus al ve ekranı tazele
    // setStateFlags(stateFlags() | sfFocus); 
    update();
  }

  fun edit::doMouseMove(poit<f32> Pos, shiftStateSet State) -> void
  {
    if (!(State & shiftStates::ssLeft))
      return;

    SkFont font(m_font); // Context'ten fontu al
    font.setSize(15.0f); // Draw fonksiyonundaki ile aynı boyut olmalı

    float startX = 12.0f; // Metnin çizilmeye başladığı X ofseti
    float localX = Pos.X -startX +m_scrollX;

    if (Pos.X -startX < 0)
      m_scrollX = std::max<f32>(0, m_scrollX +((Pos.X -startX) *0.4));
    ef (Pos.X > width() -startX) {
      //m_scrollX += (Pos.X -width() -startX) * 0.4f;

      //m_scrollX = std::min(m_scrollX, std::max(0.0f, totalTextWidth - (width() - (startX * 2))));
    }

    // Eğer tıklama metnin solundaysa imleci başa al
    if (localX <= 0) {
      m_pos2 = 0;
      update();
      return;
    }

    size_t textLen = m_text.size();
    const char* ptr = m_text.c_str();
    
    u32 lastPos = 0;
    float lastWidth = 0;
    bool found = false;

    // 2. Metni karakter karakter gezerek genişlik kontrolü yap
    for (u32 i = 0; i < textLen; ) {
        // Mevcut UTF-8 karakterinin uzunluğunu al
        i8 charLen = utf8_char_length(ptr[i]);
        if (charLen <= 0) charLen = 1; // Hatalı karakter koruması

        u32 nextPos = i + charLen;
        
        // Bu karaktere kadar olan toplam genişliği ölç (Advance Width)
        float currentWidth = font.measureText(ptr, nextPos, SkTextEncoding::kUTF8);

        // Tıklanan nokta bu karakterin sınırları içinde mi?
        if (localX <= currentWidth) {
            // Karakterin orta noktasını bul (Görsel doğruluk için)
            float charMidPoint = lastWidth + (currentWidth - lastWidth) / 2.0f;
            
            if (localX < charMidPoint)
                m_pos2 = i;       // Karakterin soluna tıklandı
            else
                m_pos2 = nextPos; // Karakterin sağına tıklandı
            
            found = true;
            break;
        }

        lastWidth = currentWidth;
        lastPos = nextPos;
        i = nextPos;
    }

    // 3. Eğer döngü bittiyse ve bulunamadıysa metnin en sonuna tıklanmıştır
    if (!found) {
        m_pos2 = (u32)textLen;
    }

    // Fokus al ve ekranı tazele
    // setStateFlags(stateFlags() | sfFocus); 
    update();
  }

  void edit::doKeyDown(u64 __Key, u32 KeyCode, shiftStateSet State)
  {
    char* KeyStr = (char*)&__Key;

    bool handled{true};
    switch (KeyCode) {
      case 111: // Home
        m_pos = 0;
        break;

      case 116: // End
        m_pos = (u32)m_text.size();
        break;

      case 113: // Left
        if (m_pos > 0)
          m_pos -= utf8_char_length_from_end(m_text.c_str() + m_pos, m_pos);
        break;

      case 114: // Right
        if (m_pos < m_text.size()) {
          i8 L = utf8_char_length(*(m_text.c_str() + m_pos));
          m_pos = min<u32>((u32)m_text.size(), m_pos + (L > 0 ? L : 1));
        }
        break;

      case 119: // Delete (Genelde Linux'ta 119 veya farklı olabilir, eklemek iyi olur)
        if (m_pos < m_text.size()) {
          i8 L = utf8_char_length(*(m_text.c_str() + m_pos));
          m_text.erase(m_pos, L);
        }
        break;

      default:
        handled = false;
        break;
    }

    if (__Key && handled) {
      update();
      return;
    }

    
    // Backspace (ASCII 0x08)
    if (KeyStr[0] == 0x08) {
      if (m_pos > 0) {
        // Önce silinecek karakterin uzunluğunu hesapla
        u32 len = utf8_char_length_from_end(m_text.c_str() + m_pos, m_pos);
        // İmleci geri çek
        m_pos -= len;
        // O konumdaki karakteri sil
        m_text.erase(m_pos, len);
      }
    }
    // Yazdırılabilir Karakter Girişi (Kontrol karakteri değilse)
    else if ((u8)KeyStr[0] >= 32 || (u8)KeyStr[0] > 127) { 
      // Not: 127'den büyükse muhtemelen çok baytlı UTF-8'in başlangıcıdır.
      string input(KeyStr);
      m_text.insert(m_pos, input);
      m_pos += (u32)input.size();
    }

    update();
  }
  
}
