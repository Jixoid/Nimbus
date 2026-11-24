/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod vec_edit;

pub use vec_edit::{Vec2Edit, Vec3Edit, Vec4Edit, VecEdit};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("inspector::vec2", || Box::new(Vec2Edit::default()));
	registry.register("inspector::vec3", || Box::new(Vec3Edit::default()));
	registry.register("inspector::vec4", || Box::new(Vec4Edit::default()));
}
