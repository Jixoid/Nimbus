/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{cell::Cell, rc::Rc};
use nimbus::core::{Application, MethodTable, WidgetRegistry, Window};
use nimbus::math::Color;
use nimbus::theme::{ElevationTokens, Monet, RadiusTokens, StrokeTokens, Theme};
use nimbus::except::*;


fn make_monet(palette: i32, is_dark: bool) -> Monet {
	match palette {
		1 => Monet::new(
			Color::from_oklch(0.65, 0.14, 245.0),
			Color::from_oklch(0.58, 0.06, 220.0),
			Color::from_oklch(0.60, 0.10, 175.0),
			is_dark,
		),
		2 => Monet::new(
			Color::from_oklch(0.64, 0.13, 145.0),
			Color::from_oklch(0.56, 0.05, 130.0),
			Color::from_oklch(0.62, 0.10, 95.0),
			is_dark,
		),
		3 => Monet::new(
			Color::from_oklch(0.66, 0.15, 355.0),
			Color::from_oklch(0.58, 0.06, 330.0),
			Color::from_oklch(0.62, 0.11, 35.0),
			is_dark,
		),
		_ => Monet::new(
			Color::from_oklch(0.675, 0.138, 65.0),
			Color::from_oklch(0.563, 0.038, 65.0),
			Color::from_oklch(0.517, 0.052, 300.0),
			is_dark,
		),
	}
}


