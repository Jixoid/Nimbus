/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{
	Anchor, BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::Vec2;
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};

use crate::tree::DockZone;


pub struct DockPane {
	pub title: String,
	pub closable: bool,
	pub initial_slot: DockZone,
	pub padding: f32,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
}

impl Default for DockPane {
	fn default() -> Self {
		Self {
			title: "Pane".to_string(),
			closable: true,
			initial_slot: DockZone::Center,
			padding: 10.0,
			background: None,
			corner_radius: None,
		}
	}
}

impl DockPane {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			..Default::default()
		}
	}

	pub fn builder() -> DockPaneBuilder {
		DockPaneBuilder::new()
	}
}

impl Widget for DockPane {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		ctx.set_clip_children(true);

		let children = ctx.children();
		let pad = self.padding * 2.0;

		let final_size = constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				240.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				180.0
			},
		));
		let inner_size = Vec2::new((final_size.x - pad).max(0.0), (final_size.y - pad).max(0.0));

		for child in children {
			let anchor = ctx.child_anchor(child).unwrap_or(Anchor::Fill);
			let child_constraints = anchor.child_constraints(inner_size);
			let child_size = ctx.layout_child(child, child_constraints);
			let pos = Vec2::new(self.padding, self.padding) + anchor.position(inner_size, child_size);
			ctx.set_child_position(child, pos);
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		if ctx.bounds.width() <= 0.5 || ctx.bounds.height() <= 0.5 {
			return;
		}
		let monet = &ctx.theme.monet;
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		canvas.draw(RectShape::new(ctx.bounds, bg).radius(radius));
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"title" => self.title = value.expect_str()?.to_string(),
			"closable" => self.closable = value.expect_bool()?,
			"slot" => {
				let s = value.expect_str()?;
				self.initial_slot = DockZone::parse(s).ok_or(PropertyError::InvalidValue {
					expected: "\"center\", \"left\", \"right\", \"top\", or \"bottom\"",
				})?;
			}
			"paddings" => self.padding = value.expect_f32()?.max(0.0),
			"color" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}
}


#[derive(Default)]
pub struct DockPaneBuilder {
	inner: DockPane,
}

impl DockPaneBuilder {
	pub fn new() -> Self {
		Self {
			inner: DockPane::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn closable(mut self, closable: bool) -> Self {
		self.inner.closable = closable;
		self
	}

	pub fn slot(mut self, slot: DockZone) -> Self {
		self.inner.initial_slot = slot;
		self
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.inner.padding = padding.max(0.0);
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius.max(0.0));
		self
	}

	pub fn build(self) -> DockPane {
		self.inner
	}
}

impl From<DockPaneBuilder> for DockPane {
	fn from(builder: DockPaneBuilder) -> Self {
		builder.build()
	}
}
