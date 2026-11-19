/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Color;


#[derive(Clone, Debug)]
pub struct Theme {
	pub background: Color,
	pub surface: Color,
	pub surface_elevated: Color,
	pub primary: Color,
	pub primary_hover: Color,
	pub primary_active: Color,
	pub text_primary: Color,
	pub text_secondary: Color,
	pub text_muted: Color,
	pub border: Color,
	pub border_focus: Color,
	pub shadow: Color,
	pub card_radius: f32,
	pub button_radius: f32,
	pub base_font_size: f32,
}

impl Theme {
	pub fn dark() -> Self {
		Self {
			background: Color::from_hex(0x0C0E14FF),
			surface: Color::from_hex(0x161922FF),
			surface_elevated: Color::from_hex(0x1E222EFF),
			primary: Color::from_hex(0x4F46E5FF),        // Deep Indigo
			primary_hover: Color::from_hex(0x6366F1FF),  // Hover Indigo
			primary_active: Color::from_hex(0x4338CAFF), // Pressed Indigo
			text_primary: Color::from_hex(0xF9FAFBFF),
			text_secondary: Color::from_hex(0x9CA3AFFF),
			text_muted: Color::from_hex(0x6B7280FF),
			border: Color::from_hex(0x272B38FF),
			border_focus: Color::from_hex(0x6366F1FF),
			shadow: Color::new(0.0, 0.0, 0.0, 0.45),
			card_radius: 12.0,
			button_radius: 8.0,
			base_font_size: 14.0,
		}
	}

	pub fn light() -> Self {
		Self {
			background: Color::from_hex(0xF8FAFCFF),
			surface: Color::from_hex(0xFFFFFFFF),
			surface_elevated: Color::from_hex(0xF1F5F9FF),
			primary: Color::from_hex(0x4F46E5FF),
			primary_hover: Color::from_hex(0x6366F1FF),
			primary_active: Color::from_hex(0x4338CAFF),
			text_primary: Color::from_hex(0x0F172AFF),
			text_secondary: Color::from_hex(0x475569FF),
			text_muted: Color::from_hex(0x94A3B8FF),
			border: Color::from_hex(0xE2E8F0FF),
			border_focus: Color::from_hex(0x4F46E5FF),
			shadow: Color::new(0.0, 0.0, 0.0, 0.08),
			card_radius: 12.0,
			button_radius: 8.0,
			base_font_size: 14.0,
		}
	}
}

impl Default for Theme {
	fn default() -> Self {
		Self::dark()
	}
}
