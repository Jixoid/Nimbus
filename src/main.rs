/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{Application, EventRegistry, WidgetRegistry, Window};
use nimbus_except::*;


fn main() {
	nimbus_except::start();
	nimbus_ui::register_widgets(&mut WidgetRegistry::global().write().unwrap());

	// Application
	let mut app = Application::new();
	let theme = app.theme.clone();

	// Window
	let main_handle = app.create_window(Window::new("Nimbus Desktop Engine", 860, 560));
	let main_window = app.window_mut(main_handle).unwrap();

	let qdl_tool = r#"
	styles {
		tool_header {
			font_size: 16
		}
	}

	root qstd::vbox {
		spacing: 12
		paddings: 16

		node qstd::text {
			style: "tool_header"
			text: "QDL Declarative Inspector"
			color: "monet(Primary)"
		}

		node qstd::text {
			text: "Loaded dynamically from QDL"
		}

		node qstd::progbar {
			value: 0.65
		}

		node qstd::button {
			text: "Trigger QDL Action"
			on_click: "ToolAction"
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
