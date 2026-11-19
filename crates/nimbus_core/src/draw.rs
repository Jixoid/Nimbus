/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Rect;
use crate::id::ObjectID;
use crate::theme::Theme;


pub struct DrawCtx<'a> {
	pub node_id: ObjectID,
	pub bounds: Rect<f32>,
	pub theme: &'a Theme,
	pub hovered: bool,
	pub pressed: bool,
	pub focused: bool,
}
