/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file types.hh
 * @brief Foundational data types, colors, and geometries.
 */

#pragma once

#include "basis.hh"
#include "include/core/SkCanvas.h"
#include "include/core/SkColor.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkFontParameters.h"
#include "include/core/SkFourByteTag.h"
#include "include/core/SkPaint.h"
#include "include/core/SkTypeface.h"
#include "include/private/base/SkPoint_impl.h"
#include "include/private/base/SkSpan_impl.h"
#include <algorithm>
#include <array>
#include <concepts>
#include <numbers>
#include <type_traits>
#include <utility>

using namespace std;



namespace qcl::platform
{
  extern "C" [[gnu::noreturn]] fun qcl_error(string_view) -> void;
  extern "C" fun qcl_warning(string_view) -> void;
}


namespace qcl
{
  class window;


  /**
   * @brief Represents a RGBA color in floating point space [0, 1].
   */
  struct color
  {
  protected:
    f32 m_r{}, m_g{}, m_b{}, m_a{};

  public:
    /**
     * @brief Default constructor initializing to transparent.
     */
    inline constexpr color() = default;

  protected:
    /**
     * @brief Constructs from an RGBA array.
     */
    inline constexpr color(array<f32, 4> RGBA): m_r(RGBA[0]), m_g(RGBA[1]), m_b(RGBA[2]), m_a(RGBA[3]) {}


  public:
    /**
     * @brief Creates a color from RGBA components [0, 1].
     */
    [[nodiscard]] inline constexpr static fun rgba(f32 R, f32 G, f32 B, f32 A = 1.0) -> color
    {
      return color({R,G,B,A});
    }

    /**
     * @brief Creates a color from a 32-bit integer (AARRGGBB format).
     */
    [[nodiscard]] inline constexpr static fun rgba(u32 val) -> color
    {
      return color({
        ((val >> 16) & 0xFF) / 255.0f,
        ((val >> 8) & 0xFF) / 255.0f,
        (val & 0xFF) / 255.0f,
        ((val >> 24) & 0xFF) / 255.0f
      });
    }

    /**
     * @brief Creates a color from HSL values.
     * 
     * @param Hue Hue in degrees [0, 360).
     * @param Saturation Saturation [0, 1].
     * @param Light Lightness [0, 1].
     */
    [[nodiscard]] inline constexpr static fun hsl(f64 Hue, f64 Saturation, f64 Light) -> color
    {
      auto hue2rgb = [](f64 p, f64 q, f64 t) -> f32 {
        if (t < 0.0) t += 1.0;
        if (t > 1.0) t -= 1.0;
        if (t < 1.0 /6.0) return p +(q -p) *6.0 *t;
        if (t < 1.0 /2.0) return q;
        if (t < 2.0 /3.0) return p +(q -p) *(2.0 /3.0 -t) *6.0;
        
        return p;
      };


      if (Saturation == 0.0)
        return color({(f32)Light, (f32)Light, (f32)Light, 1.0});
      
      else {
        f64 h = Hue /360.0; // normalize hue to [0,1)
        f64 q = (Light < 0.5) ? (Light *(1.0 +Saturation)) : (Light +Saturation -Light *Saturation);
        f64 p = 2.0 * Light -q;

        return color({hue2rgb(p, q, h +1.0 /3.0), hue2rgb(p, q, h), hue2rgb(p, q, h -1.0 /3.0), 1.0});
      }
    }

