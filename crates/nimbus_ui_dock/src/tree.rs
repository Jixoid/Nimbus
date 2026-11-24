/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::ObjectID;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DockSplitAxis {
	/// Side-by-side (left | right)
	Horizontal,
	/// Stacked (top / bottom)
	Vertical,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum DockZone {
	#[default]
	Center,
	Left,
	Right,
	Top,
	Bottom,
}

impl DockZone {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim().to_ascii_lowercase().as_str() {
			"center" | "tab" => Some(Self::Center),
			"left" => Some(Self::Left),
			"right" => Some(Self::Right),
			"top" => Some(Self::Top),
			"bottom" => Some(Self::Bottom),
			_ => None,
		}
	}
}

/// Path through a `DockNode` binary tree (`0` = first child, `1` = second child).
pub type DockPath = Vec<u8>;


#[derive(Clone, Debug, Default, PartialEq)]
pub enum DockNode {
	#[default]
	Empty,
	Tabs {
		panes: Vec<ObjectID>,
		active: usize,
	},
	Split {
		axis: DockSplitAxis,
		ratio: f32,
		first: Box<DockNode>,
		second: Box<DockNode>,
	},
}

impl DockNode {
	pub fn tabs(panes: Vec<ObjectID>) -> Self {
		if panes.is_empty() {
			Self::Empty
		} else {
			Self::Tabs { panes, active: 0 }
		}
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	pub fn contains_pane(&self, pane_id: ObjectID) -> bool {
		match self {
			Self::Empty => false,
			Self::Tabs { panes, .. } => panes.contains(&pane_id),
			Self::Split { first, second, .. } => {
				first.contains_pane(pane_id) || second.contains_pane(pane_id)
			}
		}
	}

	pub fn node_at_mut(&mut self, path: &[u8]) -> Option<&mut Self> {
		if path.is_empty() {
			return Some(self);
		}
		match self {
			Self::Split { first, second, .. } => match path[0] {
				0 => first.node_at_mut(&path[1..]),
				1 => second.node_at_mut(&path[1..]),
				_ => None,
			},
			_ => None,
		}
	}

	pub fn set_active_at(&mut self, path: &[u8], index: usize) -> bool {
		if let Some(Self::Tabs { panes, active }) = self.node_at_mut(path)
			&& index < panes.len()
			&& *active != index
		{
			*active = index;
			return true;
		}
		false
	}

	pub fn set_ratio_at(&mut self, path: &[u8], new_ratio: f32) -> bool {
		let clamped = new_ratio.clamp(0.12, 0.88);
		if let Some(Self::Split { ratio, .. }) = self.node_at_mut(path)
			&& (*ratio - clamped).abs() > 1e-4
		{
			*ratio = clamped;
			return true;
		}
		false
	}

	/// Docks `pane_id` into the node at `path` according to `zone`
	pub fn dock_at(&mut self, path: &[u8], pane_id: ObjectID, zone: DockZone) -> bool {
		let Some(target) = self.node_at_mut(path) else {
			return false;
		};
		target.dock_here(pane_id, zone);
		true
	}

	pub fn dock_here(&mut self, pane_id: ObjectID, zone: DockZone) {
		if matches!(self, Self::Empty) {
			*self = Self::Tabs {
				panes: vec![pane_id],
				active: 0,
			};
			return;
		}

		match zone {
			DockZone::Center => match self {
				Self::Tabs { panes, active } => {
					if let Some(pos) = panes.iter().position(|&p| p == pane_id) {
						*active = pos;
					} else {
						panes.push(pane_id);
						*active = panes.len() - 1;
					}
				}
				Self::Split { first, .. } => {
					first.dock_here(pane_id, DockZone::Center);
				}
				Self::Empty => unreachable!(),
			},
			DockZone::Left | DockZone::Right | DockZone::Top | DockZone::Bottom => {
				let old = std::mem::take(self);
				let new_leaf = Self::Tabs {
					panes: vec![pane_id],
					active: 0,
				};
				let (axis, first, second, ratio) = match zone {
					DockZone::Left => (DockSplitAxis::Horizontal, new_leaf, old, 0.25),
					DockZone::Right => (DockSplitAxis::Horizontal, old, new_leaf, 0.72),
					DockZone::Top => (DockSplitAxis::Vertical, new_leaf, old, 0.28),
					DockZone::Bottom => (DockSplitAxis::Vertical, old, new_leaf, 0.72),
					DockZone::Center => unreachable!(),
				};
				*self = Self::Split {
					axis,
					ratio,
					first: Box::new(first),
					second: Box::new(second),
				};
			}
		}
	}

	pub fn remove_pane(&mut self, pane_id: ObjectID) -> bool {
		let removed = match self {
			Self::Empty => false,
			Self::Tabs { panes, active } => {
				if let Some(idx) = panes.iter().position(|&p| p == pane_id) {
					panes.remove(idx);
					if panes.is_empty() {
						*self = Self::Empty;
					} else if *active >= panes.len() {
						*active = panes.len() - 1;
					}
					true
				} else {
					false
				}
			}
			Self::Split { first, second, .. } => {
				first.remove_pane(pane_id) || second.remove_pane(pane_id)
			}
		};

		if removed {
			self.simplify();
		}
		removed
	}

	/// Collapses any `Split` where one or both children are `Empty`
	pub fn simplify(&mut self) {
		if let Self::Split { first, second, .. } = self {
			first.simplify();
			second.simplify();
			match (first.is_empty(), second.is_empty()) {
				(true, true) => *self = Self::Empty,
				(true, false) => {
					let kept = std::mem::take(second.as_mut());
					*self = kept;
				}
				(false, true) => {
					let kept = std::mem::take(first.as_mut());
					*self = kept;
				}
				(false, false) => {}
			}
		}
	}

	/// Finds the path to the `Tabs` leaf containing `pane_id`.
	pub fn find_pane_path(&self, pane_id: ObjectID) -> Option<DockPath> {
		let mut path = Vec::new();
		if self.find_pane_path_inner(pane_id, &mut path) {
			Some(path)
		} else {
			None
		}
	}

	fn find_pane_path_inner(&self, pane_id: ObjectID, path: &mut DockPath) -> bool {
		match self {
			Self::Empty => false,
			Self::Tabs { panes, .. } => panes.contains(&pane_id),
			Self::Split { first, second, .. } => {
				path.push(0);
				if first.find_pane_path_inner(pane_id, path) {
					return true;
				}
				path.pop();

				path.push(1);
				if second.find_pane_path_inner(pane_id, path) {
					return true;
				}
				path.pop();
				false
			}
		}
	}

	/// Moves `pane_id` from its current location into the leaf containing `target_anchor_pane`
	/// (which stays stable even after removing `pane_id` collapses splits).
	pub fn move_pane_to(
		&mut self,
		pane_id: ObjectID,
		target_anchor_pane: ObjectID,
		zone: DockZone,
	) -> bool {
		if pane_id == target_anchor_pane && zone == DockZone::Center {
			return false;
		}
		// If dragging the sole pane of a leaf onto its own edge, do nothing
		if let Some(src_path) = self.find_pane_path(pane_id)
			&& let Some(dst_path) = self.find_pane_path(target_anchor_pane)
			&& src_path == dst_path
			&& let Some(Self::Tabs { panes, .. }) = self.node_at_mut(&src_path)
			&& panes.len() <= 1
		{
			return false;
		}

		if !self.remove_pane(pane_id) {
			return false;
		}

		if let Some(new_dst_path) = self.find_pane_path(target_anchor_pane) {
			self.dock_at(&new_dst_path, pane_id, zone)
		} else {
			self.dock_here(pane_id, zone);
			true
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use slotmap::SlotMap;

	fn fake_ids(n: usize) -> Vec<ObjectID> {
		let mut map: SlotMap<ObjectID, ()> = SlotMap::with_key();
		(0..n).map(|_| map.insert(())).collect()
	}

	#[test]
	fn test_dock_split_tab_and_collapse() {
		let ids = fake_ids(4);
		let mut root = DockNode::Empty;

		root.dock_here(ids[0], DockZone::Center);
		assert!(root.contains_pane(ids[0]));

		root.dock_here(ids[1], DockZone::Left);
		root.dock_here(ids[2], DockZone::Right);
		root.dock_here(ids[3], DockZone::Bottom);

		for &id in &ids {
			assert!(root.contains_pane(id));
		}

		// Move ids[3] into ids[0]'s tab group
		assert!(root.move_pane_to(ids[3], ids[0], DockZone::Center));
		assert_eq!(root.find_pane_path(ids[3]), root.find_pane_path(ids[0]));

		// Remove all panes and ensure tree collapses cleanly to Empty
		for &id in &ids {
			assert!(root.remove_pane(id));
		}
		assert!(root.is_empty());
	}
}
