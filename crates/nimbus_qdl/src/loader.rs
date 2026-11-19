/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;

use nimbus_core::{
	MethodTable, ObjectID, PropertyCtx, PropertyValue, Theme, UiTree, Widget, WidgetRegistry,
};
use nimbus_except::{nimbus_log, nimbus_warn};

use crate::ast::{QdlDocument, QdlNode};
use crate::parser::parse;


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
	methods: &mut MethodTable,
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
	let mut empty_methods = MethodTable::new();
	load_into_tree(qdl_source, tree, theme, &mut empty_methods)
}

pub fn load_into_tree_with_registry(
	qdl_source: &str,
	tree: &mut UiTree,
	theme: &Theme,
	methods: &mut MethodTable,
	registry: &WidgetRegistry,
) -> Result<QdlScene, String> {
	let doc = parse(qdl_source)?;
	let root_node = doc
		.root
		.as_ref()
		.ok_or_else(|| "No root node found in QDL document".to_string())?;

	let mut named_nodes = HashMap::new();
	let root_widget = instantiate_node(root_node, &doc, theme, methods, registry)?;
	let root_id = tree.insert_root_boxed(root_widget);

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

#[allow(clippy::too_many_arguments)]
fn load_child_recursive(
	node: &QdlNode,
	parent_id: ObjectID,
	tree: &mut UiTree,
	doc: &QdlDocument,
	theme: &Theme,
	methods: &mut MethodTable,
	registry: &WidgetRegistry,
	named_nodes: &mut HashMap<String, ObjectID>,
) -> Result<ObjectID, String> {
	let widget = instantiate_node(node, doc, theme, methods, registry)?;
	let id = tree.insert_child_boxed(parent_id, widget);

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
	methods: &mut MethodTable,
	registry: &WidgetRegistry,
) -> Result<Box<dyn Widget>, String> {
	let mut widget = registry.create(&node.kind).ok_or_else(|| {
		format!(
			"Unknown widget kind '{}' in QDL. Did you register it in WidgetRegistry?",
			node.kind
		)
	})?;

	let ctx = PropertyCtx { theme };

	// Styles
	for style_name in &node.styles {
		if let Some(style_props) = doc.styles.get(style_name) {
			for (key, val) in style_props {
				apply_property(&mut widget, key, val, &ctx, methods);
			}
		}
	}

	// Methods
	for (key, val) in &node.props {
		apply_property(&mut widget, key, val, &ctx, methods);
	}

	Ok(widget)
}

fn apply_property(
	widget: &mut Box<dyn Widget>,
	key: &str,
	val: &PropertyValue,
	ctx: &PropertyCtx,
	methods: &mut MethodTable,
) {
	if widget.is_method(key) {
		if let Some(method_name) = val.as_str() {
			if let Some(method_ptr) = methods.take(method_name) {
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
		if !widget.set_property(key, val, ctx) {
			nimbus_log!(
				"Property '{}' not found by widget",
				key,
			);
		}
	}
}
