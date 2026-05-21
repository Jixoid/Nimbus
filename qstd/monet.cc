/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <iostream>
#include <sstream>
#include "qstd/monet.hh"

#define ef else if
#define el else

using namespace std;
using namespace qcl;



namespace qstd
{
  monet Monet({}, {}, {}, {});


  [[gnu::weak]] subMonetPalette::subMonetPalette(color theme, bool isDark, f64 Chroma)
  {
    auto [L, C, H] = theme.to_oklch();

    if (isDark) {
      this->m_main        = color::oklch(0.80, 0.06 *Chroma, H); 
      this->m_onMain      = color::oklch(0.10, 0.04 *Chroma, H);
      
      this->m_container   = color::oklch(0.70, 0.10'5 *Chroma, H);
      this->m_onContainer = color::oklch(0.10, 0.03 *Chroma, H);
      
      this->m_fixed       = color::oklch(0.60, 0.10 *Chroma, H);
      this->m_onFixed     = color::oklch(0.95, 0.02 *Chroma, H);

      this->m_variant     = color::oklch(0.60, 0.02 *Chroma, H);
      this->m_onVariant   = color::oklch(0.10, 0.01 *Chroma, H);
    }
    else {
      this->m_main        = color::oklch(0.45, std::max(0.10, C) *Chroma, H);
      this->m_onMain      = color::oklch(0.98, 0.01 *Chroma, H);
      
      this->m_container   = color::oklch(0.92, 0.04 *Chroma, H);
      this->m_onContainer = color::oklch(0.15, 0.02 *Chroma, H);
      
      this->m_fixed       = color::oklch(0.85, 0.05 *Chroma, H);
      this->m_onFixed     = color::oklch(0.10, 0.02 *Chroma, H);

      this->m_variant     = color::oklch(0.80, 0.03 *Chroma, H);
      this->m_onVariant   = color::oklch(0.30, 0.02 *Chroma, H);
    }
  }

  [[gnu::weak]] subMonetSurface::subMonetSurface(color theme, bool isDark)
  {
    auto [L, C, H] = theme.to_oklch();

    static constinit f64 surfaceChroma = 0.005;

    if (isDark) {
      this->m_main   = color::oklch(0.15, surfaceChroma, H);
      this->m_onMain = color::oklch(0.90, 0.01, H);

      this->m_container   = color::oklch(0.20, surfaceChroma, H);
      this->m_onContainer = color::oklch(0.95, 0.02, H);

      this->m_containerLowest   = color::oklch(0.10, surfaceChroma, H);
      this->m_onContainerLowest = color::oklch(0.85, 0.01, H);
      
      this->m_containerLow   = color::oklch(0.18, surfaceChroma *0.1, H);
      this->m_onContainerLow = color::oklch(0.90, 0.01, H);
      
      this->m_containerHigh   = color::oklch(0.25, surfaceChroma, H);
      this->m_onContainerHigh = color::oklch(0.92, 0.01, H);
      
      this->m_containerHighest   = color::oklch(0.30, surfaceChroma, H);
      this->m_onContainerHighest = color::oklch(0.95, 0.01, H);
      
      this->m_variant   = color::oklch(0.22, surfaceChroma * 1.5, H);
      this->m_onVariant = color::oklch(0.85, 0.02, H);

      this->m_fixed   = color::oklch(0.90, 0.02, H);
      this->m_onFixed = color::oklch(0.10, 0.02, H);

      this->m_dim      = color::oklch(0.12, surfaceChroma, H);
      this->m_onDim    = color::oklch(0.80, 0.01, H);
      this->m_bright   = color::oklch(0.35, surfaceChroma, H);
      this->m_onBright = color::oklch(0.98, 0.01, H);
      
      this->m_outline        = color::oklch(0.50, 0.02, H);
      this->m_outlineVariant = color::oklch(0.35, 0.02, H);
    }
    else {
      this->m_main           = color::oklch(0.98, surfaceChroma * 0.5, H);
      this->m_onMain         = color::oklch(0.10, 0.02, H);
      
      this->m_container      = color::oklch(0.94, surfaceChroma, H);
      this->m_onContainer    = color::oklch(0.15, 0.02, H);
      
      this->m_fixed          = color::oklch(0.90, surfaceChroma, H);
      this->m_onFixed        = color::oklch(0.10, 0.02, H);
      
      this->m_variant        = color::oklch(0.92, surfaceChroma * 1.2, H);
      this->m_onVariant      = color::oklch(0.25, 0.02, H);

      this->m_dim            = color::oklch(0.88, surfaceChroma, H);
      this->m_onDim          = color::oklch(0.10, 0.02, H);
      this->m_bright         = color::oklch(0.99, surfaceChroma, H);
      this->m_onBright       = color::oklch(0.05, 0.02, H);
      
      this->m_outline        = color::oklch(0.60, 0.02, H);
      this->m_outlineVariant = color::oklch(0.80, 0.02, H);
    }
  }



  fun ParseColor(string Value) -> tuple<monetRole, f32, color>
  {
    if (Value.rfind("rgb", 0) == 0)
    {
      u0 SPos = Value.find('(') +1;
      u0 EPos = Value.find(')');
      string Content = Value.substr(SPos, EPos -SPos);
      
      auto [R,G,B,A] = array<u16,4>{0,0,0,255};
      
      char Sep;
      stringstream ss(Content);
      ss >> hex >> R >> Sep >> G >> Sep >> B;
      
      if (Value.rfind("rgba", 0) == 0)
      {
        ss >> Sep >> A;
      }

      return {monetRole::None, 1.0, color::rgba(R/255.0, G/255.0, B/255.0, A/255.0)};
    }

    ef (Value.rfind("hsl", 0) == 0)
    {
      u0 SPos = Value.find('(') +1;
      u0 EPos = Value.find(')');

      stringstream ss(Value.substr(SPos, EPos -SPos));

      f64 h, s, l;
      
      char sep;
      ss >> h >> sep >> s >> sep >> l;
      return {monetRole::None, 1.0, color::hsl(h, s/100.0, l/100.0)};
    }

    ef (Value.rfind("oklch", 0) == 0)
    {
      u0 SPos = Value.find('(') +1;
      u0 EPos = Value.find(')');

      stringstream ss(Value.substr(SPos, EPos -SPos));

      f64 l, c, h;
      
      char sep;
      ss >> l >> sep >> c >> sep >> h;
      return {monetRole::None, 1.0, color::oklch(l, c, h)};
    }
    

    string Target;
    f64 TAlpha = 1;
    
    if (Value.rfind("monet(", 0) == 0)
    {
      u0 SPos = Value.find('(') +1;
      u0 EPos = Value.find(')');
      string Content = Value.substr(SPos, EPos -SPos);
      

      if (size_t commaPos = Content.find(','); commaPos != string::npos)
      {
        Target = Content.substr(0, commaPos);
        TAlpha = stof(Content.substr(commaPos +1)); 
      }
      el {
        Target = Content;
      }
    }

    #define Reg(Obj, StrName) \
      (Target == StrName) \
      { \
        monetRole R = Obj; \
        return {R, TAlpha, Monet.get(R)}; \
      }

    
    if Reg(monetRole::Primary,            "Primary")
    ef Reg(monetRole::OnPrimary,          "OnPrimary")
    ef Reg(monetRole::PrimaryContainer,   "PrimaryContainer")
    ef Reg(monetRole::OnPrimaryContainer, "OnPrimaryContainer")
    ef Reg(monetRole::PrimaryFixed,       "PrimaryFixed")
    ef Reg(monetRole::OnPrimaryFixed,     "OnPrimaryFixed")
    ef Reg(monetRole::PrimaryVariant,     "PrimaryVariant")
    ef Reg(monetRole::OnPrimaryVariant,   "OnPrimaryVariant")

    ef Reg(monetRole::Secondary,            "Secondary")
    ef Reg(monetRole::OnSecondary,          "OnSecondary")
    ef Reg(monetRole::SecondaryContainer,   "SecondaryContainer")
    ef Reg(monetRole::OnSecondaryContainer, "OnSecondaryContainer")
    ef Reg(monetRole::SecondaryFixed,       "SecondaryFixed")
    ef Reg(monetRole::OnSecondaryFixed,     "OnSecondaryFixed")
    ef Reg(monetRole::SecondaryVariant,     "SecondaryVariant")
    ef Reg(monetRole::OnSecondaryVariant,   "OnSecondaryVariant")

    ef Reg(monetRole::Tertiary,            "Tertiary")
    ef Reg(monetRole::OnTertiary,          "OnTertiary")
    ef Reg(monetRole::TertiaryContainer,   "TertiaryContainer")
    ef Reg(monetRole::OnTertiaryContainer, "OnTertiaryContainer")
    ef Reg(monetRole::TertiaryFixed,       "TertiaryFixed")
    ef Reg(monetRole::OnTertiaryFixed,     "OnTertiaryFixed")
    ef Reg(monetRole::TertiaryVariant,     "TertiaryVariant")
    ef Reg(monetRole::OnTertiaryVariant,   "OnTertiaryVariant")

    ef Reg(monetRole::Error,            "Error")
    ef Reg(monetRole::OnError,          "OnError")
    ef Reg(monetRole::ErrorContainer,   "ErrorContainer")
    ef Reg(monetRole::OnErrorContainer, "OnErrorContainer")
    ef Reg(monetRole::ErrorFixed,       "ErrorFixed")
    ef Reg(monetRole::OnErrorFixed,     "OnErrorFixed")
    ef Reg(monetRole::ErrorVariant,     "ErrorVariant")
    ef Reg(monetRole::OnErrorVariant,   "OnErrorVariant")

    ef Reg(monetRole::Surface,               "Surface")
    ef Reg(monetRole::OnSurface,             "OnSurface")
    ef Reg(monetRole::SurfaceContainer,      "SurfaceContainer")
    ef Reg(monetRole::OnSurfaceContainer,    "OnSurfaceContainer")
    ef Reg(monetRole::SurfaceFixed,          "SurfaceFixed")
    ef Reg(monetRole::OnSurfaceFixed,        "OnSurfaceFixed")
    ef Reg(monetRole::SurfaceVariant,        "SurfaceVariant")
    ef Reg(monetRole::OnSurfaceVariant,      "OnSurfaceVariant")
    
    ef Reg(monetRole::SurfaceContainerLowest,    "SurfaceContainerLowest")
    ef Reg(monetRole::OnSurfaceContainerLowest,  "OnSurfaceContainerLowest")
    ef Reg(monetRole::SurfaceContainerLow,       "SurfaceContainerLow")
    ef Reg(monetRole::OnSurfaceContainerLow,     "OnSurfaceContainerLow")
    ef Reg(monetRole::SurfaceContainerHigh,      "SurfaceContainerHigh")
    ef Reg(monetRole::OnSurfaceContainerHigh,    "OnSurfaceContainerHigh")
    ef Reg(monetRole::SurfaceContainerHighest,   "SurfaceContainerHighest")
    ef Reg(monetRole::OnSurfaceContainerHighest, "OnSurfaceContainerHighest")

    ef Reg(monetRole::SurfaceDim,            "SurfaceDim")
    ef Reg(monetRole::OnSurfaceDim,          "OnSurfaceDim")
    ef Reg(monetRole::SurfaceBright,         "SurfaceBright")
    ef Reg(monetRole::OnSurfaceBright,       "OnSurfaceBright")
    ef Reg(monetRole::SurfaceOutline,        "SurfaceOutline")
    ef Reg(monetRole::SurfaceOutlineVariant, "SurfaceOutlineVariant")

    el {
      cerr << "QCLDBG: unknown color: " << Target << endl;
      return {monetRole::None, 0.5, color()};
    }

    #undef Reg
  }


  fun dcolor::update(string nKey) -> void
  {
    m_key = nKey;
    m_monetRole = monetRole::None;
    update();
  }

  fun dcolor::update(monetRole nRole, f32 nOpacity) -> void
  {
    m_monetRole = nRole;
    m_opacity = nOpacity;
    update();
  }

  fun dcolor::update() -> void
  {
    if (m_monetRole.role() != monetRole::None)
    {
      m_color = Monet.get(m_monetRole) *m_opacity;
    }
    else
    {
      auto X = ParseColor(m_key);

      m_monetRole = std::get<monetRole>(X);
      m_opacity = std::get<f32>(X);
      m_color = std::get<color>(X) *m_opacity;
    }

    m_owner->update(visual::visDirtyDraw);
  }

}
