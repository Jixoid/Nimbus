/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub use nimbus_core as core;
pub use nimbus_ui as ui;
pub use nimbus_except as except;
pub use nimbus_math as math;
pub use nimbus_platform as platform;
pub use nimbus_render2d as render2d;
pub use nimbus_rhi_ash as rhi_ash;
pub use nimbus_qdl as qdl;
pub use nimbus_theme as theme;


#[cfg(test)]
mod tests {
	use std::sync::{Arc, Mutex};
	use crate::core::{
		BoxConstraints, ButtonState, MethodTable, MouseButton, UiTree, WidgetRegistry,
	};
	use crate::math::{Color, Vec2};
	use crate::theme::{
		ElevationTokens, Monet, RadiusTokens, StateTokens, StrokeTokens, Theme,
	};

	fn make_theme() -> Theme {
		Theme {
			elevation: ElevationTokens::default(),
			radius: RadiusTokens::default(),
			state: StateTokens::default(),
			stroke: StrokeTokens::default(),
			monet: Monet::new(
				Color::from_oklch(0.675, 0.138, 65.0),
				Color::from_oklch(0.563, 0.038, 65.0),
				Color::from_oklch(0.517, 0.052, 300.0),
				true,
			),
		}
	}

	#[test]
	fn test_qdl_vbox_visible_and_collapsible_integration() {
		let mut registry = WidgetRegistry::new();
		crate::ui::register_widgets(&mut registry);

		let theme = make_theme();
		let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

		let mut methods = MethodTable::new();
		{
			let log = log.clone();
			methods.register("OnCheck", move |checked: bool| {
				log.lock().unwrap().push(format!("check:{checked}"));
			});
		}
		{
			let log = log.clone();
			methods.register("OnToggle", move |open: bool| {
				log.lock().unwrap().push(format!("collapsible:{open}"));
			});
		}
		{
			let log = log.clone();
			methods.register("OnTab", move |idx: usize, name: String| {
				log.lock().unwrap().push(format!("tab:{idx}:{name}"));
			});
		}

		let qdl_src = r#"
			root qstd::vbox "root" {
				spacing: 10
				paddings: 0

				node qstd::spacer "s1" {
					width: 100
					height: 20
				}
				node qstd::spacer "hidden_spacer" {
					width: 100
					height: 80
					visible: false
				}
				node qstd::check "chk" {
					text: "Accept"
					checked: false
					on_changed: "OnCheck"
				}
				node qstd::tabs "tabs" {
					tabs: ["One", "Two"]
					on_changed: "OnTab"
				}
				node qstd::collapsible "sec" {
					title: "More"
					header_height: 36
					paddings: 8
					spacing: 6
					open: false
					on_changed: "OnToggle"

					node qstd::spacer "inner" {
						width: 120
						height: 24
					}
				}
			}
		"#;

		let mut tree = UiTree::new();
		let scene = crate::qdl::load_into_tree_with_registry(
			qdl_src,
			&mut tree,
			&theme,
			&methods,
			&registry,
		)
		.unwrap();

		let s1 = scene.find("s1").unwrap();
		let hidden = scene.find("hidden_spacer").unwrap();
		let chk = scene.find("chk").unwrap();
		let sec = scene.find("sec").unwrap();

		let root_sz = tree.layout_node(scene.root_id, BoxConstraints::loose(Vec2::new(400.0, 800.0)));
		let chk_h = tree.bounds(chk).height();
		let tabs_h = tree.bounds(scene.find("tabs").unwrap()).height();
		let sec_h = tree.bounds(sec).height();
		assert_eq!(sec_h, 36.0);
		assert_eq!(tree.bounds(hidden).size(), Vec2::ZERO);
		// Visible children: s1, chk, tabs, sec -> 3 spacings of 10.0
		assert_eq!(root_sz.y, 20.0 + chk_h + tabs_h + 36.0 + 30.0);

		// Click checkbox
		let chk_center = tree.bounds(chk).center();
		tree.dispatch_pointer_move(chk_center);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, chk_center);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Released, chk_center);

		// Toggle hidden spacer visible via TaskQueue from another thread
		let tq = tree.task_queue();
		std::thread::spawn(move || {
			tq.push(move |t: &mut UiTree| {
				t.set_visible(hidden, true);
			});
		})
		.join()
		.unwrap();
		assert!(tree.process_tasks());
		assert!(tree.is_visible(hidden));

		let root_sz_after = tree.layout_node(scene.root_id, BoxConstraints::loose(Vec2::new(400.0, 800.0)));
		assert_eq!(root_sz_after.y, root_sz.y + 80.0 + 10.0);
		assert_eq!(tree.bounds(s1).height(), 20.0);

		// Expand collapsible inside VBox and verify VBox grows by body height (24 + 8*2 = 40)
		tree.get_widget_mut::<crate::ui::Collapsible>(sec)
			.unwrap()
			.set_open(true, false);
		let root_sz_expanded =
			tree.layout_node(scene.root_id, BoxConstraints::loose(Vec2::new(400.0, 800.0)));
		assert_eq!(tree.bounds(sec).height(), 36.0 + 40.0);
		assert_eq!(root_sz_expanded.y, root_sz_after.y + 40.0);

		let entries = log.lock().unwrap().clone();
		assert_eq!(entries, vec!["check:true"]);
	}
}
