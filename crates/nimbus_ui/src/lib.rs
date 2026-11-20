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
pub mod edit;
pub mod flex;
pub mod flow;
pub mod interval;
pub mod multi_progress;
pub mod progress;
pub mod radio;
pub mod slider;
pub mod text;
pub mod switch;

pub use button::{Button, ButtonVariant};
pub use card::Card;
pub use check::Check;
pub use edit::Edit;
pub use flex::{HBox, VBox};
pub use flow::Flow;
pub use interval::Interval;
pub use multi_progress::{MultiProgressBar, ProgressSegment};
pub use progress::ProgressBar;
pub use radio::Radio;
pub use slider::Slider;
pub use text::Text;
pub use switch::Switch;

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("qstd::button", || Box::new(Button::default()));
	registry.register("qstd::card", || Box::new(Card::default()));
	registry.register("qstd::edit", || Box::new(Edit::default()));
	registry.register("qstd::text", || Box::new(Text::default()));
	registry.register("qstd::vbox", || Box::new(VBox::default()));
	registry.register("qstd::flow", || Box::new(Flow::default()));
	registry.register("qstd::hbox", || Box::new(HBox::default()));
	registry.register("qstd::progbar", || Box::new(ProgressBar::default()));
	registry.register("qstd::check", || Box::new(Check::default()));
	registry.register("qstd::radio", || Box::new(Radio::default()));
	registry.register("qstd::switch", || Box::new(Switch::default()));
	registry.register("qstd::slider", || Box::new(Slider::default()));
	registry.register("qstd::interval", || Box::new(Interval::default()));
	registry.register("qstd::multi_progbar", || Box::new(MultiProgressBar::default()));
	registry.register("qstd::multiprogress", || Box::new(MultiProgressBar::default()));
}
