/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{Application, MethodTable, WidgetRegistry, Window};
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

	let mut events = MethodTable::new();
	events.register("ToolAction", || {
		nimbus_disp!("QDL Event Callback!");
	});

	nimbus_qdl::load_into_tree(include_str!("main.qdl"), main_window.tree_mut(), &theme, &mut events)
		.expect("Failed to load qdl window");

	
	// Run Application
	if let Err(e) = app.run() {
		nimbus_error!("Application exited with error: {}", e);
	}
}
