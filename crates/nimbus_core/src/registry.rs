/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::widget::Widget;


pub type WidgetFactory = Box<dyn Fn() -> Box<dyn Widget> + Send + Sync>;

#[derive(Default)]
pub struct WidgetRegistry {
	factories: HashMap<String, WidgetFactory>,
}

impl WidgetRegistry {
	pub fn new() -> Self {
		Self {
			factories: HashMap::new(),
		}
	}

	pub fn register<F>(&mut self, name: impl Into<String>, factory: F) -> &mut Self
	where
		F: Fn() -> Box<dyn Widget> + Send + Sync + 'static,
	{
		self.factories.insert(name.into(), Box::new(factory));
		self
	}

	pub fn create(&self, name: &str) -> Option<Box<dyn Widget>> {
		self.factories.get(name).map(|f| f())
	}

	pub fn contains(&self, name: &str) -> bool {
		self.factories.contains_key(name)
	}

	pub fn len(&self) -> usize {
		self.factories.len()
	}

	pub fn is_empty(&self) -> bool {
		self.factories.is_empty()
	}

	pub fn global() -> &'static RwLock<WidgetRegistry> {
		static GLOBAL: OnceLock<RwLock<WidgetRegistry>> = OnceLock::new();
		GLOBAL.get_or_init(|| RwLock::new(WidgetRegistry::new()))
	}
}

pub fn register_global_widget<F>(name: impl Into<String>, factory: F)
where
	F: Fn() -> Box<dyn Widget> + Send + Sync + 'static,
{
	if let Ok(mut reg) = WidgetRegistry::global().write() {
		reg.register(name, factory);
	}
}

pub fn create_widget(name: &str) -> Option<Box<dyn Widget>> {
	if let Ok(reg) = WidgetRegistry::global().read() {
		reg.create(name)
	} else {
		None
	}
}
