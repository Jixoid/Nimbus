/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod anim;
pub mod app;
pub mod draw;
pub mod event;
pub mod id;
pub mod layout;
pub mod method;
pub mod node;
pub mod property;
pub mod registry;
pub mod tree;
pub mod widget;
pub mod window;

pub use anim::{Animated, Easing, Interpolate, Spring};
pub use app::Application;
pub use draw::DrawCtx;
pub use event::{
	ButtonState, EventCtx, EventResponse, InputEvent, Message, MessageTarget, MouseButton,
};
pub use id::{ObjectID, WindowHandle};
pub use layout::{BoxConstraints, LayoutCtx};
pub use method::{ActionCallback, EventRegistry, MethodCallback, MethodTable};
pub use node::NodeData;
pub use property::{parse_color, PropertyCtx, PropertyError, PropertyValue};
pub use registry::{create_widget, register_global_widget, WidgetFactory, WidgetRegistry};
pub use nimbus_render2d::Canvas2D;
pub use tree::UiTree;
pub use widget::{AsAny, Widget};
pub use window::Window;
