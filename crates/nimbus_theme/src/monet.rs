/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::time::Duration;

use nimbus_math::Color;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monet {
  pub primary: SubMonetPalette,
  pub secondary: SubMonetPalette,
  pub tertiary: SubMonetPalette,
  pub error: SubMonetPalette,
  pub surface: SubMonetSurface,
  pub shadow: Color,
}

impl Monet {
  pub fn new(primary: Color, secondary: Color, tertiary: Color, is_dark: bool) -> Self {
    let (_, p_c, _) = primary.to_oklch();
    let (_, s_c, _) = secondary.to_oklch();
    let (_, t_c, _) = tertiary.to_oklch();

    // Yüzeyler
    let surface = SubMonetSurface::new(primary, is_dark);
    
    // Aksan Paletleri
    let primary = SubMonetPalette::new(primary, p_c, is_dark);
    let secondary = SubMonetPalette::new(secondary, s_c, is_dark);
    let tertiary = SubMonetPalette::new(tertiary, t_c, is_dark);

    // Error Paleti
    let error = SubMonetPalette::new(Color::from_oklch(1.0, 1.0, 28.0), 0.20, is_dark);


    Self {
      primary,
      secondary,
      tertiary,
      error,
      surface,
      shadow: Color::from_oklch(0.0, 0.0, 0.0),
    }
  }
  
