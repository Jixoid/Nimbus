/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(f16)]
#![feature(f128)]


mod aabb;
mod affine;
mod color;
mod degree;
mod euler;
mod frustum;
mod mat3x4;
mod mat4x4;
mod plane;
mod quat;
mod ray;
mod rect;
mod traits;
mod transform;
mod vec2;
mod vec3;
mod vec4;

pub use aabb::*;
pub use affine::*;
pub use color::*;
pub use degree::*;
pub use euler::*;
pub use frustum::*;
pub use mat3x4::*;
pub use mat4x4::*;
pub use plane::*;
pub use quat::*;
pub use ray::*;
pub use rect::*;
pub use traits::*;
pub use transform::*;
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;
