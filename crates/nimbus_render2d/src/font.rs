/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub static DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../res/DejaVuSans.ttf");
static DEFAULT_FONT: OnceLock<FontFace> = OnceLock::new();


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

impl FontId {
	pub const DEFAULT: Self = Self(0);
}


#[derive(Clone)]
pub struct FontFace {
	pub(crate) inner: Arc<fontdue::Font>,
}

impl FontFace {
	pub fn new(font: fontdue::Font) -> Self {
		Self {
			inner: Arc::new(font),
		}
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
		let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?;
		Ok(Self {
			inner: Arc::new(font),
		})
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

	pub fn raw(&self) -> &fontdue::Font {
		&self.inner
	}
}


pub struct FontManager {
	fonts: Vec<FontFace>,
	names: HashMap<String, FontId>,
}

impl FontManager {
	pub fn new() -> Self {
		let default_face = Self::default_font_face().clone();
		let mut names = HashMap::new();
		names.insert("default".to_string(), FontId::DEFAULT);
		Self {
			fonts: vec![default_face],
			names,
		}
	}

	pub fn default_font_face() -> &'static FontFace {
		DEFAULT_FONT.get_or_init(|| {
			let font = fontdue::Font::from_bytes(DEFAULT_FONT_BYTES, fontdue::FontSettings::default())
				.expect("Failed to parse embedded default font");

			FontFace {
				inner: Arc::new(font),
			}
		})
	}

	pub fn default_font() -> &'static FontFace {
		Self::default_font_face()
	}

	pub fn register(&mut self, name: impl Into<String>, font: FontFace) -> FontId {
		let id = FontId(self.fonts.len() as u32);
		let name_str = name.into();
		self.names.insert(name_str, id);
		self.fonts.push(font);
		id
	}

	pub fn load_from_bytes(
		&mut self,
		name: impl Into<String>,
		bytes: &[u8],
	) -> Result<FontId, &'static str> {
		let face = FontFace::from_bytes(bytes)?;
		Ok(self.register(name, face))
	}

	pub fn get(&self, id: FontId) -> Option<&FontFace> {
		self.fonts.get(id.0 as usize)
	}

	pub fn get_by_name(&self, name: &str) -> Option<FontId> {
		self.names.get(name).copied()
	}

	pub fn font_or_default(&self, id: Option<FontId>) -> (FontId, &FontFace) {
		if let Some(id) = id
			&& let Some(face) = self.get(id)
		{
			return (id, face);
		}
		(
			FontId::DEFAULT,
			self
				.fonts
				.first()
				.unwrap_or_else(|| Self::default_font_face()),
		)
	}
}

impl Default for FontManager {
	fn default() -> Self {
		Self::new()
	}
}


#[derive(Clone, Copy)]
pub struct Font<'a> {
	id: FontId,
	fface: &'a FontFace,
	size: f32,
}

impl<'a> Font<'a> {
	pub fn new(fface: &'a FontFace, size: f32) -> Font<'a> {
		Self {
			id: FontId::DEFAULT,
			fface,
			size,
		}
	}

	pub fn with_id(id: FontId, fface: &'a FontFace, size: f32) -> Font<'a> {
		Self { id, fface, size }
	}

	pub fn id(&self) -> FontId {
		self.id
	}

	pub fn face(&self) -> &'a FontFace {
		self.fface
	}

	pub fn size(&self) -> f32 {
		self.size
	}

	pub fn measure_text(&self, text: &str) -> (f32, f32) {
		self.fface.measure_text(text, self.size)
	}
}