    /**
     * @brief Creates a color from OKLCH values.
     * 
     * @param Lightness OKLCH lightness [0, 1].
     * @param Chroma OKLCH chroma [0, ~0.4].
     * @param Hue OKLCH hue angle [0, 360].
     */
    [[nodiscard]] inline constexpr static fun oklch(f64 Lightness, f64 Chroma, f64 Hue) -> color
    {
      // OKLCH -> OKLAB
      f64 h_rad = Hue *(std::numbers::pi / 180.0);
      f64 L = Lightness;
      f64 a = Chroma *std::cos(h_rad);
      f64 b = Chroma *std::sin(h_rad);

      // OKLAB -> Linear sRGB
      f64 l_ = L +(0.3963377774 *a) +(0.2158037573 *b);
      f64 m_ = L -(0.1055613458 *a) -(0.0638541728 *b);
      f64 s_ = L -(0.0894841775 *a) -(1.2914855480 *b);

      f64 l = l_ *l_ *l_;
      f64 m = m_ *m_ *m_;
      f64 s = s_ *s_ *s_;

      f64 r_lin = (+4.0767416621 *l) -(3.3077115913 *m) +(0.2309699292 *s);
      f64 g_lin = (-1.2684380046 *l) +(2.6097574011 *m) -(0.3413193965 *s);
      f64 b_lin = (-0.0041960863 *l) -(0.7034186147 *m) +(1.7076147010 *s);

      // Linear sRGB -> sRGB (Gamma Correction)
      auto gamma_correct = [](f64 c) -> f32 {
        if (c <= 0.0031308)
          return 12.92 *c;
        
        return 1.055 *std::pow(c, 1.0 / 2.4) -0.055;
      };

      auto clamp01 = [](f32 v) -> f32 {
        return std::clamp(v, 0.0f, 1.0f);
      };

      return color({
        clamp01(gamma_correct(r_lin)),
        clamp01(gamma_correct(g_lin)),
        clamp01(gamma_correct(b_lin)),
        1.0f
      });
    }


  public:
    /**
     * @brief Linearly interpolates between two colors.
     * 
     * @param Start Beginning color.
     * @param End Ending color.
     * @param t Interpolation factor [0.0, 1.0].
     */
    [[nodiscard]] inline constexpr static fun lerp(const color& Start, const color& End, f32 t) -> color
    {
      t = std::clamp(t, 0.0f, 1.0f);
      return color({
        Start.m_r +(End.m_r -Start.m_r) *t,
        Start.m_g +(End.m_g -Start.m_g) *t,
        Start.m_b +(End.m_b -Start.m_b) *t,
        Start.m_a +(End.m_a -Start.m_a) *t
      });
    }

    /**
     * @brief Calculates relative luminance of the color.
     * @return Luminance value [0.0, 1.0].
     */
    [[nodiscard]] inline constexpr fun luminance() const noexcept -> f32
    {
      return (0.2126f * m_r) + (0.7151f * m_g) + (0.0722f * m_b);
    }
    

  public:
    [[nodiscard]] inline constexpr static fun white() -> color { return color({1,1,1,1}); }
    [[nodiscard]] inline constexpr static fun black() -> color { return color({0,0,0,1}); }
    [[nodiscard]] inline constexpr static fun none() -> color  { return color({0,0,0,0}); }


  public:
    /**
     * @brief Returns a generic vector of RGBA components.
     */
    [[nodiscard]] inline constexpr fun to_rgba_vec() const -> vec<f32, 4>   { return {m_r,m_g,m_b,m_a}; }
    
    /**
     * @brief Returns a generic array of RGBA components.
     */
    [[nodiscard]] inline constexpr fun to_rgba_arr() const -> array<f32, 4> { return {m_r,m_g,m_b,m_a}; }
    
    /**
     * @brief Returns the color as a packed 32-bit ARGB integer.
     */
    [[nodiscard]] inline constexpr fun to_rgba_u32() const noexcept -> u32
    {
      return 
        ((u32)(m_a *255.0f) << 24) |
        ((u32)(m_r *255.0f) << 16) |
        ((u32)(m_g *255.0f) << 8)  |
        ((u32)(m_b *255.0f));
    }
    
    /**
     * @brief Returns RGBA components.
     */
    struct rgba_t { f32 R,G,B,A; };
    [[nodiscard]] inline constexpr fun to_rgba() const -> rgba_t {
      return {m_r,m_g,m_b,m_a};
    }
    

