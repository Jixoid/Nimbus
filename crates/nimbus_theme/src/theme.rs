/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Color;

use crate::Monet;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
  pub monet: Monet,
  pub radius: RadiusTokens,
  pub elevation: ElevationTokens,
  pub state: StateTokens,
  pub stroke: StrokeTokens,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
  pub offset_x: f32,
  pub offset_y: f32,
  pub blur_radius: f32,
  pub spread_radius: f32,
  pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationTokens {
  pub level0: Shadow,
  pub level1: Shadow,
  pub level2: Shadow,
  pub level3: Shadow,
  pub level4: Shadow,
  pub level5: Shadow,
}

impl Default for ElevationTokens {
  fn default() -> Self {
    Self {
      level0: Shadow{offset_x: 0.0, offset_y: 0.0, blur_radius: 0.0, spread_radius: 0.0, color: Color::BLACK},
      level1: Shadow{offset_x: 0.0, offset_y: 1.0, blur_radius: 3.0, spread_radius: 1.0, color: Color::BLACK},
      level2: Shadow{offset_x: 0.0, offset_y: 2.0, blur_radius: 6.0, spread_radius: 2.0, color: Color::BLACK},
      level3: Shadow{offset_x: 0.0, offset_y: 4.0, blur_radius: 8.0, spread_radius: 3.0, color: Color::BLACK},
      level4: Shadow{offset_x: 0.0, offset_y: 6.0, blur_radius: 10.0, spread_radius: 4.0, color: Color::BLACK},
      level5: Shadow{offset_x: 0.0, offset_y: 8.0, blur_radius: 12.0, spread_radius: 6.0, color: Color::BLACK},
    }
  }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadiusTokens {
  pub none: f32,
  pub xsmall: f32,
  pub small: f32,
  pub medium: f32,
  pub large: f32,
  pub xlarge: f32,
  pub full: f32,
}

impl Default for RadiusTokens {
  fn default() -> Self {
    RadiusTokens {
      none: 0.0,
      xsmall: 4.0,
      small: 8.0,
      medium: 12.0,
      large: 16.0,
      xlarge: 28.0,
      full: f32::MAX,
    }
  }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateTokens {
  pub hover: f32,
  pub focus: f32,
  pub pressed: f32,
  pub dragged: f32,
  pub disabled: f32,
}

impl Default for StateTokens {
  fn default() -> Self {
    Self {
      hover: 0.08,
      focus: 0.12,
      pressed: 0.12,
      dragged: 0.16,
      disabled: 0.38,
    }
  }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeTokens {
  pub thin: f32,
  pub thick: f32,
  pub heavier: f32,
}

impl Default for StrokeTokens {
  fn default() -> Self {
    Self {
      thin: 1.0,
      thick: 2.0,
      heavier: 3.0,
    }
  }
}
