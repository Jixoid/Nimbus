/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::{Rect, Vec2};
use crate::id::ObjectID;
use crate::layout::BoxConstraints;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Anchor {
	TopLeft,
	TopCenter,
	TopRight,
	CenterLeft,
	Center,
	CenterRight,
	BottomLeft,
	BottomCenter,
	BottomRight,
	Fill,
	TopFill,
	BottomFill,
	LeftFill,
	RightFill,
}

impl Anchor {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim() {
			"left-top"    => Some(Self::TopLeft),
			"left-bottom" => Some(Self::BottomLeft),
			
			"right-top"    => Some(Self::TopRight),
			"right-bottom" => Some(Self::BottomRight),
			
			"center" 				=> Some(Self::Center),
			"top-center"    => Some(Self::TopCenter),
			"bottom-center" => Some(Self::BottomCenter),
			"left-center"   => Some(Self::CenterLeft),
			"right-center"  => Some(Self::CenterRight),
			
			"fill"   								 => Some(Self::Fill),
			"top-fill" | "top"       => Some(Self::TopFill),
			"bottom-fill" | "bottom" => Some(Self::BottomFill),
			"left-fill" | "left"     => Some(Self::LeftFill),
			"right-fill" | "right"   => Some(Self::RightFill),

			_ => None
		}
	}

	pub fn child_constraints(self, inner_size: Vec2<f32>) -> BoxConstraints {
		let w = inner_size.x.max(0.0);
		let h = inner_size.y.max(0.0);
		match self {
			Self::Fill => BoxConstraints::tight(Vec2::new(w, h)),
			Self::TopFill | Self::BottomFill => BoxConstraints::new(w, w, 0.0, h),
			Self::LeftFill | Self::RightFill => BoxConstraints::new(0.0, w, h, h),
			_ => BoxConstraints::loose(Vec2::new(w, h)),
		}
	}

	pub fn position(self, inner_size: Vec2<f32>, child_size: Vec2<f32>) -> Vec2<f32> {
		let rem_x = (inner_size.x - child_size.x).max(0.0);
		let rem_y = (inner_size.y - child_size.y).max(0.0);

		let x = match self {
			Self::TopLeft | Self::CenterLeft | Self::BottomLeft | Self::Fill | Self::TopFill | Self::BottomFill | Self::LeftFill => 0.0,
			Self::TopCenter | Self::Center | Self::BottomCenter => rem_x * 0.5,
			Self::TopRight | Self::CenterRight | Self::BottomRight | Self::RightFill => rem_x,
		};

		let y = match self {
			Self::TopLeft | Self::TopCenter | Self::TopRight | Self::Fill | Self::TopFill | Self::LeftFill | Self::RightFill => 0.0,
			Self::CenterLeft | Self::Center | Self::CenterRight => rem_y * 0.5,
			Self::BottomLeft | Self::BottomCenter | Self::BottomRight | Self::BottomFill => rem_y,
		};

		Vec2::new(x, y)
	}
}

impl core::str::FromStr for Anchor {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s).ok_or(())
	}
}


#[derive(Clone, Debug)]
pub struct NodeData {
	pub parent: Option<ObjectID>,
	pub children: Vec<ObjectID>,
	pub overlay: Option<ObjectID>,
	pub bounds: Rect<f32>,
	pub anchor: Option<Anchor>,
	pub visible: bool,
	pub enabled: bool,
	pub hovered: bool,
	pub pressed: bool,
	pub focused: bool,
	pub clip_children: bool,
}

impl Default for NodeData {
	fn default() -> Self {
		Self {
			parent: None,
			children: Vec::new(),
			overlay: None,
			bounds: Rect::ZERO,
			anchor: None,
			visible: true,
			enabled: true,
			hovered: false,
			pressed: false,
			focused: false,
			clip_children: false,
		}
	}
}

impl NodeData {
	pub fn new() -> Self {
		Self::default()
	}
}