    /**
     * @brief Converts color to OKLCH.
     */
    struct oklch_t { f64 L,C,H; };
    [[nodiscard]] inline constexpr fun to_oklch() const -> oklch_t
    {
      // sRGB -> Linear sRGB (Gamma Expansion)
      auto to_linear = [](f32 c) -> f64 {
        if (c <= 0.04045f)
          return c / 12.92;
        return std::pow((c + 0.055) / 1.055, 2.4);
      };

      f64 r_lin = to_linear(m_r);
      f64 g_lin = to_linear(m_g);
      f64 b_lin = to_linear(m_b);

      // Linear sRGB -> LMS
      f64 l = (0.4122214708 *r_lin) +(0.5363325363 *g_lin) +(0.0514459929 *b_lin);
      f64 m = (0.2119034982 *r_lin) +(0.6806995451 *g_lin) +(0.1073969566 *b_lin);
      f64 s = (0.0883024619 *r_lin) +(0.2817188976 *g_lin) +(0.6299787005 *b_lin);

      // 3. LMS -> Non-linear LMS (Cube Root)
      f64 l_ = std::pow(l, 1.0 / 3.0);
      f64 m_ = std::pow(m, 1.0 / 3.0);
      f64 s_ = std::pow(s, 1.0 / 3.0);

      // 4. Non-linear LMS -> OKLAB (L, a, b)
      f64 L = (0.2104542553 *l_) +(0.7936177850 *m_) -(0.0040720468 *s_);
      f64 a = (1.9779984951 *l_) -(2.4285922050 *m_) +(0.4505937099 *s_);
      f64 b = (0.0259040371 *l_) +(0.7827717662 *m_) -(0.8086757660 *s_);

      // 5. OKLAB -> OKLCH (L, C, h)
      f64 C = std::sqrt(a * a + b * b);
      f64 h = std::atan2(b, a) * (180.0 / std::numbers::pi);

      if (h < 0.0) h += 360.0;

      return {L, C, h};
    }


    /**
     * @brief Converts color to HSL.
     */
    struct hsl_t { f64 H,S,L; };
    [[nodiscard]] inline constexpr fun to_hsl() const -> hsl_t
    {
      f64 r = (f64)m_r;
      f64 g = (f64)m_g;
      f64 b = (f64)m_b;

      f64 max = std::max({r, g, b});
      f64 min = std::min({r, g, b});
      f64 delta = (max - min);

      f64 h = 0.0;
      f64 s = 0.0;
      f64 l = (max + min) / 2.0;

      if (delta != 0.0) {
        // Saturation calculation
        s = (l > 0.5) ? (delta / (2.0 - max - min)) : (delta / (max + min));

        // Hue calculation
        if (max == r)
          h = (g - b) / delta + (g < b ? 6.0 : 0.0);
        else if (max == g)
          h = (b - r) / delta + 2.0;
        else
          h = (r - g) / delta + 4.0;

        h *= 60.0;
      }

      return { h, s, l };
    }


  public:
    inline constexpr fun operator==(const color &It) const noexcept -> bool {
      return (m_a == It.m_a) && (m_r == It.m_r) && (m_g == It.m_g) && (m_b == It.m_b);
    }

    /**
     * @brief Multiplies alpha by a factor.
     */
    inline constexpr fun operator*(f32 Alpha) const -> color
    {
      color Ret(*this);
      Ret.m_a *= Alpha;

      return Ret;
    }

  public:
    /**
     * @brief Implicit conversion to Skia color.
     */
    inline constexpr operator SkColor4f() const { return {m_r,m_g,m_b,m_a}; }
  };



  /**
   * @brief Holds state parameters required during a render cycle.
   */
  struct renderContext
  {
    private:
      SkCanvas* m_canvas{};
      SkPaint& m_paint;
      SkFont& m_font;
      sk_sp<SkTypeface> m_fontType{};
      sk_sp<SkFontMgr> m_fontMgr{};
      
      f32 m_scaleFactor{};
      f32 m_fontFactor{};
      u64 m_currentTime{};


