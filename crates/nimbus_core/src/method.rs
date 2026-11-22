/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


pub type MethodCallback = Box<dyn FnMut() + 'static>;
pub type ActionCallback = MethodCallback;

#[derive(Default)]
pub struct MethodTable {
	methods: HashMap<String, Rc<RefCell<dyn FnMut() + 'static>>>,
}

impl MethodTable {
	pub fn new() -> Self {
		Self {
			methods: HashMap::new(),
		}
	}

	pub fn register<F>(&mut self, name: impl Into<String>, callback: F) -> &mut Self
	where
		F: FnMut() + 'static,
	{
		self.methods.insert(name.into(), Rc::new(RefCell::new(callback)));
		self
	}

	pub fn get(&self, name: &str) -> Option<MethodCallback> {
		let shared = self.methods.get(name)?.clone();
		Some(Box::new(move || {
			if let Ok(mut cb) = shared.try_borrow_mut() {
				cb();
			}
		}))
	}

	pub fn take(&mut self, name: &str) -> Option<MethodCallback> {
		let shared = self.methods.remove(name)?;
		Some(Box::new(move || {
			if let Ok(mut cb) = shared.try_borrow_mut() {
				cb();
			}
		}))
	}

	pub fn contains(&self, name: &str) -> bool {
		self.methods.contains_key(name)
	}

	pub fn is_empty(&self) -> bool {
		self.methods.is_empty()
	}

	pub fn len(&self) -> usize {
		self.methods.len()
	}
}

pub type EventRegistry = MethodTable;
