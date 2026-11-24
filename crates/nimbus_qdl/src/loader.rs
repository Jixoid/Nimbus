/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;

use nimbus_core::{
	Anchor, MethodTable, ObjectID, PropertyCtx, PropertyError, PropertyValue, UiTree, Widget,
	WidgetRegistry,
};
use nimbus_except::{nimbus_log, nimbus_warn};
use nimbus_theme::Theme;

use crate::ast::{QdlDocument, QdlNode};
use crate::parser::parse;


#[derive(Default)]
struct NodeAttributes {
	anchor: Option<Anchor>,
	visible: Option<bool>,
	enabled: Option<bool>,
}

pub struct QdlScene {
	pub root_id: ObjectID,
	pub named_nodes: HashMap<String, ObjectID>,
}

impl QdlScene {
	pub fn find(&self, name: &str) -> Option<ObjectID> {
		self.named_nodes.get(name).copied()
	}
}

pub fn load_into_tree(
	qdl_source: &str,
	tree: &mut UiTree,
	theme: &Theme,
	methods: &MethodTable,
) -> Result<QdlScene, String> {
	let registry_guard = WidgetRegistry::global()
		.read()
		.map_err(|e| format!("Failed to read global WidgetRegistry: {e}"))?;
	load_into_tree_with_registry(qdl_source, tree, theme, methods, &registry_guard)
}

pub fn load_into_tree_simple(
	qdl_source: &str,
	tree: &mut UiTree,
	theme: &Theme,
) -> Result<QdlScene, String> {
	let empty_methods = MethodTable::new();
	load_into_tree(qdl_source, tree, theme, &empty_methods)
}

pub fn load_into_tree_with_registry(
	qdl_source: &str,
	tree: &mut UiTree,
	theme: &Theme,
	methods: &MethodTable,
	registry: &WidgetRegistry,
) -> Result<QdlScene, String> {
	let doc = parse(qdl_source)?;
	let root_node = doc
		.root
		.as_ref()
		.ok_or_else(|| "No root node found in QDL document".to_string())?;

	let mut named_nodes = HashMap::new();
	let (root_widget, root_attrs) = instantiate_node(root_node, &doc, theme, methods, registry)?;
	let root_id = tree.insert_root_boxed(root_widget);
	apply_node_attributes(tree, root_id, root_attrs);

	if let Some(name) = &root_node.name {
		named_nodes.insert(name.clone(), root_id);
	}

	for child in &root_node.children {
		load_child_recursive(
			child,
			root_id,
			tree,
			&doc,
			theme,
			methods,
			registry,
			&mut named_nodes,
		)?;
	}

	Ok(QdlScene {
		root_id,
		named_nodes,
	})
}

fn apply_node_attributes(tree: &mut UiTree, id: ObjectID, attrs: NodeAttributes) {
	if let Some(anchor) = attrs.anchor {
		tree.set_anchor(id, Some(anchor));
	}
	if let Some(visible) = attrs.visible {
		tree.set_visible(id, visible);
	}
	if let Some(enabled) = attrs.enabled {
		tree.set_enabled(id, enabled);
	}
}

#[allow(clippy::too_many_arguments)]
fn load_child_recursive(
	node: &QdlNode,
	parent_id: ObjectID,
	tree: &mut UiTree,
	doc: &QdlDocument,
	theme: &Theme,
	methods: &MethodTable,
	registry: &WidgetRegistry,
	named_nodes: &mut HashMap<String, ObjectID>,
) -> Result<ObjectID, String> {
	let (widget, attrs) = instantiate_node(node, doc, theme, methods, registry)?;
	let id = tree.insert_child_boxed(parent_id, widget);
	apply_node_attributes(tree, id, attrs);

	if let Some(name) = &node.name {
		named_nodes.insert(name.clone(), id);
	}

	for child in &node.children {
		load_child_recursive(child, id, tree, doc, theme, methods, registry, named_nodes)?;
	}

	Ok(id)
}

fn instantiate_node(
	node: &QdlNode,
	doc: &QdlDocument,
	theme: &Theme,
	methods: &MethodTable,
	registry: &WidgetRegistry,
) -> Result<(Box<dyn Widget>, NodeAttributes), String> {
	let mut widget = registry.create(&node.kind).ok_or_else(|| {
		format!(
			"Unknown widget kind '{}' in QDL. Did you register it in WidgetRegistry?",
			node.kind
		)
	})?;

	let ctx = PropertyCtx { theme };
	let mut attrs = NodeAttributes::default();

	// Styles
	for style_name in &node.styles {
		if let Some(style_props) = doc.styles.get(style_name) {
			for (key, val) in style_props {
				apply_property(&mut widget, &mut attrs, key, val, &ctx, methods);
			}
		}
	}

	// Properties & Methods
	for (key, val) in &node.props {
		apply_property(&mut widget, &mut attrs, key, val, &ctx, methods);
	}

	Ok((widget, attrs))
}