    public:
      /**
       * @brief The Skia canvas for drawing operations.
       */
      inline fun canvas() const noexcept -> SkCanvas*     { return m_canvas; }
      
      /**
       * @brief The current active SkPaint object.
       */
      inline fun paint() const noexcept -> const SkPaint& { return m_paint; }
      
      /**
       * @brief The current active SkFont.
       */
      inline fun font() const noexcept -> const SkFont&   { return m_font; }
      
      /**
       * @brief The default loaded font typeface.
       */
      inline fun fontType() const noexcept -> sk_sp<SkTypeface> { return m_fontType; }
      
      /**
       * @brief The Skia font manager.
       */
      inline fun fontMgr() const noexcept -> sk_sp<SkFontMgr> { return m_fontMgr; }
      
      /**
       * @brief Creates a cloned typeface adjusted with variable font parameters.
       * 
       * @param GRAD Gradient axis value.
       * @param WGHT Weight axis value.
       * @param WDTH Width axis value.
       * @param OPSZ Optical size axis value.
       * @return The modified typeface.
       */
      [[nodiscard]] inline fun fontType(f32 GRAD, f32 WGHT, f32 WDTH, f32 OPSZ) const noexcept -> sk_sp<SkTypeface>
      {
        // Types
        struct AxisInfo { bool supported{false}; f32 min, max, def; };
        struct FontVitals { AxisInfo grad, wght, wdth, opsz; };

        auto scale_axis = [](f32 val, AxisInfo AX) noexcept -> f32 {
          val = std::clamp<f32>(val, -1.0, +1.0);

          return val >= 0
            ? AX.def + (val * (AX.max - AX.def))
            : AX.def + (val * (AX.def - AX.min));
        };


        // Axis Check
        int axisCount = m_fontType->getVariationDesignParameters(SkSpan<SkFontParameters::Variation::Axis>(nil, 0));
        if (axisCount <= 0) [[unlikely]] {
          static bool SentWarning{false};

          if (!SentWarning) {
            platform::qcl_warning("The font used is not variable");
            SentWarning = true;
          }

          return m_fontType;
        }

        vector<SkFontParameters::Variation::Axis> axes(axisCount);
        m_fontType->getVariationDesignParameters(SkSpan(axes.data(), axes.size()));

        FontVitals Vitals{};
        for (const auto& axis: axes) {
          auto map_axis = [&](AxisInfo& q_axis) {
            q_axis.min = axis.min;
            q_axis.max = axis.max;
            q_axis.def = axis.def;
            q_axis.supported = true;
          };

          switch (axis.tag) {
            case SkSetFourByteTag('G', 'R', 'A', 'D'): map_axis(Vitals.grad); break;
            case SkSetFourByteTag('w', 'g', 'h', 't'): map_axis(Vitals.wght); break;
            case SkSetFourByteTag('w', 'd', 't', 'h'): map_axis(Vitals.wdth); break;
            case SkSetFourByteTag('o', 'p', 's', 'z'): map_axis(Vitals.opsz); break;
          }
        }


        // Create Axis
        vector<SkFontArguments::VariationPosition::Coordinate> Coors;

        if (GRAD != 0 && Vitals.grad.supported)
          Coors.push_back({SkSetFourByteTag('G','R','A','D'), scale_axis(GRAD, Vitals.grad)});

        if (WGHT != 0 && Vitals.wght.supported)
          Coors.push_back({SkSetFourByteTag('w','g','h','t'), scale_axis(WGHT, Vitals.wght)});

        if (WDTH != 0 && Vitals.wdth.supported)
          Coors.push_back({SkSetFourByteTag('w','d','t','h'), scale_axis(WDTH, Vitals.wdth)});
        
        if (OPSZ != 0 && Vitals.opsz.supported)
          Coors.push_back({SkSetFourByteTag('o','p','s','z'), scale_axis(OPSZ, Vitals.opsz)});

        
        SkFontArguments Args;
        Args.setVariationDesignPosition({Coors.data(), (int)Coors.size()});

        return m_fontType->makeClone(Args);
      }

