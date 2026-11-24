/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{cell::Cell, rc::Rc};
use nimbus_core::{Application, MethodTable, WidgetRegistry, Window};
use nimbus_math::Color;
use nimbus_theme::{ElevationTokens, Monet, RadiusTokens, StateTokens, StrokeTokens, Theme};
use nimbus_except::*;


fn main() {
	nimbus_except::start();
	{
		let mut registry = WidgetRegistry::global().write().unwrap();
		nimbus_ui::register_widgets(&mut registry);
		nimbus_ui_purple_print::register_widgets(&mut registry);
	}

	//let monet = Monet::from_system().unwrap();
	let monet = Monet::new(
		Color::from_oklch(0.675, 0.138, 65.0),
		Color::from_oklch(0.563, 0.038, 65.0),
		Color::from_oklch(0.517, 0.052, 300.0),
		true
	);

	let theme = Theme {
		elevation: ElevationTokens::default(),
		radius: RadiusTokens::default(),
		state: StateTokens::default(),
		stroke: StrokeTokens::default(),
		monet,
	};

	// Application
	let mut app = Application::new(theme);
	let theme = app.theme.clone();

	// Window
	let main_handle = app.create_window(Window::new("Nimbus Application", 860, 560));
	let main_window = app.window_mut(main_handle).unwrap();
	let tasks = main_window.task_queue();

	let tasks_dialog = tasks.clone();
	let tasks_toast = tasks.clone();
	let tasks_tabs = tasks.clone();

	let target_id = Rc::new(Cell::new(None));
	let target_id_cb = target_id.clone();
	let page_comp_id = Rc::new(Cell::new(None));
	let page_pp_id = Rc::new(Cell::new(None));
	let page_comp_cb = page_comp_id.clone();
	let page_pp_cb = page_pp_id.clone();

	let mut events = MethodTable::new();
	events.register("ToolAction", || {
		nimbus_disp!("QDL Event Callback!");
	});
	events.register("SwitchPage", move |tab_idx: usize| {
		let comp = page_comp_cb.get();
		let pp = page_pp_cb.get();
		tasks_tabs.push_tree(move |tree| {
			if let Some(id) = comp {
				tree.set_visible(id, tab_idx == 0);
			}
			if let Some(id) = pp {
				tree.set_visible(id, tab_idx == 1);
			}
		});
	});
	events.register("OpenWaitDialog", move || {
		if let Some(target) = target_id_cb.get() {
			let tasks_for_worker = tasks_dialog.clone();
			tasks_dialog.push_tree(move |tree| {
				let dlg_id = nimbus_ui::Dialog::attach_wait_overlay(
					tree,
					target,
					"Please Wait",
					"Loading resources, please wait...",
				);

				std::thread::spawn(move || {
					std::thread::sleep(std::time::Duration::from_secs(3));
					tasks_for_worker.push_tree(move |tree| {
						if let Some(dlg) = tree.get_widget_mut::<nimbus_ui::Dialog>(dlg_id) {
							dlg.set_open(false, true);
						}
					});
				});
			});
		}
	});

	let toast_counter = Rc::new(Cell::new(0usize));
	events.register("ShowToast", move || {
		let idx = toast_counter.get();
		toast_counter.set(idx + 1);
		let (title, msg, kind) = match idx % 4 {
			0 => ("Bilgi", "Yeni bir bildirim oluşturuldu.", nimbus_ui::ToastKind::Info),
			1 => ("Başarılı", "Değişiklikler kaydedildi.", nimbus_ui::ToastKind::Success),
			2 => ("Uyarı", "Bazı alanlar varsayılan değerle kaldı.", nimbus_ui::ToastKind::Warning),
			_ => ("Hata", "Sunucuya bağlanılamadı.", nimbus_ui::ToastKind::Error),
		};
		tasks_toast.push_tree(move |tree| {
			nimbus_ui::Toast::builder()
				.title(title)
				.message(msg)
				.kind(kind)
				.show(tree);
		});
	});

	let scene = nimbus_qdl::load_into_tree(
		include_str!("main.qdl"),
		main_window.tree_mut(),
		&theme,
		&events,
	)
	.expect("Failed to load qdl window");

	target_id.set(Some(scene.root_id));
	page_comp_id.set(scene.named_nodes.get("page_components").copied());
	page_pp_id.set(scene.named_nodes.get("page_purpleprint").copied());

	// Pre-connect example PurplePrint wires
	if let (Some(&pp_id), Some(&var_a), Some(&ev), Some(&add)) = (
		scene.named_nodes.get("page_purpleprint"),
		scene.named_nodes.get("pp_var_a"),
		scene.named_nodes.get("pp_event"),
		scene.named_nodes.get("pp_add"),
	) && let Some(pp) = main_window
		.tree_mut()
		.get_widget_mut::<nimbus_ui_purple_print::PurplePrintView>(pp_id)
	{
		pp.connect(ev, 0, add, 0);
		pp.connect(var_a, 0, add, 1);
	}

	
	// Run Application
	if let Err(e) = app.run() {
		nimbus_error!("Application exited with error: {}", e);
	}
}