fn apply_property(
	widget: &mut Box<dyn Widget>,
	attrs: &mut NodeAttributes,
	key: &str,
	val: &PropertyValue,
	ctx: &PropertyCtx,
	methods: &MethodTable,
) {
	if key == "anchor" {
		if let Some(parsed) = val.as_str().and_then(Anchor::parse) {
			attrs.anchor = Some(parsed);
		} else {
			nimbus_warn!(
				"Invalid value {:?} for property 'anchor' on widget '{}' (expected e.g. 'left-top', 'right-top', 'center', 'fill')",
				val,
				widget.name(),
			);
		}
		return;
	}

	if key == "visible" {
		if let Some(b) = val.as_bool() {
			attrs.visible = Some(b);
		} else {
			nimbus_warn!(
				"Invalid value {:?} for property 'visible' on widget '{}' (expected bool)",
				val,
				widget.name(),
			);
		}
		return;
	}

	if key == "enabled" {
		if let Some(b) = val.as_bool() {
			attrs.enabled = Some(b);
		} else {
			nimbus_warn!(
				"Invalid value {:?} for property 'enabled' on widget '{}' (expected bool)",
				val,
				widget.name(),
			);
		}
		return;
	}

	if widget.is_method(key) {
		if let Some(method_name) = val.as_str() {
			if let Some(method_ptr) = methods.get(method_name) {
				widget.bind_method(key, method_ptr);
			} else {
				nimbus_warn!(
					"Method '{}' referenced by property '{}' was not found in MethodTable",
					method_name,
					key,
				);
			}
		}
	} else {
		match widget.set_property(key, val, ctx) {
			Ok(()) => {}
			Err(PropertyError::NotFound) => {
				nimbus_log!(
					"Property '{}' not found on widget '{}'",
					key,
					widget.name(),
				);
			}
			Err(PropertyError::InvalidValue { expected }) => {
				nimbus_warn!(
					"Invalid value {:?} for property '{}' on widget '{}' (expected {})",
					val,
					key,
					widget.name(),
					expected,
				);
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use nimbus_core::{BoxConstraints, Canvas2D, DrawCtx, LayoutCtx};
	use nimbus_math::{Color, Vec2};
	use nimbus_theme::{ElevationTokens, Monet, RadiusTokens, StrokeTokens};

	struct DummyBox {
		spacing: f32,
		height: f32,
	}

	impl Widget for DummyBox {
		fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
			let children = ctx.children();
			let mut y = 0.0f32;
			let mut max_w = 0.0f32;
			for (i, child) in children.iter().enumerate() {
				if i > 0 {
					y += self.spacing;
				}
				let sz = ctx.layout_child(*child, BoxConstraints::loose(constraints.max_size()));
				ctx.set_child_position(*child, Vec2::new(0.0, y));
				y += sz.y;
				max_w = max_w.max(sz.x);
			}
			if children.is_empty() {
				constraints.constrain(Vec2::new(100.0, self.height))
			} else {
				constraints.constrain(Vec2::new(max_w, y))
			}
		}

		fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}

		fn set_property(
			&mut self,
			key: &str,
			value: &PropertyValue,
			_ctx: &PropertyCtx,
		) -> Result<(), PropertyError> {
			match key {
				"spacing" => self.spacing = value.expect_f32()?,
				"height" => self.height = value.expect_f32()?,
				_ => return Err(PropertyError::NotFound),
			}
			Ok(())
		}
	}

	#[test]
	fn test_qdl_visible_and_enabled_attributes() {
		let mut registry = WidgetRegistry::new();
		registry.register("dummy::box", || Box::new(DummyBox { spacing: 10.0, height: 30.0 }));

		let theme = Theme {
			elevation: ElevationTokens::default(),
			radius: RadiusTokens::default(),
			stroke: StrokeTokens::default(),
			monet: Monet::new(
				Color::from_oklch(0.675, 0.138, 65.0),
				Color::from_oklch(0.563, 0.038, 65.0),
				Color::from_oklch(0.517, 0.052, 300.0),
				true,
			),
		};

		let qdl = r#"
			root dummy::box {
				spacing: 10
				node dummy::box "first" {
					height: 20
				}
				node dummy::box "hidden" {
					height: 50
					visible: false
					enabled: false
				}
				node dummy::box "last" {
					height: 30
				}
			}
		"#;

		let mut tree = UiTree::new();
		let methods = MethodTable::new();
		let scene = load_into_tree_with_registry(qdl, &mut tree, &theme, &methods, &registry).unwrap();

		let first = scene.find("first").unwrap();
		let hidden = scene.find("hidden").unwrap();
		let last = scene.find("last").unwrap();

		assert!(tree.is_visible(first));
		assert!(!tree.is_visible(hidden));
		assert!(!tree.is_enabled(hidden));
		assert!(tree.is_visible(last));

		tree.layout_node(scene.root_id, BoxConstraints::loose(Vec2::new(400.0, 400.0)));
		// Only first (20) + spacing (10) + last (30) = 60, hidden contributes 0 height and no extra spacing
		assert_eq!(tree.bounds(scene.root_id).height(), 60.0);
		assert_eq!(tree.bounds(hidden).size(), Vec2::ZERO);
		assert_eq!(tree.bounds(last).min.y, 30.0);
	}
}
