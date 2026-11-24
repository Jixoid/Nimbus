/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod camera;
pub mod orientation;
pub mod transform;

pub use camera::GizmoCamera;
pub use nimbus_math::{Mat4, Plane, Quat, Rad, Ray, Transform, Vec3};
pub use orientation::{CameraViewPreset, OrientationGizmo, OrientationGizmoBuilder};
pub use transform::{
	GizmoHandle, GizmoMode, GizmoSnap, GizmoSpace, TransformGizmo, TransformGizmoBuilder,
};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("gizmo::transform", || Box::new(TransformGizmo::default()));
	registry.register("gizmo::orientation", || Box::new(OrientationGizmo::default()));
}