fn main() {
	nimbus::except::start();
	{
		let mut registry = WidgetRegistry::global().write().unwrap();
		nimbus::ui::register_widgets(&mut registry);
		nimbus::ext::purple_print::register_widgets(&mut registry);
		nimbus::ext::gizmo::register_widgets(&mut registry);
		nimbus::ext::dock::register_widgets(&mut registry);
		nimbus::ext::timeline::register_widgets(&mut registry);
		nimbus::ext::inspector::register_widgets(&mut registry);
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
		stroke: StrokeTokens::default(),
		monet,
	};

	// Application
	let mut app = Application::new(theme);
	let theme = app.theme.clone();
	let app_tasks = app.task_queue();

	// Window
	let main_handle = app.create_window(Window::new("Nimbus Application", 860, 620));
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
	let page_dock_id = Rc::new(Cell::new(None));
	let page_timeline_id = Rc::new(Cell::new(None));
	let page_inspector_id = Rc::new(Cell::new(None));
	let page_dnd_id = Rc::new(Cell::new(None));
	let page_comp_cb = page_comp_id.clone();
	let page_pp_cb = page_pp_id.clone();
	let page_gizmo_cb = page_gizmo_id.clone();
	let page_dock_cb = page_dock_id.clone();
	let page_timeline_cb = page_timeline_id.clone();
	let page_inspector_cb = page_inspector_id.clone();
	let page_dnd_cb = page_dnd_id.clone();
	let dnd_zone_shader_id = Rc::new(Cell::new(None));
	let dnd_zone_any_id = Rc::new(Cell::new(None));
	let dnd_zone_files_id = Rc::new(Cell::new(None));

	let tl_view_id = Rc::new(Cell::new(None));
	let tl_curve_id = Rc::new(Cell::new(None));
	let tl_play_btn_id = Rc::new(Cell::new(None));
	let tl_readout_id = Rc::new(Cell::new(None));

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
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(0, true, true);
					}
				}
				"ViewPurplePrint" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(1, true, true);
					}
				}
				"ViewGizmo" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(2, true, true);
					}
				}
				"ViewDock" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(3, true, true);
					}
				}
				"ViewTimeline" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(4, true, true);
					}
				}
				"ViewInspector" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(5, true, true);
					}
				}
				"ViewDragDrop" => {
					if let Some(id) = tabs_id
						&& let Some(tabs) = tree.get_widget_mut::<nimbus::ui::Tabs>(id)
					{
						tabs.set_tab(6, true, true);
					}
				}
				_ => {
					nimbus::ui::Toast::builder()
						.title(format!("Menu: {}", menu))
						.message(format!("Action triggered: {}", action))
						.kind(nimbus::ui::ToastKind::Info)
						.show(tree);
				}
			}
		});
	});
	events.register("SwitchPage", move |tab_idx: usize| {
		let comp = page_comp_cb.get();
		let pp = page_pp_cb.get();
		let gizmo = page_gizmo_cb.get();
		let dock = page_dock_cb.get();
		let timeline = page_timeline_cb.get();
		let inspector = page_inspector_cb.get();
		let dnd = page_dnd_cb.get();
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
			if let Some(id) = dock {
				tree.set_visible(id, tab_idx == 3);
			}
			if let Some(id) = timeline {
				tree.set_visible(id, tab_idx == 4);
			}
			if let Some(id) = inspector {
				tree.set_visible(id, tab_idx == 5);
			}
			if let Some(id) = dnd {
				tree.set_visible(id, tab_idx == 6);
			}
		});
	});
	events.register("OpenWaitDialog", move || {
		if let Some(target) = target_id_cb.get() {
			let tasks_for_worker = tasks_dialog.clone();
			tasks_dialog.push_tree(move |tree| {
				let dlg_id = nimbus::ui::Dialog::attach_wait_overlay(
					tree,
					target,
					"Please Wait",
					"Loading resources, please wait...",
				);

				std::thread::spawn(move || {
					std::thread::sleep(std::time::Duration::from_secs(3));
					tasks_for_worker.push_tree(move |tree| {
						if let Some(dlg) = tree.get_widget_mut::<nimbus::ui::Dialog>(dlg_id) {
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
			0 => ("Bilgi", "Yeni bir bildirim oluşturuldu.", nimbus::ui::ToastKind::Info),
			1 => ("Başarılı", "Değişiklikler kaydedildi.", nimbus::ui::ToastKind::Success),
			2 => ("Uyarı", "Bazı alanlar varsayılan değerle kaldı.", nimbus::ui::ToastKind::Warning),
			_ => ("Hata", "Sunucuya bağlanılamadı.", nimbus::ui::ToastKind::Error),
		};
		tasks_toast.push_tree(move |tree| {
			nimbus::ui::Toast::builder()
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
			1 => nimbus::ext::gizmo::GizmoMode::Rotate,
			2 => nimbus::ext::gizmo::GizmoMode::Scale,
			_ => nimbus::ext::gizmo::GizmoMode::Translate,
		};
		tasks_gizmo_mode.push_tree(move |tree| {
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
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
			1 => nimbus::ext::gizmo::GizmoSpace::Local,
			_ => nimbus::ext::gizmo::GizmoSpace::World,
		};
		tasks_gizmo_space.push_tree(move |tree| {
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
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
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
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
			let cam = nimbus::ext::gizmo::GizmoCamera::from_orbit(
				nimbus::ext::gizmo::Vec3::ZERO,
				5.0,
				0.65,
				0.45,
				nimbus::ext::gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
			{
				gizmo.transform = nimbus::ext::gizmo::Transform::IDENTITY;
				gizmo.animate_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus::ext::gizmo::OrientationGizmo>(id)
			{
				orient.animate_camera(cam);
			}
			if let Some(id) = info
				&& let Some(text) = tree.get_widget_mut::<nimbus::ui::Text>(id)
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
				.and_then(|id| tree.get_widget::<nimbus::ext::gizmo::TransformGizmo>(id))
				.map(|gizmo| {
					let p = gizmo.transform.translation;
					let s = gizmo.transform.scale;
					format!(
						"Pos: ({:.2}, {:.2}, {:.2})  |  Scale: ({:.2}, {:.2}, {:.2})",
						p.x, p.y, p.z, s.x, s.y, s.z
					)
				});
			if let (Some(id), Some(label)) = (info, label)
				&& let Some(text) = tree.get_widget_mut::<nimbus::ui::Text>(id)
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
			"Right" => nimbus::ext::gizmo::CameraViewPreset::Right.orbit_angles(),
			"Left" => nimbus::ext::gizmo::CameraViewPreset::Left.orbit_angles(),
			"Top" => nimbus::ext::gizmo::CameraViewPreset::Top.orbit_angles(),
			"Bottom" => nimbus::ext::gizmo::CameraViewPreset::Bottom.orbit_angles(),
			"Back" => nimbus::ext::gizmo::CameraViewPreset::Back.orbit_angles(),
			_ => nimbus::ext::gizmo::CameraViewPreset::Front.orbit_angles(),
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
			let cam = nimbus::ext::gizmo::GizmoCamera::from_orbit(
				nimbus::ext::gizmo::Vec3::ZERO,
				5.0,
				yaw,
				pitch,
				nimbus::ext::gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
			{
				gizmo.animate_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus::ext::gizmo::OrientationGizmo>(id)
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
			let cam = nimbus::ext::gizmo::GizmoCamera::from_orbit(
				nimbus::ext::gizmo::Vec3::ZERO,
				5.0,
				yaw,
				pitch,
				nimbus::ext::gizmo::Rad::new(std::f32::consts::FRAC_PI_4),
				aspect,
			);
			if let Some(id) = tg
				&& let Some(gizmo) = tree.get_widget_mut::<nimbus::ext::gizmo::TransformGizmo>(id)
			{
				gizmo.set_camera(cam);
			}
			if let Some(id) = og
				&& let Some(orient) = tree.get_widget_mut::<nimbus::ext::gizmo::OrientationGizmo>(id)
			{
				orient.set_camera(cam);
			}
		});
	});

	let theme_palette = Rc::new(Cell::new(0i32));
	let theme_is_dark = Rc::new(Cell::new(true));

	let app_tasks_pal = app_tasks.clone();
	let pal_cb = theme_palette.clone();
	let dark_for_pal = theme_is_dark.clone();
	events.register("ChangePalette", move |idx: i32, _label: String| {
		pal_cb.set(idx);
		let dark = dark_for_pal.get();
		app_tasks_pal.push_theme(move |theme| {
			theme.monet = make_monet(idx, dark);
		});
	});

	let app_tasks_mode = app_tasks.clone();
	let pal_for_mode = theme_palette.clone();
	let dark_cb = theme_is_dark.clone();
	events.register("ChangeThemeMode", move |idx: i32, _label: String| {
		let dark = idx == 0;
		dark_cb.set(dark);
		let pal = pal_for_mode.get();
		app_tasks_mode.push_theme(move |theme| {
			theme.monet = make_monet(pal, dark);
		});
	});

	let app_tasks_radius = app_tasks.clone();
	events.register("ChangeRadius", move |idx: i32, _label: String| {
		let radius = match idx {
			0 => RadiusTokens {
				none: 0.0,
				xsmall: 0.0,
				small: 0.0,
				medium: 0.0,
				large: 0.0,
				xlarge: 0.0,
				full: 0.0,
			},
			1 => RadiusTokens {
				none: 0.0,
				xsmall: 0.0,
				small: 2.0,
				medium: 4.0,
				large: 6.0,
				xlarge: 10.0,
				full: f32::MAX,
			},
			3 => RadiusTokens {
				none: 0.0,
				xsmall: 8.0,
				small: 14.0,
				medium: 20.0,
				large: 26.0,
				xlarge: 36.0,
				full: f32::MAX,
			},
			_ => RadiusTokens::default(),
		};
		app_tasks_radius.push_theme(move |theme| {
			theme.radius = radius;
		});
	});

	let app_tasks_stroke = app_tasks.clone();
	events.register("ChangeStroke", move |idx: i32, _label: String| {
		let stroke = match idx {
			0 => StrokeTokens {
				thin: 0.5,
				thick: 1.0,
				heavier: 1.5,
			},
			2 => StrokeTokens {
				thin: 2.0,
				thick: 3.5,
				heavier: 5.0,
			},
			_ => StrokeTokens::default(),
		};
		app_tasks_stroke.push_theme(move |theme| {
			theme.stroke = stroke;
		});
	});

	let tasks_dock_close = tasks.clone();
	events.register("OnDockPaneClosed", move |title: String| {
		tasks_dock_close.push_tree(move |tree| {
			nimbus::ui::Toast::builder()
				.title("Panel Kapatıldı")
				.message(format!("'{}' paneli kapatıldı.", title))
				.kind(nimbus::ui::ToastKind::Info)
				.show(tree);
		});
	});

	let tasks_tl_play = tasks.clone();
	let tl_view_play = tl_view_id.clone();
	let tl_btn_play = tl_play_btn_id.clone();
	events.register("TimelineTogglePlay", move || {
		let vid = tl_view_play.get();
		let bid = tl_btn_play.get();
		tasks_tl_play.push_tree(move |tree| {
			let mut now_playing = false;
			if let Some(id) = vid
				&& let Some(tl) = tree.get_widget_mut::<nimbus::ext::timeline::TimelineView>(id)
			{
				now_playing = !tl.playing;
				tl.set_playing(now_playing);
			}
			if let Some(id) = bid
				&& let Some(btn) = tree.get_widget_mut::<nimbus::ui::Button>(id)
			{
				btn.label = if now_playing { "Pause" } else { "Play" }.to_string();
			}
		});
	});

	let tasks_tl_stop = tasks.clone();
	let tl_view_stop = tl_view_id.clone();
	let tl_curve_stop = tl_curve_id.clone();
	let tl_btn_stop = tl_play_btn_id.clone();
	let tl_ro_stop = tl_readout_id.clone();
	events.register("TimelineStop", move || {
		let vid = tl_view_stop.get();
		let cid = tl_curve_stop.get();
		let bid = tl_btn_stop.get();
		let rid = tl_ro_stop.get();
		tasks_tl_stop.push_tree(move |tree| {
			let mut summary = None;
			if let Some(id) = vid
				&& let Some(tl) = tree.get_widget_mut::<nimbus::ext::timeline::TimelineView>(id)
			{
				tl.set_playing(false);
				tl.set_playhead(0.0);
				let x = tl.tracks.first().map(|tr| tr.sample(0.0)).unwrap_or(0.0);
				let y = tl.tracks.get(1).map(|tr| tr.sample(0.0)).unwrap_or(0.0);
				summary = Some(format!("t = 0.00s  |  Pos X: {:.2}  |  Pos Y: {:.2}", x, y));
			}
			if let Some(id) = cid
				&& let Some(curve) = tree.get_widget_mut::<nimbus::ext::timeline::CurveEditor>(id)
			{
				curve.playhead = 0.0;
			}
			if let Some(id) = bid
				&& let Some(btn) = tree.get_widget_mut::<nimbus::ui::Button>(id)
			{
				btn.label = "Play".to_string();
			}
			if let (Some(id), Some(txt)) = (rid, summary)
				&& let Some(label) = tree.get_widget_mut::<nimbus::ui::Text>(id)
			{
				label.text = txt;
			}
		});
	});

	let tasks_tl_snap = tasks.clone();
	let tl_view_snap = tl_view_id.clone();
	let tl_curve_snap = tl_curve_id.clone();
	events.register("TimelineSnap", move |enabled: bool| {
		let vid = tl_view_snap.get();
		let cid = tl_curve_snap.get();
		tasks_tl_snap.push_tree(move |tree| {
			if let Some(id) = vid
				&& let Some(tl) = tree.get_widget_mut::<nimbus::ext::timeline::TimelineView>(id)
			{
				tl.snap = enabled;
			}
			if let Some(id) = cid
				&& let Some(curve) = tree.get_widget_mut::<nimbus::ext::timeline::CurveEditor>(id)
			{
				curve.snap = enabled;
			}
		});
	});

	let tasks_tl_time = tasks.clone();
	let tl_view_time = tl_view_id.clone();
	let tl_curve_time = tl_curve_id.clone();
	let tl_ro_time = tl_readout_id.clone();
	events.register("TimelineTimeChanged", move |t: f32| {
		let vid = tl_view_time.get();
		let cid = tl_curve_time.get();
		let rid = tl_ro_time.get();
		tasks_tl_time.push_tree(move |tree| {
			let summary = vid
				.and_then(|id| tree.get_widget::<nimbus::ext::timeline::TimelineView>(id))
				.map(|tl| {
					let x = tl.tracks.first().map(|tr| tr.sample(t)).unwrap_or(0.0);
					let y = tl.tracks.get(1).map(|tr| tr.sample(t)).unwrap_or(0.0);
					format!("t = {:.2}s  |  Pos X: {:.2}  |  Pos Y: {:.2}", t, x, y)
				});
			if let Some(id) = cid
				&& let Some(curve) = tree.get_widget_mut::<nimbus::ext::timeline::CurveEditor>(id)
			{
				curve.playhead = t;
			}
			if let (Some(id), Some(txt)) = (rid, summary)
				&& let Some(label) = tree.get_widget_mut::<nimbus::ui::Text>(id)
			{
				label.text = txt;
			}
		});
	});

	let tasks_tl_kf = tasks.clone();
	let tl_view_kf = tl_view_id.clone();
	let tl_curve_kf = tl_curve_id.clone();
	events.register("TimelineKeyframeChanged", move |_track: i32, _time: f32| {
		let vid = tl_view_kf.get();
		let cid = tl_curve_kf.get();
		tasks_tl_kf.push_tree(move |tree| {
			let tracks = vid
				.and_then(|id| tree.get_widget::<nimbus::ext::timeline::TimelineView>(id))
				.map(|tl| tl.tracks.clone());
			if let (Some(id), Some(tracks)) = (cid, tracks)
				&& let Some(curve) = tree.get_widget_mut::<nimbus::ext::timeline::CurveEditor>(id)
			{
				curve.tracks = tracks;
			}
		});
	});

	let tasks_tl_curve = tasks.clone();
	let tl_view_curve = tl_view_id.clone();
	let tl_curve_curve = tl_curve_id.clone();
	events.register("TimelineCurveChanged", move || {
		let vid = tl_view_curve.get();
		let cid = tl_curve_curve.get();
		tasks_tl_curve.push_tree(move |tree| {
			let tracks = cid
				.and_then(|id| tree.get_widget::<nimbus::ext::timeline::CurveEditor>(id))
				.map(|c| c.tracks.clone());
			if let (Some(id), Some(tracks)) = (vid, tracks)
				&& let Some(tl) = tree.get_widget_mut::<nimbus::ext::timeline::TimelineView>(id)
			{
				tl.tracks = tracks;
			}
		});
	});

	let tasks_dnd_drop = tasks.clone();
	events.register("OnDndDrop", move |kind: String, value: String| {
		tasks_dnd_drop.push_tree(move |tree| {
			nimbus::ui::Toast::builder()
				.title(format!("Dropped [{}]", kind))
				.message(value)
				.kind(nimbus::ui::ToastKind::Success)
				.show(tree);
		});
	});

	let tasks_dnd_clear = tasks.clone();
	let z_shader = dnd_zone_shader_id.clone();
	let z_any = dnd_zone_any_id.clone();
	let z_files = dnd_zone_files_id.clone();
	events.register("DndClearZones", move || {
		let ids = [z_shader.get(), z_any.get(), z_files.get()];
		tasks_dnd_clear.push_tree(move |tree| {
			for id in ids.into_iter().flatten() {
				if let Some(zone) = tree.get_widget_mut::<nimbus::ui::DropZone>(id) {
					zone.clear();
				}
			}
			tree.needs_layout = true;
			tree.needs_redraw = true;
		});
	});

	let scene = nimbus::qdl::load_into_tree(
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
	page_dock_id.set(scene.named_nodes.get("page_dock").copied());
	page_timeline_id.set(scene.named_nodes.get("page_timeline").copied());
	page_inspector_id.set(scene.named_nodes.get("page_inspector").copied());
	page_dnd_id.set(scene.named_nodes.get("page_dnd").copied());
	dnd_zone_shader_id.set(scene.named_nodes.get("dnd_zone_shader").copied());
	dnd_zone_any_id.set(scene.named_nodes.get("dnd_zone_any").copied());
	dnd_zone_files_id.set(scene.named_nodes.get("dnd_zone_files").copied());
	tl_view_id.set(scene.named_nodes.get("demo_timeline").copied());
	tl_curve_id.set(scene.named_nodes.get("demo_curve").copied());
	tl_play_btn_id.set(scene.named_nodes.get("tl_play_btn").copied());
	tl_readout_id.set(scene.named_nodes.get("tl_readout").copied());

	// Seed example tracks for the Timeline + CurveEditor demo
	let demo_tracks = vec![
		nimbus::ext::timeline::TimelineTrack::new("Position X")
			.with_keyframes(vec![
				nimbus::ext::timeline::Keyframe::new(0.0, -0.8)
					.with_tangents(nimbus::math::Vec2::new(-0.3, 0.0), nimbus::math::Vec2::new(0.6, 0.2)),
				nimbus::ext::timeline::Keyframe::new(2.0, 1.0)
					.with_tangents(nimbus::math::Vec2::new(-0.5, 0.1), nimbus::math::Vec2::new(0.5, -0.1)),
				nimbus::ext::timeline::Keyframe::new(4.2, -0.4)
					.with_tangents(nimbus::math::Vec2::new(-0.4, -0.2), nimbus::math::Vec2::new(0.5, 0.2)),
				nimbus::ext::timeline::Keyframe::new(6.0, 0.6),
			]),
		nimbus::ext::timeline::TimelineTrack::new("Position Y")
			.with_keyframes(vec![
				nimbus::ext::timeline::Keyframe::new(0.0, 0.2).with_interp(nimbus::ext::timeline::KeyframeInterp::Linear),
				nimbus::ext::timeline::Keyframe::new(1.5, -0.9).with_interp(nimbus::ext::timeline::KeyframeInterp::Bezier),
				nimbus::ext::timeline::Keyframe::new(3.5, 0.85).with_interp(nimbus::ext::timeline::KeyframeInterp::Linear),
				nimbus::ext::timeline::Keyframe::new(5.5, 0.0),
			]),
		nimbus::ext::timeline::TimelineTrack::new("Camera Cut")
			.with_clips(vec![
				nimbus::ext::timeline::TimelineClip::new(0.2, 2.2, "Intro Shot"),
				nimbus::ext::timeline::TimelineClip::new(2.7, 2.8, "Hero Orbit"),
			]),
	];
	if let Some(id) = tl_view_id.get()
		&& let Some(tl) = main_window
			.tree_mut()
			.get_widget_mut::<nimbus::ext::timeline::TimelineView>(id)
	{
		tl.tracks = demo_tracks.clone();
	}
	if let Some(id) = tl_curve_id.get()
		&& let Some(curve) = main_window
			.tree_mut()
			.get_widget_mut::<nimbus::ext::timeline::CurveEditor>(id)
	{
		curve.tracks = demo_tracks;
	}
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
		.get_widget_mut::<nimbus::ext::purple_print::PurplePrintView>(pp_id)
	{
		pp.connect(ev, 0, add, 0);
		pp.connect(var_a, 0, add, 1);
	}

	
	// Run Application
	if let Err(e) = app.run() {
		nimbus_error!("Application exited with error: {}", e);
	}
}
