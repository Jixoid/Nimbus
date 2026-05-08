/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/types.hh"
#include "qcl/visual.hh"

using namespace std;
using namespace qcl;



namespace qstd
{

  struct monetRole
  {
    public:
      enum monetRoleType: u32 {
        None = 0x0,

        Primary          = 0x1'1'0, OnPrimary          = 0x1'1'1,
        PrimaryContainer = 0x1'2'0, OnPrimaryContainer = 0x1'2'1,
        PrimaryFixed     = 0x1'3'0, OnPrimaryFixed     = 0x1'3'1,
        PrimaryVariant   = 0x1'4'0, OnPrimaryVariant   = 0x1'4'1,

        Secondary          = 0x2'1'0, OnSecondary          = 0x2'1'1,
        SecondaryContainer = 0x2'2'0, OnSecondaryContainer = 0x2'2'1,
        SecondaryFixed     = 0x2'3'0, OnSecondaryFixed     = 0x2'3'1,
        SecondaryVariant   = 0x2'4'0, OnSecondaryVariant   = 0x2'4'1,

        Tertiary          = 0x3'1'0, OnTertiary          = 0x3'1'1,
        TertiaryContainer = 0x3'2'0, OnTertiaryContainer = 0x3'2'1,
        TertiaryFixed     = 0x3'3'0, OnTertiaryFixed     = 0x3'3'1,
        TertiaryVariant   = 0x3'4'0, OnTertiaryVariant   = 0x3'4'1,

        Error          = 0x4'1'0, OnError          = 0x4'1'1,
        ErrorContainer = 0x4'2'0, OnErrorContainer = 0x4'2'1,
        ErrorFixed     = 0x4'3'0, OnErrorFixed     = 0x4'3'1,
        ErrorVariant   = 0x4'4'0, OnErrorVariant   = 0x4'4'1,

        Surface          = 0x5'1'0, OnSurface          = 0x5'1'1,
        SurfaceContainer = 0x5'2'0, OnSurfaceContainer = 0x5'2'1,
        SurfaceFixed     = 0x5'3'0, OnSurfaceFixed     = 0x5'3'1,
        SurfaceVariant   = 0x5'4'0, OnSurfaceVariant   = 0x5'4'1,
        
        SurfaceContainerLowest  = 0x5'5'0, OnSurfaceContainerLowest  = 0x5'5'1,
        SurfaceContainerLow     = 0x5'6'0, OnSurfaceContainerLow     = 0x5'6'1,
        SurfaceContainerHigh    = 0x5'7'0, OnSurfaceContainerHigh    = 0x5'7'1,
        SurfaceContainerHighest = 0x5'8'0, OnSurfaceContainerHighest = 0x5'8'1,

        SurfaceDim       = 0x5'9'0, OnSurfaceDim       = 0x5'9'1,
        SurfaceBright    = 0x5'A'0, OnSurfaceBright    = 0x5'A'1,
        SurfaceOutline   = 0x5'B'0,
        SurfaceOutlineVariant = 0x5'C'0,
      };


    private:
      monetRoleType m_role;

    public:
      inline fun role() const noexcept -> monetRoleType { return m_role; }
      inline fun setRole(monetRoleType val) noexcept -> void { m_role = val; }

    public:
      monetRole() = delete;

      inline monetRole(monetRoleType val): m_role(val) {}

      
    public:
      constexpr fun on() const noexcept -> monetRole { return monetRole(monetRoleType(m_role | 0x1)); }
      constexpr fun unOn() const noexcept -> monetRole { return monetRole(monetRoleType(m_role & ~0x1)); }
      constexpr fun isOn() const noexcept -> bool { return (m_role & 0x1); }

      constexpr fun isPrimary() const noexcept -> bool   { return (m_role & 0xF'0'0) == 0x1'0'0; }
      constexpr fun isSecondary() const noexcept -> bool { return (m_role & 0xF'0'0) == 0x2'0'0; }
      constexpr fun isTertiary() const noexcept -> bool  { return (m_role & 0xF'0'0) == 0x3'0'0; }
      constexpr fun isError() const noexcept -> bool     { return (m_role & 0xF'0'0) == 0x4'0'0; }
      constexpr fun isSurface() const noexcept -> bool   { return (m_role & 0xF'0'0) == 0x5'0'0; }