      /**
       * @brief UI display scaling factor (DPI).
       */
      inline fun scaleFactor() const noexcept -> f32 { return m_scaleFactor; };
      
      /**
       * @brief Font scaling factor.
       */
      inline fun fontFactor() const noexcept -> f32  { return m_fontFactor; };
      
      /**
       * @brief Current animation frame timestamp.
       */
      inline fun currentTime() const noexcept -> u64 { return m_currentTime; };
    

    public:
      renderContext() = delete;

      /**
       * @brief Constructs a new render context.
       */
      inline renderContext(SkCanvas* nCanvas, SkPaint& nPaint, SkFont& nFont, sk_sp<SkTypeface> nFontType, sk_sp<SkFontMgr> nFontMgr, f32 nScaleFactor, f32 nFontFactor, u64 nCurrentTime)
        : m_canvas(nCanvas)
        , m_paint(nPaint)
        , m_font(nFont)
        , m_fontType(nFontType)
        , m_fontMgr(nFontMgr)
        , m_scaleFactor(nScaleFactor)
        , m_fontFactor(nFontFactor)
        , m_currentTime(nCurrentTime)
      {}


    public:
      /**
       * @brief Converts density-independent pixels (dp) to logical pixels based on scale factor.
       */
      inline fun dp(f32 pixels) const noexcept -> f32 { return pixels *scaleFactor(); }
      
      /**
       * @brief Converts scale-independent pixels (sp) to logical pixels based on font factor.
       */
      inline fun sp(f32 pixels) const noexcept -> f32 { return pixels *fontFactor(); }
  };



  /**
   * @brief A generic two-dimensional size descriptor (width, height).
   */
  template <typename T>
  struct size
  {
  public:
    T W,H;
    
  
  public:
    /**
     * @brief Default constructor. Initializes to zero.
     */
    inline size(): W(0), H(0) {}

    /**
     * @brief Constructs size with specific width and height.
     */
    inline size(T nW, T nH): W(nW), H(nH) {}

    template <typename U>
    inline size(const size<U> &Other): W((T)Other.W), H((T)Other.H) {}


  public:
    inline fun operator==(const size &It) const noexcept -> bool { return (W == It.W) && (H == It.H); }

    inline fun operator+(const size &It) const noexcept -> size { return {W +It.W, H +It.H}; }
    inline fun operator-(const size &It) const noexcept -> size { return {W -It.W, H -It.H}; }
    inline fun operator*(const T It) const noexcept -> size { return {W *It, H *It}; }
    inline fun operator/(const T It) const noexcept -> size { return {W /It, H /It}; }

    inline fun operator+=(const size &It) noexcept -> size& { W += It.W; H += It.H; return *this; }
    inline fun operator-=(const size &It) noexcept -> size& { W -= It.W; H -= It.H; return *this; }
    inline fun operator*=(const T It) noexcept -> size& { W *= It; H *= It; return *this; }
    inline fun operator/=(const T It) noexcept -> size& { W /= It; H /= It; return *this; }

  };


  
  /**
   * @brief A generic two-dimensional coordinate point (x, y).
   */
  template <typename T>
  struct poit
  {
  public:
    T X,Y;
    
  
  public:
    /**
     * @brief Default constructor. Initializes to zero.
     */
    inline poit(): X(0), Y(0) {}

    /**
     * @brief Constructs point with given X and Y coordinates.
     */
    inline poit(T nX, T nY): X(nX), Y(nY) {}

    template <typename U>
    inline poit(const poit<U> &Other): X((T)Other.X), Y((T)Other.Y) {}


  public:
    inline fun operator==(const poit &It) const noexcept -> bool { return (X == It.X) & (Y == It.Y); }
    