  pub fn from_system() -> Option<Self> {
    let c = mundy::Preferences::once_blocking(mundy::Interest::AccentColor, Duration::from_millis(500))?.accent_color.0?;

    let p = Color::new(c.red as f32, c.green as f32, c.blue as f32, c.alpha as f32);
    let lhs = p.to_oklch();

    let s = Color::from_oklch(lhs.0, lhs.1, lhs.2 + 30_f32.to_radians());
    let t = Color::from_oklch(lhs.0, lhs.1, lhs.2 + 60_f32.to_radians());

    Some(Self::new(p, s, t, true))
  }

  
  pub fn get(&self, role: MonetRole) -> Color {
    use MonetRole::*;
    
    match role {
      // Primary
      Primary => self.primary.main,
      OnPrimary => self.primary.on_main,
      PrimaryContainer => self.primary.container,
      OnPrimaryContainer => self.primary.on_container,
      PrimaryFixed => self.primary.fixed,
      PrimaryFixedDim => self.primary.fixed_dim,
      OnPrimaryFixed => self.primary.on_fixed,
      OnPrimaryFixedVariant => self.primary.on_fixed_variant,

      // Secondary
      Secondary => self.secondary.main,
      OnSecondary => self.secondary.on_main,
      SecondaryContainer => self.secondary.container,
      OnSecondaryContainer => self.secondary.on_container,
      SecondaryFixed => self.secondary.fixed,
      SecondaryFixedDim => self.secondary.fixed_dim,
      OnSecondaryFixed => self.secondary.on_fixed,
      OnSecondaryFixedVariant => self.secondary.on_fixed_variant,

      // Tertiary
      Tertiary => self.tertiary.main,
      OnTertiary => self.tertiary.on_main,
      TertiaryContainer => self.tertiary.container,
      OnTertiaryContainer => self.tertiary.on_container,
      TertiaryFixed => self.tertiary.fixed,
      TertiaryFixedDim => self.tertiary.fixed_dim,
      OnTertiaryFixed => self.tertiary.on_fixed,
      OnTertiaryFixedVariant => self.tertiary.on_fixed_variant,

      // Error
      Error => self.error.main,
      OnError => self.error.on_main,
      ErrorContainer => self.error.container,
      OnErrorContainer => self.error.on_container,
      ErrorFixed => self.error.fixed,
      ErrorFixedDim => self.error.fixed_dim,
      OnErrorFixed => self.error.on_fixed,
      OnErrorFixedVariant => self.error.on_fixed_variant,

      // Surface
      Surface => self.surface.main,
      OnSurface => self.surface.on_main,
      SurfaceVariant => self.surface.variant,
      OnSurfaceVariant => self.surface.on_variant,
      SurfaceDim => self.surface.dim,
      SurfaceBright => self.surface.bright,
      SurfaceContainerLowest => self.surface.container_lowest,
      SurfaceContainerLow => self.surface.container_low,
      SurfaceContainer => self.surface.container,
      SurfaceContainerHigh => self.surface.container_high,
      SurfaceContainerHighest => self.surface.container_highest,
      Outline => self.surface.outline,
      OutlineVariant => self.surface.outline_variant,

      // Shadow
      Shadow => self.shadow,
    }
  }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonetRole {
  // Primary
  Primary,
  OnPrimary,
  PrimaryContainer,
  OnPrimaryContainer,
  PrimaryFixed,
  PrimaryFixedDim,
  OnPrimaryFixed,
  OnPrimaryFixedVariant,

  // Secondary
  Secondary,
  OnSecondary,
  SecondaryContainer,
  OnSecondaryContainer,
  SecondaryFixed,
  SecondaryFixedDim,
  OnSecondaryFixed,
  OnSecondaryFixedVariant,

  // Tertiary
  Tertiary,
  OnTertiary,
  TertiaryContainer,
  OnTertiaryContainer,
  TertiaryFixed,
  TertiaryFixedDim,
  OnTertiaryFixed,
  OnTertiaryFixedVariant,

  // Error
  Error,
  OnError,
  ErrorContainer,
  OnErrorContainer,
  ErrorFixed,
  ErrorFixedDim,
  OnErrorFixed,
  OnErrorFixedVariant,

  // Surface
  Surface,
  OnSurface,
  SurfaceVariant,
  OnSurfaceVariant,
  SurfaceDim,
  SurfaceBright,
  SurfaceContainerLowest,
  SurfaceContainerLow,
  SurfaceContainer,
  SurfaceContainerHigh,
  SurfaceContainerHighest,

  // Outlines
  Outline,
  OutlineVariant,

  // System
  Shadow,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubMonetSurface {
  pub main: Color,
  pub on_main: Color,
  
  pub variant: Color,
  pub on_variant: Color,

  pub dim: Color,
  pub bright: Color,

  pub container_lowest: Color,
  pub container_low: Color,
  pub container: Color,
  pub container_high: Color,
  pub container_highest: Color,
  
  pub outline: Color,
  pub outline_variant: Color,
}

impl SubMonetSurface {
  pub fn new(color: Color, is_dark: bool) -> Self {
    let (.., h) = color.to_oklch();

    let c_neutral = 0.012;
    let c_variant = 0.024;

    if is_dark {
      Self {
        main: Color::from_oklch(0.06, c_neutral, h),
        on_main: Color::from_oklch(0.90, c_neutral, h),
        variant: Color::from_oklch(0.30, c_variant, h),
        on_variant: Color::from_oklch(0.80, c_variant, h),
        dim: Color::from_oklch(0.06, c_neutral, h),
        bright: Color::from_oklch(0.24, c_neutral, h),
        container_lowest: Color::from_oklch(0.04, c_neutral, h),
        container_low: Color::from_oklch(0.10, c_neutral, h),
        container: Color::from_oklch(0.12, c_neutral, h),
        container_high: Color::from_oklch(0.17, c_neutral, h),
        container_highest: Color::from_oklch(0.22, c_neutral, h),
        outline: Color::from_oklch(0.60, c_variant, h),
        outline_variant: Color::from_oklch(0.30, c_variant, h),
      }
    } else {
      Self {
        main: Color::from_oklch(0.98, c_neutral, h),
        on_main: Color::from_oklch(0.10, c_neutral, h),
        variant: Color::from_oklch(0.90, c_variant, h),
        on_variant: Color::from_oklch(0.30, c_variant, h),
        dim: Color::from_oklch(0.87, c_neutral, h),
        bright: Color::from_oklch(0.98, c_neutral, h),
        container_lowest: Color::from_oklch(1.00, 0.0, h),
        container_low: Color::from_oklch(0.96, c_neutral, h),
        container: Color::from_oklch(0.94, c_neutral, h),
        container_high: Color::from_oklch(0.92, c_neutral, h),
        container_highest: Color::from_oklch(0.90, c_neutral, h),
        outline: Color::from_oklch(0.50, c_variant, h),
        outline_variant: Color::from_oklch(0.80, c_variant, h),
      }
    }
  }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubMonetPalette {
  pub main: Color,
  pub on_main: Color,
  pub container: Color,
  pub on_container: Color,

  pub fixed: Color,
  pub fixed_dim: Color,
  pub on_fixed: Color,
  pub on_fixed_variant: Color,
}

impl SubMonetPalette {
  pub fn new(color: Color, chroma: f32, is_dark: bool) -> Self {
    let (.., h) = color.to_oklch();

    let fixed = Color::from_oklch(0.90, chroma, h); 
    let fixed_dim = Color::from_oklch(0.80, chroma, h); 
    let on_fixed = Color::from_oklch(0.10, chroma, h);
    let on_fixed_variant = Color::from_oklch(0.30, chroma, h);

    if is_dark { 
      Self {
        main: Color::from_oklch(0.80, chroma, h),
        on_main: Color::from_oklch(0.20, chroma, h),
        container: Color::from_oklch(0.30, chroma, h),
        on_container: Color::from_oklch(0.90, chroma, h),
        fixed,
        fixed_dim,
        on_fixed,
        on_fixed_variant,
      }
    } else {
      Self {
        main: Color::from_oklch(0.40, chroma, h),
        on_main: Color::from_oklch(1.00, 0.0, h),
        container: Color::from_oklch(0.90, chroma, h),
        on_container: Color::from_oklch(0.10, chroma, h),
        fixed,
        fixed_dim,
        on_fixed,
        on_fixed_variant,
      }
    }
  }
}



#[derive(Debug, Clone, Copy, PartialEq)]
enum DColorKey {
  Monet(MonetRole, f32),
  Color(Color<f32>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DColor {
  key: DColorKey,
}

impl DColor {
  pub fn new_monet(mrole: MonetRole, opacity: f32) -> Self {
    Self {
      key: DColorKey::Monet(mrole, opacity),
    }
  }

  pub fn new_color(color: Color) -> Self {
    Self {
      key: DColorKey::Color(color),
    }
  }

  
  pub fn get(&self, monet: &Monet) -> Color {
    match &self.key {
      DColorKey::Color(v) => *v,
      DColorKey::Monet(m, o) => monet.get(*m) * Color::new(0.0, 0.0, 0.0, *o),
    }
  }
}