      constexpr fun isMain() const noexcept -> bool      { return (m_role & 0xF'0) == 0x1'0'0; }
      constexpr fun isContainer() const noexcept -> bool { return (m_role & 0xF'0) == 0x2'0'0; }
      constexpr fun isFixed() const noexcept -> bool     { return (m_role & 0xF'0) == 0x3'0'0; }
      constexpr fun isVariant() const noexcept -> bool   { return (m_role & 0xF'0) == 0x4'0'0; }
  };


  struct subMonetPalette
  {
    public:
      subMonetPalette() = delete;

      subMonetPalette(color theme, bool isDark, f64 Chroma = 1.0);

    private:
      color m_main,      m_onMain;
      color m_container, m_onContainer;
      color m_fixed,     m_onFixed;
      color m_variant,   m_onVariant;

    public:
      inline fun main() const noexcept -> const color& { return m_main; } // En Dikkat çekici renk
      inline fun onMain() const noexcept -> const color& { return m_onMain; }
      inline fun container() const noexcept -> const color& { return m_container; } // Container için renk
      inline fun onContainer() const noexcept -> const color& { return m_onContainer; }
      inline fun fixed() const noexcept -> const color& { return m_fixed; } // Sabit yerli ve renkli yerler
      inline fun onFixed() const noexcept -> const color& { return m_onFixed; }
      inline fun variant() const noexcept -> const color& { return m_variant; } // Disabled görünümlü ikonlar
      inline fun onVariant() const noexcept -> const color& { return m_onVariant; }
  };


  struct subMonetSurface
  {
    public:
      subMonetSurface() = delete;

      subMonetSurface(color theme, bool isDark);

    private:
      color m_main,      m_onMain;
      color m_container, m_onContainer;
      color m_fixed,     m_onFixed;
      color m_variant,   m_onVariant;
      color m_dim,       m_onDim;
      color m_bright,    m_onBright;
      color m_containerLowest,  m_onContainerLowest;
      color m_containerLow,     m_onContainerLow;
      color m_containerHigh,    m_onContainerHigh;
      color m_containerHighest, m_onContainerHighest;
      color m_outline;
      color m_outlineVariant;

    public:
      inline fun main() const noexcept -> const color& { return m_main; }
      inline fun onMain() const noexcept -> const color& { return m_onMain; }
      inline fun container() const noexcept -> const color& { return m_container; }
      inline fun onContainer() const noexcept -> const color& { return m_onContainer; }
      inline fun fixed() const noexcept -> const color& { return m_fixed; }
      inline fun onFixed() const noexcept -> const color& { return m_onFixed; }
      inline fun variant() const noexcept -> const color& { return m_variant; }
      inline fun onVariant() const noexcept -> const color& { return m_onVariant; }
      inline fun dim() const noexcept -> const color& { return m_dim; }
      inline fun onDim() const noexcept -> const color& { return m_onDim; }
      inline fun bright() const noexcept -> const color& { return m_bright; }
      inline fun onBright() const noexcept -> const color& { return m_onBright; }
      inline fun containerLowest() const noexcept -> const color& { return m_containerLowest; }
      inline fun onContainerLowest() const noexcept -> const color& { return m_onContainerLowest; }
      inline fun containerLow() const noexcept -> const color& { return m_containerLow; }
      inline fun onContainerLow() const noexcept -> const color& { return m_onContainerLow; }
      inline fun containerHigh() const noexcept -> const color& { return m_containerHigh; }
      inline fun onContainerHigh() const noexcept -> const color& { return m_onContainerHigh; }
      inline fun containerHighest() const noexcept -> const color& { return m_containerHighest; }
      inline fun onContainerHighest() const noexcept -> const color& { return m_onContainerHighest; }
      inline fun outline() const noexcept -> const color& { return m_outline; }
      inline fun outlineVariant() const noexcept -> const color& { return m_outlineVariant; }
  };


  struct monet
  {
    public:
      monet() = delete;

      inline monet(color p, color s, color t, bool isDark)
        : m_primary(p, isDark)
        , m_secondary(s, isDark, 0.7)
        , m_tertiary(t, isDark)
        , m_error(color::rgba(1,0,0), isDark)
        , m_surface(p, isDark)
      {}

    private:
      subMonetPalette m_primary;
      subMonetPalette m_secondary;
      subMonetPalette m_tertiary;
      subMonetPalette m_error;
      subMonetSurface m_surface;

    public:
      inline fun primary() const noexcept -> const subMonetPalette& { return m_primary; }
      inline fun secondary() const noexcept -> const subMonetPalette& { return m_secondary; }
      inline fun tertiary() const noexcept -> const subMonetPalette& { return m_tertiary; }
      inline fun error() const noexcept -> const subMonetPalette& { return m_error; }
      inline fun surface() const noexcept -> const subMonetSurface& { return m_surface; }

    public:
      inline constexpr fun get(monetRole Role) const noexcept -> const color {switch (Role.role()) {
        case monetRole::Primary:            return m_primary.main();
        case monetRole::OnPrimary:          return m_primary.onMain();
        case monetRole::PrimaryContainer:   return m_primary.container();
        case monetRole::OnPrimaryContainer: return m_primary.onContainer();
        case monetRole::PrimaryFixed:       return m_primary.fixed();
        case monetRole::OnPrimaryFixed:     return m_primary.onFixed();
        case monetRole::PrimaryVariant:     return m_primary.variant();
        case monetRole::OnPrimaryVariant:   return m_primary.onVariant();

        case monetRole::Secondary:            return m_secondary.main();
        case monetRole::OnSecondary:          return m_secondary.onMain();
        case monetRole::SecondaryContainer:   return m_secondary.container();
        case monetRole::OnSecondaryContainer: return m_secondary.onContainer();
        case monetRole::SecondaryFixed:       return m_secondary.fixed();
        case monetRole::OnSecondaryFixed:     return m_secondary.onFixed();
        case monetRole::SecondaryVariant:     return m_secondary.variant();
        case monetRole::OnSecondaryVariant:   return m_secondary.onVariant();

        case monetRole::Tertiary:            return m_tertiary.main();
        case monetRole::OnTertiary:          return m_tertiary.onMain();
        case monetRole::TertiaryContainer:   return m_tertiary.container();
        case monetRole::OnTertiaryContainer: return m_tertiary.onContainer();
        case monetRole::TertiaryFixed:       return m_tertiary.fixed();
        case monetRole::OnTertiaryFixed:     return m_tertiary.onFixed();
        case monetRole::TertiaryVariant:     return m_tertiary.variant();
        case monetRole::OnTertiaryVariant:   return m_tertiary.onVariant();

        case monetRole::Error:            return m_error.main();
        case monetRole::OnError:          return m_error.onMain();
        case monetRole::ErrorContainer:   return m_error.container();
        case monetRole::OnErrorContainer: return m_error.onContainer();
        case monetRole::ErrorFixed:       return m_error.fixed();
        case monetRole::OnErrorFixed:     return m_error.onFixed();
        case monetRole::ErrorVariant:     return m_error.variant();
        case monetRole::OnErrorVariant:   return m_error.onVariant();

        case monetRole::Surface:               return m_surface.main();
        case monetRole::OnSurface:             return m_surface.onMain();
        case monetRole::SurfaceContainer:      return m_surface.container();
        case monetRole::OnSurfaceContainer:    return m_surface.onContainer();
        case monetRole::SurfaceFixed:          return m_surface.fixed();
        case monetRole::OnSurfaceFixed:        return m_surface.onFixed();
        case monetRole::SurfaceVariant:        return m_surface.variant();
        case monetRole::OnSurfaceVariant:      return m_surface.onVariant();
        
        case monetRole::SurfaceContainerLowest:    return m_surface.containerLowest();
        case monetRole::OnSurfaceContainerLowest:  return m_surface.onContainerLowest();
        case monetRole::SurfaceContainerLow:       return m_surface.containerLow();
        case monetRole::OnSurfaceContainerLow:     return m_surface.onContainerLow();
        case monetRole::SurfaceContainerHigh:      return m_surface.containerHigh();
        case monetRole::OnSurfaceContainerHigh:    return m_surface.onContainerHigh();
        case monetRole::SurfaceContainerHighest:   return m_surface.containerHighest();
        case monetRole::OnSurfaceContainerHighest: return m_surface.onContainerHighest();

        case monetRole::SurfaceDim:            return m_surface.dim();
        case monetRole::OnSurfaceDim:          return m_surface.onDim();
        case monetRole::SurfaceBright:         return m_surface.bright();
        case monetRole::OnSurfaceBright:       return m_surface.onBright();
        case monetRole::SurfaceOutline:        return m_surface.outline();
        case monetRole::SurfaceOutlineVariant: return m_surface.outlineVariant();

        default: return{};
      }}
  };

  extern monet Monet;



  struct dcolor
  {
    protected:
      monetRole m_monetRole{monetRole::None};
      f32 m_opacity{1.0};
      visual* m_owner{};

      string m_key;
      color m_color;
    
    
    public:
      dcolor() = delete;

      inline dcolor(visual* nOwner, monetRole nRole, f32 nOpacity = 1.0): m_owner(nOwner), m_monetRole(nRole), m_opacity(nOpacity) { update(); }
      inline dcolor(visual* nOwner, string nKey): m_owner(nOwner), m_key(nKey), m_color() { update(); }


    public:
      fun update(string nKey) -> void;
      fun update(monetRole nRole, f32 nOpacity = 1.0) -> void;
      fun update() -> void;


    public:
      inline operator color() const noexcept { return m_color; }

      inline operator SkColor4f() const { return m_color; }
  };
  
}
