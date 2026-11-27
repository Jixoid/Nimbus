/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any::Any;
use nimbus_math::Vec2;
use nimbus_render2d::Canvas2D;

use crate::draw::DrawCtx;
use crate::event::{Cursor, EventCtx, EventResponse, InputEvent};
use crate::layout::{BoxConstraints, LayoutCtx};
use crate::method::MethodCallback;
use crate::property::{PropertyCtx, PropertyError, PropertyValue};


pub trait AsAny: 'static {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AsAny for T {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}

pub trait Widget: AsAny + 'static {
	fn name(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32>;

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D);

	fn draw_foreground(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}

	fn should_draw_children(&self) -> bool {
		true
	}

	fn children_stretch(&self) -> Option<([f32; 2], [f32; 2])> {
		None
	}

	fn blocks_touch_scroll(&self, _delta: Vec2<f32>) -> bool {
		false
	}

	fn cursor(&self) -> Cursor {
		Cursor::Default
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let _ = (ctx, event);
		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> Result<(), PropertyError> {
		let _ = (key, value, ctx);
		Err(PropertyError::NotFound)
	}

	fn is_method(&self, name: &str) -> bool {
		let _ = name;
		false
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		let _ = (method_name, callback);
		false
	}

	fn handle_message(&mut self, ctx: &mut EventCtx, name: &str, payload: &PropertyValue) -> bool {
		let _ = (ctx, name, payload);
		false
	}
}

impl Widget for Box<dyn Widget> {
	fn name(&self) -> &'static str {
		(**self).name()
	}

	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		(**self).layout(ctx, constraints)
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		(**self).draw(ctx, canvas);
	}

	fn draw_foreground(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		(**self).draw_foreground(ctx, canvas);
	}

	fn should_draw_children(&self) -> bool {
		(**self).should_draw_children()
	}

	fn children_stretch(&self) -> Option<([f32; 2], [f32; 2])> {
		(**self).children_stretch()
	}

	fn blocks_touch_scroll(&self, delta: Vec2<f32>) -> bool {
		(**self).blocks_touch_scroll(delta)
	}

	fn cursor(&self) -> Cursor {
		(**self).cursor()
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		(**self).handle_event(ctx, event)
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> Result<(), PropertyError> {
		(**self).set_property(key, value, ctx)
	}

	fn is_method(&self, name: &str) -> bool {
		(**self).is_method(name)
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		(**self).bind_method(method_name, callback)
	}

	fn handle_message(&mut self, ctx: &mut EventCtx, name: &str, payload: &PropertyValue) -> bool {
		(**self).handle_message(ctx, name, payload)
	}
}