    inline fun operator+(const poit &It) const noexcept -> poit { return poit{X +It.X, Y +It.Y}; }
    inline fun operator-(const poit &It) const noexcept -> poit { return poit{X -It.X, Y -It.Y}; }
    inline fun operator*(const T It) const noexcept -> poit { return poit{X *It, Y *It}; }
    inline fun operator/(const T It) const noexcept -> poit { return poit{X /It, Y /It}; }
    
    inline fun operator+(const size<T> &It) const noexcept -> poit { return poit{X +It.W, Y +It.H}; }
    inline fun operator-(const size<T> &It) const noexcept -> poit { return poit{X -It.W, Y -It.H}; }
    
    inline fun operator+=(const poit &It) noexcept -> poit& { X += It.X; Y += It.Y; return *this; }
    inline fun operator-=(const poit &It) noexcept -> poit& { X -= It.X; Y -= It.Y; return *this; }

    inline fun operator+=(const T It) noexcept -> poit& { X += It; Y += It; return *this; }
    inline fun operator-=(const T It) noexcept -> poit& { X -= It; Y -= It; return *this; }
    inline fun operator*=(const T It) noexcept -> poit& { X *= It; Y *= It; return *this; }
    inline fun operator/=(const T It) noexcept -> poit& { X /= It; Y /= It; return *this; }


  public:
    inline operator size<T>() const noexcept { return size<T>(X,Y); }

    inline operator SkPoint() const { return {(f32)X, (f32)Y}; }
  };



  /**
   * @brief A generic bounding rectangle defined by top-left (X1, Y1) and bottom-right (X2, Y2).
   */
  template <typename T>
  struct rect
  {
  public:
    T X1,Y1, X2,Y2;


  public:
    /**
     * @brief Default constructor. Initializes to zero.
     */
    inline rect(): X1(0), Y1(0), X2(0), Y2(0) {}

    /**
     * @brief Constructs rectangle with specified edges.
     */
    inline rect(T nX1, T nY1, T nX2, T nY2): X1(nX1), Y1(nY1), X2(nX2), Y2(nY2) {}

    template <typename U>
    inline rect(const rect<U> &Other): X1((T)Other.X1), X2((T)Other.X2), Y1((T)Other.Y1), Y2((T)Other.Y2) {}


    /**
     * @brief Creates a rectangle from two unordered points (corners).
     */
    inline static fun beg_end(poit<T> nBeg, poit<T> nEnd) -> rect
    {
      rect Ret;

      Ret.X1 = min(nBeg.X, nEnd.X);
      Ret.Y1 = min(nBeg.Y, nEnd.Y);

      Ret.X2 = max(nBeg.X, nEnd.X);
      Ret.Y2 = max(nBeg.Y, nEnd.Y);

      return Ret;
    }

    /**
     * @brief Creates a rectangle from a starting position and size.
     */
    inline static fun XYWH(poit<T> nPoit, size<T> nSize) -> rect
    {
      rect Ret;

      Ret.X1 = min<T>(nPoit.X, nPoit.X +nSize.W);
      Ret.Y1 = min<T>(nPoit.Y, nPoit.Y +nSize.H);
      
      Ret.X2 = max<T>(nPoit.X, nPoit.X +nSize.W);
      Ret.Y2 = max<T>(nPoit.Y, nPoit.Y +nSize.H);

      return Ret;
    }

    /**
     * @brief Creates a rectangle from X, Y, Width, and Height.
     */
    inline static fun XYWH(T nX, T nY, T nW, T nH) -> rect
    {
      rect Ret;

      Ret.X1 = nX;
      Ret.Y1 = nY;
      
      Ret.X2 = max(nX, nX +nW);
      Ret.Y2 = max(nY, nY +nH);

      return Ret;
    }

    
  public:
    inline fun get_poit() const noexcept -> const poit<T> { return {X1, Y1}; }
    inline fun get_endpoit() const noexcept -> const poit<T> { return {X2, Y2}; }
    inline fun get_size() const noexcept -> const size<T> { return {X2 -X1, Y2 -Y1}; }


