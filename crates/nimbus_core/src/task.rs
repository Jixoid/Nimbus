/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::tree::UiTree;
use crate::window::Window;


pub enum Task {
	Plain(Box<dyn FnOnce() + Send + 'static>),
	Tree(Box<dyn FnOnce(&mut UiTree) + Send + 'static>),
	Window(Box<dyn FnOnce(&mut Window) + Send + 'static>),
}

pub struct NoArgTask;
pub struct TreeTask;
pub struct WindowTask;

pub trait IntoTask<Marker = NoArgTask> {
	fn into_task(self) -> Task;
}

impl<F> IntoTask<NoArgTask> for F
where
	F: FnOnce() + Send + 'static,
{
	fn into_task(self) -> Task {
		Task::Plain(Box::new(self))
	}
}

impl<F> IntoTask<TreeTask> for F
where
	F: FnOnce(&mut UiTree) + Send + 'static,
{
	fn into_task(self) -> Task {
		Task::Tree(Box::new(self))
	}
}

impl<F> IntoTask<WindowTask> for F
where
	F: FnOnce(&mut Window) + Send + 'static,
{
	fn into_task(self) -> Task {
		Task::Window(Box::new(self))
	}
}

#[derive(Clone, Default)]
pub struct TaskQueue {
	queue: Arc<Mutex<VecDeque<Task>>>,
}

pub type TaskSender = TaskQueue;

impl TaskQueue {
	pub fn new() -> Self {
		Self {
			queue: Arc::new(Mutex::new(VecDeque::new())),
		}
	}

	pub fn push<M, F: IntoTask<M>>(&self, task: F) {
		if let Ok(mut guard) = self.queue.lock() {
			guard.push_back(task.into_task());
		}
	}

	pub fn push_fn<F: FnOnce() + Send + 'static>(&self, task: F) {
		self.push::<NoArgTask, _>(task);
	}

	pub fn push_tree<F: FnOnce(&mut UiTree) + Send + 'static>(&self, task: F) {
		self.push::<TreeTask, _>(task);
	}

	pub fn push_window<F: FnOnce(&mut Window) + Send + 'static>(&self, task: F) {
		self.push::<WindowTask, _>(task);
	}

	pub fn is_empty(&self) -> bool {
		self.queue.lock().map(|g| g.is_empty()).unwrap_or(true)
	}

	pub fn len(&self) -> usize {
		self.queue.lock().map(|g| g.len()).unwrap_or(0)
	}

	pub(crate) fn drain_all(&self) -> Vec<Task> {
		let mut guard = match self.queue.lock() {
			Ok(g) => g,
			Err(poisoned) => poisoned.into_inner(),
		};
		guard.drain(..).collect()
	}

	pub(crate) fn push_front_all(&self, tasks: impl IntoIterator<Item = Task>) {
		let items: Vec<Task> = tasks.into_iter().collect();
		if items.is_empty() {
			return;
		}
		let mut guard = match self.queue.lock() {
			Ok(g) => g,
			Err(poisoned) => poisoned.into_inner(),
		};
		for task in items.into_iter().rev() {
			guard.push_front(task);
		}
	}
}

// Keep PhantomData used if needed externally in generic helper signatures
#[allow(dead_code)]
pub(crate) struct MarkerRef<M>(PhantomData<fn() -> M>);
