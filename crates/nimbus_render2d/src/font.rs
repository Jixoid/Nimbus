/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub static DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../res/DejaVuSans.ttf");

pub struct Font {
	inner: fontdue::Font,
}

impl Font {
	pub fn default_font() -> Self {
		let font = fontdue::Font::from_bytes(DEFAULT_FONT_BYTES, fontdue::FontSettings::default())
			.expect("Failed to parse embedded default font");
		Self { inner: font }
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
		let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?;
		Ok(Self { inner: font })
	}

	pub fn raw(&self) -> &fontdue::Font {
		&self.inner
	}

	pub fn measure_text(&self, text: &str, size_px: f32) -> (f32, f32) {
		let mut width = 0.0f32;
		let line_height = self
			.inner
			.horizontal_line_metrics(size_px)
			.map(|m| m.new_line_size)
			.unwrap_or(size_px * 1.2);

		for ch in text.chars() {
			if ch == '\n' {
				continue;
			}
			let metrics = self.inner.metrics(ch, size_px);
			width += metrics.advance_width;
		}

		(width, line_height)
	}
}
