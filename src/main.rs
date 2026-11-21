/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{Application, EventRegistry, WidgetRegistry, Window};
use nimbus_math::Color;
use nimbus_theme::{ElevationTokens, Monet, RadiusTokens, StateTokens, StrokeTokens, Theme};
use nimbus_except::*;


fn main() {
	nimbus_except::start();
	nimbus_ui::register_widgets(&mut WidgetRegistry::global().write().unwrap());

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

	let qdl_tool = r#"
	styles {
		tool_header {
			font_size: 16
		}
	}

	root qstd::hbox {
		spacing: 10
		paddings: 16
		color: "monet(SurfaceContainerLow)"

		node qstd::vbox {
			spacing: 10
			paddings: 16
			color: "monet(SurfaceContainer)"
			corner_radius: 16
			scrollable: true

			node qstd::text {
				style: "tool_header"
				text: "Nimbus Application"
				color: "monet(PrimaryFixed)"
				font_size: 25
			}

			node qstd::radio {
				text: "Radio 1"
				checked: false
			}
			node qstd::radio {
				text: "Radio 2"
				checked: true
			}
				
			node qstd::check {
				text: "Check 1"
				checked: false
			}
			node qstd::check {
				text: "Check 2"
				checked: true
			}
			
			node qstd::switch {
				text: "Switch 1"
				checked: false
			}
			node qstd::switch {
				text: "Switch 2"
				checked: true
			}
			
			node qstd::button {
				text: "filled"
				variant: "filled"
			}
			node qstd::button {
				text: "tonal"
				variant: "tonal"
			}
			node qstd::button {
				text: "elevated"
				variant: "elevated"
			}
			node qstd::button {
				text: "outlined"
				variant: "outlined"
			}
		}

		node qstd::vbox {
			spacing: 10
			paddings: 16
			color: "monet(SurfaceContainer)"
			corner_radius: 16

			node qstd::edit {
				text: "Edit 1"
				title: "edit 1"
			}
			node qstd::edit {
				text: "Edit 2"
			}

			node qstd::slider {
				value: 45
				min: 0
				max: 100
			}
			node qstd::interval {
				pos1: 25
				pos2: 75
				min: 0
				max: 100
			}

			node qstd::multiprogress {
				values: [40, 30, 20]
			}

			node qstd::choice {
				tabs: ["cho1", "cho2", "cho3"]
			}
			node qstd::tabs {
				tabs: ["tab1", "tab2", "tab3"]
			}

			node qstd::popup {
				items: [
					"open",
					"save",
					"-",
					{
						text: "others",
						items: [
							"other 1"
							"other 2"
						],
					},
					
				]
			}
		}

	}
	"#;

	let mut events = EventRegistry::new();
	events.register("ToolAction", || {
		nimbus_disp!("QDL Event Callback!");
	});

	nimbus_qdl::load_into_tree(qdl_tool, main_window.tree_mut(), &theme, &mut events)
		.expect("Failed to load qdl window");

	
	// Run Application
	if let Err(e) = app.run() {
		nimbus_error!("Application exited with error: {}", e);
	}
}
