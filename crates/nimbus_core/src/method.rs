/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use nimbus_math::{Color, Vec2};

use crate::property::PropertyValue;


pub trait FromMethodArg: Sized {
	fn from_method_args(args: &[PropertyValue]) -> Option<Self> {
		args.first().and_then(Self::from_single_value)
	}

	fn from_single_value(val: &PropertyValue) -> Option<Self>;
}

impl FromMethodArg for bool {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(b) = val.as_bool() {
			return Some(b);
		}
		if let Some(n) = val.as_f64() {
			return Some(n != 0.0);
		}
		if let Some(s) = val.as_str() {
			return match s.trim() {
				"true" | "1" => Some(true),
				"false" | "0" => Some(false),
				_ => None,
			};
		}
		None
	}
}

impl FromMethodArg for f32 {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f32() {
			return Some(n);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1.0 } else { 0.0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<f32>().ok();
		}
		None
	}
}

impl FromMethodArg for f64 {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f64() {
			return Some(n);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1.0 } else { 0.0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<f64>().ok();
		}
		None
	}
}

impl FromMethodArg for i32 {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f64() {
			return Some(n as i32);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1 } else { 0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<i32>().ok();
		}
		None
	}
}

impl FromMethodArg for i64 {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f64() {
			return Some(n as i64);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1 } else { 0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<i64>().ok();
		}
		None
	}
}

impl FromMethodArg for u32 {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f64() {
			return Some(n.max(0.0) as u32);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1 } else { 0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<u32>().ok();
		}
		None
	}
}

impl FromMethodArg for usize {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		if let Some(n) = val.as_f64() {
			return Some(n.max(0.0) as usize);
		}
		if let Some(b) = val.as_bool() {
			return Some(if b { 1 } else { 0 });
		}
		if let Some(s) = val.as_str() {
			return s.trim().parse::<usize>().ok();
		}
		None
	}
}

impl FromMethodArg for String {
	fn from_method_args(args: &[PropertyValue]) -> Option<Self> {
		if let Some(s) = args.iter().find_map(|v| v.as_str().map(str::to_string)) {
			return Some(s);
		}
		args.first().and_then(Self::from_single_value)
	}

	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		match val {
			PropertyValue::String(s) | PropertyValue::Ident(s) => Some(s.clone()),
			PropertyValue::Number(n) => Some(n.to_string()),
			PropertyValue::Bool(b) => Some(b.to_string()),
			_ => None,
		}
	}
}

impl FromMethodArg for Vec2<f32> {
	fn from_method_args(args: &[PropertyValue]) -> Option<Self> {
		if args.len() >= 2
			&& let (Some(x), Some(y)) = (
				f32::from_single_value(&args[0]),
				f32::from_single_value(&args[1]),
			) {
			return Some(Vec2::new(x, y));
		}
		args.first().and_then(Self::from_single_value)
	}

	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		val.as_vec2()
	}
}

impl FromMethodArg for (f32, f32) {
	fn from_method_args(args: &[PropertyValue]) -> Option<Self> {
		if args.len() >= 2
			&& let (Some(a), Some(b)) = (
				f32::from_single_value(&args[0]),
				f32::from_single_value(&args[1]),
			) {
			return Some((a, b));
		}
		args.first().and_then(Self::from_single_value)
	}

	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		val.as_vec2().map(|v| (v.x, v.y))
	}
}

impl FromMethodArg for Color<f32> {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		val.as_color()
	}
}

impl FromMethodArg for PropertyValue {
	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		Some(val.clone())
	}
}

impl FromMethodArg for Vec<PropertyValue> {
	fn from_method_args(args: &[PropertyValue]) -> Option<Self> {
		Some(args.to_vec())
	}

	fn from_single_value(val: &PropertyValue) -> Option<Self> {
		val.as_array().map(|a| a.to_vec())
	}
}


pub struct NoArgs;
pub struct SingleArg<T>(PhantomData<fn(T)>);
pub struct TwoArgs<T1, T2>(PhantomData<fn(T1, T2)>);
pub struct RefStrArg;
pub struct RefPropArg;
pub struct RefSliceArg;
pub struct CallbackArg;

pub type SharedMethodFn = Rc<RefCell<dyn FnMut(&[PropertyValue]) + 'static>>;

pub trait IntoMethod<Marker = NoArgs> {
	fn into_method_rc(self) -> SharedMethodFn;
}

impl<F> IntoMethod<NoArgs> for F
where
	F: FnMut() + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |_args: &[PropertyValue]| {
			self();
		}))
	}
}

impl<T, F> IntoMethod<SingleArg<T>> for F
where
	T: FromMethodArg + 'static,
	F: FnMut(T) + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |args: &[PropertyValue]| {
			if let Some(arg) = T::from_method_args(args) {
				self(arg);
			}
		}))
	}
}

impl<T1, T2, F> IntoMethod<TwoArgs<T1, T2>> for F
where
	T1: FromMethodArg + 'static,
	T2: FromMethodArg + 'static,
	F: FnMut(T1, T2) + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |args: &[PropertyValue]| {
			let pair = if args.len() >= 2 {
				T1::from_single_value(&args[0]).zip(T2::from_single_value(&args[1]))
			} else if args.len() == 1 {
				if let Some(v) = args[0].as_vec2() {
					let p0 = PropertyValue::from(v.x);
					let p1 = PropertyValue::from(v.y);
					T1::from_single_value(&p0).zip(T2::from_single_value(&p1))
				} else {
					None
				}
			} else {
				None
			};

			if let Some((a, b)) = pair {
				self(a, b);
			}
		}))
	}
}

