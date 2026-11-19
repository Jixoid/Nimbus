/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod button;
pub mod card;
pub mod flex;
pub mod progress;
pub mod text;

pub use button::Button;
pub use card::Card;
pub use flex::{HBox, VBox};
pub use progress::ProgressBar;
pub use text::Text;

use nimbus_core::WidgetRegistry;

pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("qstd::button", || Box::new(Button::default()));
	registry.register("qstd::card", || Box::new(Card::default()));
	registry.register("qstd::text", || Box::new(Text::default()));
	registry.register("qstd::vbox", || Box::new(VBox::default()));
	registry.register("qstd::hbox", || Box::new(HBox::default()));
	registry.register("qstd::progbar", || Box::new(ProgressBar::default()));
}
