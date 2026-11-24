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
		nimbus_ui_gizmo::register_widgets(&mut registry);
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
	let main_handle = app.create_window(Window::new("Nimbus Application", 860, 580));
	let main_window = app.window_mut(main_handle).unwrap();
	let tasks = main_window.task_queue();

	let tasks_dialog = tasks.clone();
	let tasks_toast = tasks.clone();
	let tasks_tabs = tasks.clone();
	let tasks_menu = tasks.clone();

	let target_id = Rc::new(Cell::new(None));
	let target_id_cb = target_id.clone();
	let main_tabs_id = Rc::new(Cell::new(None));
	let main_tabs_menu = main_tabs_id.clone();
	let page_comp_id = Rc::new(Cell::new(None));
	let page_pp_id = Rc::new(Cell::new(None));
	let page_gizmo_id = Rc::new(Cell::new(None));
	let page_comp_cb = page_comp_id.clone();
	let page_pp_cb = page_pp_id.clone();
	let page_gizmo_cb = page_gizmo_id.clone();

	let transform_gizmo_id = Rc::new(Cell::new(None));
	let orientation_gizmo_id = Rc::new(Cell::new(None));
	let gizmo_info_id = Rc::new(Cell::new(None));
	let cam_yaw = Rc::new(Cell::new(0.65f32));
	let cam_pitch = Rc::new(Cell::new(0.45f32));

	let mut events = MethodTable::new();
	events.register("ToolAction", || {
		nimbus_disp!("QDL Event Callback!");
	});
	events.register("OnMenuAction", move |action: String, menu: String| {
		nimbus_disp!("Menu action: [{}] {}", menu, action);
		let tabs_id = main_tabs_menu.get();
		tasks_menu.push_tree(move |tree| {
			match action.as_str() {
				"ViewComponents" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus_ui::Tabs>(id)
					{
						tabs.set_tab(0, true, true);
					}
				}
				"ViewPurplePrint" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus_ui::Tabs>(id)
					{
						tabs.set_tab(1, true, true);
					}
				}
				"ViewGizmo" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus_ui::Tabs>(id)
					{
						tabs.set_tab(2, true, true);
					}
				}
				_ => {
					nimbus_ui::Toast::builder()
						.title(format!("Menu: {}", menu))
						.message(format!("Action triggered: {}", action))
						.kind(nimbus_ui::ToastKind::Info)
						.show(tree);
				}
			}
		});
	});
	events.register("SwitchPage", move |tab_idx: usize| {
		let comp = page_comp_cb.get();
		let pp = page_pp_cb.get();
		let gizmo = page_gizmo_cb.get();
		tasks_tabs.push_tree(move |tree| {
			if let Some(id) = comp {
				tree.set_visible(id, tab_idx == 0);
			}
			if let Some(id) = pp {
				tree.set_visible(id, tab_idx == 1);
			}
			if let Some(id) = gizmo {
				tree.set_visible(id, tab_idx == 2);
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

	let tasks_gizmo_mode = tasks.clone();
	let tg_mode = transform_gizmo_id.clone();
	events.register("GizmoMode", move |idx: i32, _label: String| {
		let tg = tg_mode.get();
		let mode = match idx {
			1 => nimbus_ui_gizmo::GizmoMode::Rotate,
			2 => nimbus_ui_gizmo::GizmoMode::Scale,
			_ => nimbus_ui_gizmo::GizmoMode::Translate,
		};
		tasks_gizmo_mode.push_tree(move |tree| {
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.mode = mode;
			}
		});
	});

	let tasks_gizmo_space = tasks.clone();
	let tg_space = transform_gizmo_id.clone();
	events.register("GizmoSpace", move |idx: i32, _label: String| {
		let tg = tg_space.get();
		let space = match idx {
			1 => nimbus_ui_gizmo::GizmoSpace::Local,
			_ => nimbus_ui_gizmo::GizmoSpace::World,
		};
		tasks_gizmo_space.push_tree(move |tree| {
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.space = space;
			}
		});
	});

	let tasks_gizmo_snap = tasks.clone();
	let tg_snap = transform_gizmo_id.clone();
	events.register("GizmoSnap", move |enabled: bool| {
		let tg = tg_snap.get();
		tasks_gizmo_snap.push_tree(move |tree| {
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.snap.enabled = enabled;
			}
		});
	});

	let tasks_gizmo_reset = tasks.clone();
	let tg_reset = transform_gizmo_id.clone();
	let og_reset = orientation_gizmo_id.clone();
	let info_reset = gizmo_info_id.clone();
	let yaw_reset = cam_yaw.clone();
	let pitch_reset = cam_pitch.clone();
	events.register("GizmoReset", move || {
		yaw_reset.set(0.65);
		pitch_reset.set(0.45);
		let tg = tg_reset.get();
		let og = og_reset.get();
		let info = info_reset.get();
		tasks_gizmo_reset.push_tree(move |tree| {
			let aspect = tg
				.map(|id| {
					let b = tree.bounds(id);
					if b.height() > 1.0 { b.width() / b.height() } else { 16.0 / 9.0 }
				})
				.unwrap_or(16.0 / 9.0);
			let cam = nimbus_ui_gizmo::GizmoCamera::from_orbit(
				nimbus_ui_gizmo::Vec3::ZERO,
				5.0,
				0.65,
				0.45,
				nimbus_ui_gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.transform = nimbus_ui_gizmo::Transform::IDENTITY;
				gizmo.animate_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus_ui_gizmo::OrientationGizmo>(id)
			{
				orient.animate_camera(cam);
			}
			if let Some(id) = info
				&& let Some(text) = tree.get_widget_mut::<nimbus_ui::Text>(id)
			{
				text.text = "Pos: (0.00, 0.00, 0.00)  |  Scale: (1.00, 1.00, 1.00)".to_string();
			}
		});
	});

	let tasks_gizmo_changed = tasks.clone();
	let tg_changed = transform_gizmo_id.clone();
	let info_changed = gizmo_info_id.clone();
	events.register("GizmoTransformChanged", move || {
		let tg = tg_changed.get();
		let info = info_changed.get();
		tasks_gizmo_changed.push_tree(move |tree| {
			let label = tg
				.and_then(|id| tree.get_widget::<nimbus_ui_gizmo::TransformGizmo>(id))
				.map(|gizmo| {
					let p = gizmo.transform.translation;
					let s = gizmo.transform.scale;
					format!(
						"Pos: ({:.2}, {:.2}, {:.2})  |  Scale: ({:.2}, {:.2}, {:.2})",
						p.x, p.y, p.z, s.x, s.y, s.z
					)
				});
			if let (Some(id), Some(label)) = (info, label)
				&& let Some(text) = tree.get_widget_mut::<nimbus_ui::Text>(id)
			{
				text.text = label;
			}
		});
	});

	let tasks_cam_snap = tasks.clone();
	let tg_cam_snap = transform_gizmo_id.clone();
	let og_cam_snap = orientation_gizmo_id.clone();
	let yaw_snap = cam_yaw.clone();
	let pitch_snap = cam_pitch.clone();
	events.register("GizmoCameraSnap", move |preset: String| {
		let (yaw, pitch) = match preset.as_str() {
			"Right" => nimbus_ui_gizmo::CameraViewPreset::Right.orbit_angles(),
			"Left" => nimbus_ui_gizmo::CameraViewPreset::Left.orbit_angles(),
			"Top" => nimbus_ui_gizmo::CameraViewPreset::Top.orbit_angles(),
			"Bottom" => nimbus_ui_gizmo::CameraViewPreset::Bottom.orbit_angles(),
			"Back" => nimbus_ui_gizmo::CameraViewPreset::Back.orbit_angles(),
			_ => nimbus_ui_gizmo::CameraViewPreset::Front.orbit_angles(),
		};
		yaw_snap.set(yaw);
		pitch_snap.set(pitch);
		let tg = tg_cam_snap.get();
		let og = og_cam_snap.get();
		tasks_cam_snap.push_tree(move |tree| {
			let aspect = tg
				.map(|id| {
					let b = tree.bounds(id);
					if b.height() > 1.0 { b.width() / b.height() } else { 16.0 / 9.0 }
				})
				.unwrap_or(16.0 / 9.0);
			let cam = nimbus_ui_gizmo::GizmoCamera::from_orbit(
				nimbus_ui_gizmo::Vec3::ZERO,
				5.0,
				yaw,
				pitch,
				nimbus_ui_gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.animate_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus_ui_gizmo::OrientationGizmo>(id)
			{
				orient.animate_camera(cam);
			}
		});
	});

	let tasks_cam_orbit = tasks.clone();
	let tg_cam_orbit = transform_gizmo_id.clone();
	let og_cam_orbit = orientation_gizmo_id.clone();
	let yaw_orbit = cam_yaw.clone();
	let pitch_orbit = cam_pitch.clone();
	events.register("GizmoCameraOrbit", move |dyaw: f32, dpitch: f32| {
		let yaw = yaw_orbit.get() + dyaw;
		let pitch = (pitch_orbit.get() + dpitch).clamp(-1.54, 1.54);
		yaw_orbit.set(yaw);
		pitch_orbit.set(pitch);
		let tg = tg_cam_orbit.get();
		let og = og_cam_orbit.get();
		tasks_cam_orbit.push_tree(move |tree| {
			let aspect = tg
				.map(|id| {
					let b = tree.bounds(id);
					if b.height() > 1.0 { b.width() / b.height() } else { 16.0 / 9.0 }
				})
				.unwrap_or(16.0 / 9.0);
			let cam = nimbus_ui_gizmo::GizmoCamera::from_orbit(
				nimbus_ui_gizmo::Vec3::ZERO,
				5.0,
				yaw,
				pitch,
				nimbus_ui_gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus_ui_gizmo::TransformGizmo>(id)
			{
				gizmo.set_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus_ui_gizmo::OrientationGizmo>(id)
			{
				orient.set_camera(cam);
			}
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
	main_tabs_id.set(scene.named_nodes.get("main_tabs").copied());
	page_comp_id.set(scene.named_nodes.get("page_components").copied());
	page_pp_id.set(scene.named_nodes.get("page_purpleprint").copied());
	page_gizmo_id.set(scene.named_nodes.get("page_gizmo").copied());
	transform_gizmo_id.set(scene.named_nodes.get("demo_transform_gizmo").copied());
	orientation_gizmo_id.set(scene.named_nodes.get("demo_orientation_gizmo").copied());
	gizmo_info_id.set(scene.named_nodes.get("gizmo_info").copied());

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
