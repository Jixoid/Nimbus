/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod button;
pub mod card;
pub mod check;
pub mod choice;
pub mod edit;
pub mod flex;
pub mod flow;
pub mod interval;
pub mod multi_progress;
pub mod popup;
pub mod progress;
pub mod radio;
pub mod slider;
pub mod scroll;
pub mod spacer;
pub mod stack;
pub mod text;
pub mod switch;
pub mod tabs;
pub mod tooltip;

pub use button::{Button, ButtonVariant};
pub use card::Card;
pub use check::Check;
pub use choice::Choice;
pub use edit::Edit;
pub use flex::{HBox, VBox};
pub use flow::Flow;
pub use interval::Interval;
pub use multi_progress::{MultiProgressBar, ProgressSegment};
pub use popup::{Popup, PopupItem};
pub use progress::ProgressBar;
pub use radio::Radio;
pub use scroll::{ScrollState, ScrollView};
pub use slider::Slider;
pub use spacer::Spacer;
pub use stack::Stack;
pub use text::Text;
pub use switch::Switch;
pub use tabs::Tabs;
pub use tooltip::{ToolTip, Tooltip, TooltipPosition};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("qstd::button", || Box::new(Button::default()));
	registry.register("qstd::card", || Box::new(Card::default()));
	registry.register("qstd::edit", || Box::new(Edit::default()));
	registry.register("qstd::text", || Box::new(Text::default()));
	registry.register("qstd::vbox", || Box::new(VBox::default()));
	registry.register("qstd::flow", || Box::new(Flow::default()));
	registry.register("qstd::hbox", || Box::new(HBox::default()));
	registry.register("qstd::progressbar", || Box::new(ProgressBar::default()));
	registry.register("qstd::check", || Box::new(Check::default()));
	registry.register("qstd::radio", || Box::new(Radio::default()));
	registry.register("qstd::switch", || Box::new(Switch::default()));
	registry.register("qstd::slider", || Box::new(Slider::default()));
	registry.register("qstd::interval", || Box::new(Interval::default()));
	registry.register("qstd::multiprogressbar", || Box::new(MultiProgressBar::default()));
	registry.register("qstd::tabs", || Box::new(Tabs::default()));
	registry.register("qstd::choice", || Box::new(Choice::default()));
	registry.register("qstd::scroll", || Box::new(ScrollView::default()));
	registry.register("qstd::spacer", || Box::new(Spacer::default()));
	registry.register("qstd::stack", || Box::new(Stack::default()));
	registry.register("qstd::popup", || Box::new(Popup::default()));
	registry.register("qstd::tooltip", || Box::new(Tooltip::default()));
}
