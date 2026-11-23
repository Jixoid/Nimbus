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
pub mod collapsible;
pub mod combobox;
pub mod dialog;
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
pub mod spinbox;
pub mod spinner;
pub mod stack;
pub mod text;
pub mod switch;
pub mod tabs;
pub mod toast;
pub mod tooltip;
pub mod viewport;

pub use button::{Button, ButtonBuilder, ButtonVariant};
pub use card::{Card, CardBuilder};
pub use check::{Check, CheckBuilder};
pub use choice::{Choice, ChoiceBuilder};
pub use collapsible::{Collapsible, CollapsibleBuilder};
pub use combobox::{ComboBox, ComboBoxBuilder};
pub use dialog::{Dialog, DialogBuilder};
pub use edit::{Edit, EditBuilder};
pub use flex::{HBox, HBoxBuilder, VBox, VBoxBuilder};
pub use flow::{Flow, FlowBuilder};
pub use interval::{Interval, IntervalBuilder};
pub use multi_progress::{MultiProgressBar, MultiProgressBarBuilder, ProgressSegment};
pub use popup::{Popup, PopupBuilder, PopupItem, PopupItemBuilder};
pub use progress::{ProgressBar, ProgressBarBuilder};
pub use radio::{Radio, RadioBuilder};
pub use scroll::{ScrollState, ScrollView, ScrollViewBuilder};
pub use slider::{Slider, SliderBuilder};
pub use spacer::{Spacer, SpacerBuilder};
pub use spinbox::{FloatSpinBox, FloatSpinBoxBuilder, SpinBox, SpinBoxBuilder};
pub use spinner::{Spinner, SpinnerBuilder};
pub use stack::{Stack, StackBuilder};
pub use text::{Text, TextBuilder};
pub use switch::{Switch, SwitchBuilder};
pub use tabs::{Tabs, TabsBuilder};
pub use toast::{Toast, ToastBuilder, ToastHost, ToastKind, ToastPosition};
pub use tooltip::{ToolTip, ToolTipBuilder, Tooltip, TooltipBuilder, TooltipPosition};
pub use viewport::{Viewport, ViewportBuilder};

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
	registry.register("qstd::collapsible", || Box::new(Collapsible::default()));
	registry.register("qstd::combobox", || Box::new(ComboBox::default()));
	registry.register("qstd::popup", || Box::new(Popup::default()));
	registry.register("qstd::tooltip", || Box::new(Tooltip::default()));
	registry.register("qstd::viewport", || Box::new(Viewport::default()));
	registry.register("qstd::spinbox", || Box::new(SpinBox::default()));
	registry.register("qstd::floatspinbox", || Box::new(FloatSpinBox::default()));
	registry.register("qstd::spinner", || Box::new(Spinner::default()));
	registry.register("qstd::dialog", || Box::new(Dialog::default()));
}