    inline fun set_poit(poit<T> V) noexcept -> void { X2 = V.X +(X2 -X1); Y2 = V.Y +(Y2 -Y1); X1 = V.X; Y1 = V.Y; }
    inline fun set_size(size<T> V) noexcept -> void { X2 = V.W -X1; Y2 = V.H -Y1; }


    /**
     * @brief Checks if a point lies within the rectangle's bounds.
     */
    inline fun is_inbound(const poit<T> &It) const noexcept -> bool { return (X1 <= It.X) & (Y1 <= It.Y) & (X2 >= It.X) & (Y2 >= It.Y); }
    
    /**
     * @brief Checks if this rectangle intersects with another.
     */
    inline fun is_intersects(const rect &It) const noexcept -> bool { return (X1 < It.X2 & X2 > It.X1 & Y1 < It.Y2 & Y2 > It.Y1); }

  public:
    inline fun operator==(const rect &It) const noexcept -> bool { return (X1 == It.X1) & (Y1 == It.Y1) & (X2 == It.X2) & (Y2 == It.Y2); }

    /**
     * @brief Implicit conversion to SkRect.
     */
    inline operator SkRect() const { return SkRect::MakeLTRB(X1, Y1, X2, Y2); }
  };



  /**
   * @brief Input modifier states (Shift, Ctrl, Mouse buttons, etc.).
   */
  enum shiftStates: u32
  {
    ssLeft     = (u32)1 << 0,
    ssRight    = (u32)1 << 1,
    ssMiddle   = (u32)1 << 2,
    
    ssCtrl     = (u32)1 << 3,
    ssShift    = (u32)1 << 4,
    ssAlt      = (u32)1 << 5,
    ssSuper    = (u32)1 << 6,
    ssAltGr    = (u32)1 << 7,
  };
  using shiftStateSet = u32;


  // QEV
  class object;
  class view;

  /**
   * @brief Represents a typed event handler holding a target instance and a member function pointer.
   * 
   * @tparam r The return type.
   * @tparam args Parameter types.
   */
  template <typename r, typename... args>
  struct qev
  {
    view *Inst{};
    r (view::*Func)(object*, args...){};


    qev(){}

    qev(view *nInst, r (view::*nFunc)(object*, args...))
      : Inst(nInst)
      , Func(nFunc)
    {}


    template <typename T>
      requires std::derived_from<T, view>
    inline fun loadFrom(T *nInst, r (T::*nFunc)(object*, args...)) -> void
    {
      Inst = nInst;
      Func = nFunc;
    }


    inline r operator() (object *Ctrl, args... Args)
    {
      if (Inst && Func) {
        if constexpr (std::is_same_v<r, void>)
          (Inst->*Func)(Ctrl, std::forward<args>(Args)...);
        else
          return (Inst->*Func)(Ctrl, std::forward<args>(Args)...);
      }
    }

    inline explicit operator bool() const
    {
      return (Inst && Func);
    }
  };


  template <typename T>
  struct is_qev : std::false_type {};

  template <typename... Args>
  struct is_qev<qev<Args...>> : std::true_type {};


  template <typename T>
    concept qev_ct = is_qev<T>::value;
    
  /**
   * @brief An untyped event handler structure, typically used during property resolution.
   */
  struct qev_seed
  {
    view *Inst{};
    void (view::*Func)(){};
    
    
    qev_seed(){}

    template <typename... args>
    qev_seed(const qev<args...> &Other) 
      : Inst(Other.Inst)
      , Func((void (view::*)())Other.Func)
    {}


    template <typename T, typename... args>
      requires std::derived_from<T, view>
    qev_seed(T *nInst, void (T::*nFunc)(object*, args...))
      : Inst((view*)nInst)
      , Func((void (view::*)())nFunc)
    {}


    template <typename r, typename... args>
    inline fun ToLoad(qev<r, args...> &Target) const -> void
    {
      Target.Inst = this->Inst;
      Target.Func = (r (view::*)(object*, args...))this->Func;
    }
  };

}