impl<F> IntoMethod<RefStrArg> for F
where
	F: FnMut(&str) + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |args: &[PropertyValue]| {
			if let Some(s) = args.iter().find_map(|v| v.as_str()) {
				self(s);
			}
		}))
	}
}

impl<F> IntoMethod<RefPropArg> for F
where
	F: FnMut(&PropertyValue) + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |args: &[PropertyValue]| {
			if let Some(first) = args.first() {
				self(first);
			}
		}))
	}
}

impl<F> IntoMethod<RefSliceArg> for F
where
	F: FnMut(&[PropertyValue]) + 'static,
{
	fn into_method_rc(mut self) -> SharedMethodFn {
		Rc::new(RefCell::new(move |args: &[PropertyValue]| {
			self(args);
		}))
	}
}

impl IntoMethod<CallbackArg> for MethodCallback {
	fn into_method_rc(self) -> SharedMethodFn {
		self.inner
	}
}


#[derive(Clone)]
pub struct MethodCallback {
	inner: SharedMethodFn,
}

impl MethodCallback {
	pub fn new<M, F: IntoMethod<M>>(callback: F) -> Self {
		Self {
			inner: callback.into_method_rc(),
		}
	}

	pub fn call(&self, args: &[PropertyValue]) {
		if let Ok(mut cb) = self.inner.try_borrow_mut() {
			cb(args);
		}
	}

	pub fn call0(&self) {
		self.call(&[]);
	}

	pub fn call1(&self, arg: impl Into<PropertyValue>) {
		self.call(&[arg.into()]);
	}

	pub fn call2(&self, arg1: impl Into<PropertyValue>, arg2: impl Into<PropertyValue>) {
		self.call(&[arg1.into(), arg2.into()]);
	}
}

pub type ActionCallback = MethodCallback;

#[derive(Default)]
pub struct MethodTable {
	methods: HashMap<String, MethodCallback>,
}

impl MethodTable {
	pub fn new() -> Self {
		Self {
			methods: HashMap::new(),
		}
	}

	pub fn register<M, F>(&mut self, name: impl Into<String>, callback: F) -> &mut Self
	where
		F: IntoMethod<M>,
	{
		self.methods.insert(name.into(), MethodCallback::new(callback));
		self
	}

	pub fn get(&self, name: &str) -> Option<MethodCallback> {
		self.methods.get(name).cloned()
	}

	pub fn take(&mut self, name: &str) -> Option<MethodCallback> {
		self.methods.remove(name)
	}

	pub fn call(&self, name: &str, args: &[PropertyValue]) -> bool {
		if let Some(cb) = self.methods.get(name) {
			cb.call(args);
			true
		} else {
			false
		}
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


#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::{Arc, Mutex};

	#[test]
	fn test_method_table_multi_value_callbacks() {
		let mut table = MethodTable::new();
		let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

		{
			let log = log.clone();
			table.register("zero", move || {
				log.lock().unwrap().push("zero".to_string());
			});
		}
		{
			let log = log.clone();
			table.register("bool_cb", move |b: bool| {
				log.lock().unwrap().push(format!("bool:{b}"));
			});
		}
		{
			let log = log.clone();
			table.register("f32_cb", move |v: f32| {
				log.lock().unwrap().push(format!("f32:{v:.1}"));
			});
		}
		{
			let log = log.clone();
			table.register("i32_cb", move |v: i32| {
				log.lock().unwrap().push(format!("i32:{v}"));
			});
		}
		{
			let log = log.clone();
			table.register("usize_cb", move |v: usize| {
				log.lock().unwrap().push(format!("usize:{v}"));
			});
		}
		{
			let log = log.clone();
			table.register("string_cb", move |s: String| {
				log.lock().unwrap().push(format!("string:{s}"));
			});
		}
		{
			let log = log.clone();
			table.register("str_cb", move |s: &str| {
				log.lock().unwrap().push(format!("str:{s}"));
			});
		}
		{
			let log = log.clone();
			table.register("two_f32", move |a: f32, b: f32| {
				log.lock().unwrap().push(format!("two:{a:.1},{b:.1}"));
			});
		}
		{
			let log = log.clone();
			table.register("prop_cb", move |v: &PropertyValue| {
				log.lock().unwrap().push(format!("prop:{:?}", v));
			});
		}

		// Zero-arg callback still runs even when invoked with arguments
		table.get("zero").unwrap().call1(true);
		table.get("zero").unwrap().call2(10.0f32, 20.0f32);

		table.get("bool_cb").unwrap().call1(true);
		table.get("f32_cb").unwrap().call1(42.5f32);
		table.get("i32_cb").unwrap().call2(3i32, "Tab 3");
		table.get("usize_cb").unwrap().call2(2usize, "Tab 2");
		table.get("string_cb").unwrap().call2(1usize, "Settings");
		table.get("str_cb").unwrap().call2(0usize, "Home");
		table.get("two_f32").unwrap().call2(25.0f32, 75.0f32);
		table.get("prop_cb").unwrap().call1("hello");

		let recorded = log.lock().unwrap().clone();
		assert_eq!(
			recorded,
			vec![
				"zero",
				"zero",
				"bool:true",
				"f32:42.5",
				"i32:3",
				"usize:2",
				"string:Settings",
				"str:Home",
				"two:25.0,75.0",
				"prop:String(\"hello\")",
			]
		);
	}
}
