/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::{Color, Vec2};
use nimbus_rhi::format::Format;
use nimbus_rhi::pipeline::{Pipeline, PipelineMeta, PolygonMode, PrimitiveTopology};
use nimbus_rhi::shader::{Shader, ShaderStage};
use nimbus_rhi::types::MsaaSamples;
use nimbus_rhi_ash::{AshVulkanBackend, Device};


pub static CURVE_VERT_SPV: &[u8] = include_bytes!("../../shaders/curve.vert.spv");
pub static CURVE_FRAG_SPV: &[u8] = include_bytes!("../../shaders/curve.frag.spv");

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CurveInstance {
	pub p0: [f32; 2],
	pub p1: [f32; 2],
	pub p2: [f32; 2],
	pub thickness: f32,
	pub _pad: f32,
	pub color: [f32; 4],
}

const _: () = {
	assert!(std::mem::size_of::<CurveInstance>() == 48);
	assert!(std::mem::align_of::<CurveInstance>() == 16);
};


pub struct CurveShape {
	pub(crate) instances: [CurveInstance; 2],
	pub(crate) count: usize,
}

impl CurveShape {
	/// Straight line segment with round caps (1 GPU instance).
	pub fn line(p0: Vec2<f32>, p1: Vec2<f32>, thickness: f32, color: Color<f32>) -> Self {
		let mid = Vec2::new((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
		Self::quadratic(p0, mid, p1, thickness, color)
	}

	/// Exact quadratic Bezier curve with round caps (1 GPU instance).
	pub fn quadratic(
		p0: Vec2<f32>,
		p1: Vec2<f32>,
		p2: Vec2<f32>,
		thickness: f32,
		color: Color<f32>,
	) -> Self {
		let inst = CurveInstance {
			p0: p0.to_array(),
			p1: p1.to_array(),
			p2: p2.to_array(),
			thickness: thickness.max(0.5),
			_pad: 0.0,
			color: color.to_array(),
		};
		Self {
			instances: [inst, CurveInstance::default()],
			count: 1,
		}
	}

	/// Smooth cubic Bezier curve split at t = 0.5 into 2 quadratic Bezier GPU instances.
	pub fn cubic(
		p0: Vec2<f32>,
		p1: Vec2<f32>,
		p2: Vec2<f32>,
		p3: Vec2<f32>,
		thickness: f32,
		color: Color<f32>,
	) -> Self {
		let m01 = (p0 + p1) * 0.5;
		let m12 = (p1 + p2) * 0.5;
		let m23 = (p2 + p3) * 0.5;
		let m012 = (m01 + m12) * 0.5;
		let m123 = (m12 + m23) * 0.5;
		let mid = (m012 + m123) * 0.5;

		// Degree reduction of each half cubic to quadratic: Q = (3*(B+C) - (A+D)) / 4
		let q_left = ((m01 + m012) * 3.0 - (p0 + mid)) * 0.25;
		let q_right = ((m123 + m23) * 3.0 - (mid + p3)) * 0.25;

		let thick = thickness.max(0.5);
		let col = color.to_array();
		Self {
			instances: [
				CurveInstance {
					p0: p0.to_array(),
					p1: q_left.to_array(),
					p2: mid.to_array(),
					thickness: thick,
					_pad: 0.0,
					color: col,
				},
				CurveInstance {
					p0: mid.to_array(),
					p1: q_right.to_array(),
					p2: p3.to_array(),
					thickness: thick,
					_pad: 0.0,
					color: col,
				},
			],
			count: 2,
		}
	}
}


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct CurvePushConstants {
	pub(crate) instance_addr: u64,
	pub(crate) screen_size: [f32; 2],
}

pub struct CurvePipeline {
	pipeline: Pipeline<AshVulkanBackend>,
}

impl CurvePipeline {
	pub fn new(device: &Device, color_format: Format) -> Self {
		let vert_shader = Shader::new(device, ShaderStage::Vertex, CURVE_VERT_SPV, "main");
		let frag_shader = Shader::new(device, ShaderStage::Pixel, CURVE_FRAG_SPV, "main");

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex,
			std::mem::size_of::<CurvePushConstants>() as u32,
		);

		let pipeline = Pipeline::new_graphics(
			device,
			&meta,
			&[&vert_shader, &frag_shader],
			PolygonMode::Fill,
			PrimitiveTopology::TriangleList,
			color_format,
			Format::Undefined,
			true,  // blend_enable
			false, // depth_write
			MsaaSamples::Off,
		);

		Self { pipeline }
	}

	pub fn pipeline(&self) -> &Pipeline<AshVulkanBackend> {
		&self.pipeline
	}
}
