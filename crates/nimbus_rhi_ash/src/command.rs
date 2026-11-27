/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;

use crate::buffer::AshBuffer;
use crate::device::DeviceCompat;
use crate::pipeline::AshPipeline;
use crate::render_pass_cache::{FramebufferKey, RenderPassKey};
use crate::swapchain::AshRenderTarget;
use crate::sync::{AshFence, AshSemaphore};
use crate::types::{to_vk_index_type, to_vk_pipeline_bind_point, to_vk_shader_stage};
use crate::AshVulkanBackend;
use nimbus_rhi::error::{RhiError, RhiResult};
use nimbus_rhi::shader::ShaderStage;
use nimbus_rhi::traits::{RhiCommandBuffer, RhiCommandQueue};
use nimbus_rhi::types::{ClearColor, IndexType, Rect2D, Viewport};


fn stage2_to_stage1(s: vk::PipelineStageFlags2) -> vk::PipelineStageFlags {
	let mut out = vk::PipelineStageFlags::empty();
	if s.contains(vk::PipelineStageFlags2::TOP_OF_PIPE) {
		out |= vk::PipelineStageFlags::TOP_OF_PIPE;
	}
	if s.contains(vk::PipelineStageFlags2::DRAW_INDIRECT) {
		out |= vk::PipelineStageFlags::DRAW_INDIRECT;
	}
	if s.contains(vk::PipelineStageFlags2::VERTEX_INPUT) {
		out |= vk::PipelineStageFlags::VERTEX_INPUT;
	}
	if s.contains(vk::PipelineStageFlags2::VERTEX_SHADER) {
		out |= vk::PipelineStageFlags::VERTEX_SHADER;
	}
	if s.contains(vk::PipelineStageFlags2::FRAGMENT_SHADER) {
		out |= vk::PipelineStageFlags::FRAGMENT_SHADER;
	}
	if s.contains(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS) {
		out |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
	}
	if s.contains(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS) {
		out |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
	}
	if s.contains(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT) {
		out |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
	}
	if s.contains(vk::PipelineStageFlags2::COMPUTE_SHADER) {
		out |= vk::PipelineStageFlags::COMPUTE_SHADER;
	}
	if s.contains(vk::PipelineStageFlags2::TRANSFER) {
		out |= vk::PipelineStageFlags::TRANSFER;
	}
	if s.contains(vk::PipelineStageFlags2::BOTTOM_OF_PIPE) {
		out |= vk::PipelineStageFlags::BOTTOM_OF_PIPE;
	}
	if s.contains(vk::PipelineStageFlags2::HOST) {
		out |= vk::PipelineStageFlags::HOST;
	}
	if s.contains(vk::PipelineStageFlags2::ALL_GRAPHICS) {
		out |= vk::PipelineStageFlags::ALL_GRAPHICS;
	}
	if s.contains(vk::PipelineStageFlags2::ALL_COMMANDS) {
		out |= vk::PipelineStageFlags::ALL_COMMANDS;
	}
	out
}

fn access2_to_access1(a: vk::AccessFlags2) -> vk::AccessFlags {
	let mut out = vk::AccessFlags::empty();
	if a.contains(vk::AccessFlags2::SHADER_READ) {
		out |= vk::AccessFlags::SHADER_READ;
	}
	if a.contains(vk::AccessFlags2::SHADER_WRITE) {
		out |= vk::AccessFlags::SHADER_WRITE;
	}
	if a.contains(vk::AccessFlags2::COLOR_ATTACHMENT_READ) {
		out |= vk::AccessFlags::COLOR_ATTACHMENT_READ;
	}
	if a.contains(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE) {
		out |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
	}
	if a.contains(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ) {
		out |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
	}
	if a.contains(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE) {
		out |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
	}
	if a.contains(vk::AccessFlags2::TRANSFER_READ) {
		out |= vk::AccessFlags::TRANSFER_READ;
	}
	if a.contains(vk::AccessFlags2::TRANSFER_WRITE) {
		out |= vk::AccessFlags::TRANSFER_WRITE;
	}
	if a.contains(vk::AccessFlags2::MEMORY_READ) {
		out |= vk::AccessFlags::MEMORY_READ;
	}
	if a.contains(vk::AccessFlags2::MEMORY_WRITE) {
		out |= vk::AccessFlags::MEMORY_WRITE;
	}
	out
}


pub struct AshCommandBuffer {
	device: ash::Device,
	handle: vk::CommandBuffer,
	current_pipeline_layout: vk::PipelineLayout,
	compat: Arc<DeviceCompat>,
}

unsafe impl Send for AshCommandBuffer {}
unsafe impl Sync for AshCommandBuffer {}

impl AshCommandBuffer {
	pub fn new(
		device: &ash::Device,
		handle: vk::CommandBuffer,
		compat: Arc<DeviceCompat>,
	) -> Self {
		Self {
			device: device.clone(),
			handle,
			current_pipeline_layout: vk::PipelineLayout::null(),
			compat,
		}
	}

	pub fn handle(&self) -> vk::CommandBuffer {
		self.handle
	}

	fn cmd_barriers(
		&self,
		mem_barriers: &[vk::MemoryBarrier2],
		img_barriers: &[vk::ImageMemoryBarrier2],
		dep_flags: vk::DependencyFlags,
	) {
		if self.compat.caps.has_synchronization2_core {
			let dep_info = vk::DependencyInfo::default()
				.dependency_flags(dep_flags)
				.memory_barriers(mem_barriers)
				.image_memory_barriers(img_barriers);
			unsafe {
				self.device.cmd_pipeline_barrier2(self.handle, &dep_info);
			}
		} else if let Some(sync2) = &self.compat.synchronization2_khr {
			let dep_info = vk::DependencyInfo::default()
				.dependency_flags(dep_flags)
				.memory_barriers(mem_barriers)
				.image_memory_barriers(img_barriers);
			unsafe {
				sync2.cmd_pipeline_barrier2(self.handle, &dep_info);
			}
		} else {
			let mut src_stage = vk::PipelineStageFlags::empty();
			let mut dst_stage = vk::PipelineStageFlags::empty();

			let legacy_mem: Vec<vk::MemoryBarrier> = mem_barriers
				.iter()
				.map(|m| {
					src_stage |= stage2_to_stage1(m.src_stage_mask);
					dst_stage |= stage2_to_stage1(m.dst_stage_mask);
					vk::MemoryBarrier::default()
						.src_access_mask(access2_to_access1(m.src_access_mask))
						.dst_access_mask(access2_to_access1(m.dst_access_mask))
				})
				.collect();

			let legacy_img: Vec<vk::ImageMemoryBarrier> = img_barriers
				.iter()
				.map(|b| {
					src_stage |= stage2_to_stage1(b.src_stage_mask);
					dst_stage |= stage2_to_stage1(b.dst_stage_mask);
					vk::ImageMemoryBarrier::default()
						.src_access_mask(access2_to_access1(b.src_access_mask))
						.dst_access_mask(access2_to_access1(b.dst_access_mask))
						.old_layout(b.old_layout)
						.new_layout(b.new_layout)
						.src_queue_family_index(b.src_queue_family_index)
						.dst_queue_family_index(b.dst_queue_family_index)
						.image(b.image)
						.subresource_range(b.subresource_range)
				})
				.collect();

			if src_stage.is_empty() {
				src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
			}
			if dst_stage.is_empty() {
				dst_stage = vk::PipelineStageFlags::BOTTOM_OF_PIPE;
			}

			unsafe {
				self.device.cmd_pipeline_barrier(
					self.handle,
					src_stage,
					dst_stage,
					dep_flags,
					&legacy_mem,
					&[],
					&legacy_img,
				);
			}
		}
	}
}

impl RhiCommandBuffer<AshVulkanBackend> for AshCommandBuffer {
	fn begin(&mut self) -> RhiResult<()> {
		let begin_info =
			vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			self
				.device
				.begin_command_buffer(self.handle, &begin_info)
				.map_err(|e| RhiError::BackendError(format!("Failed to begin command buffer: {e}")))?;
		}
		Ok(())
	}

	fn end(&mut self) -> RhiResult<()> {
		unsafe {
			self
				.device
				.end_command_buffer(self.handle)
				.map_err(|e| RhiError::BackendError(format!("Failed to end command buffer: {e}")))?;
		}
		Ok(())
	}

	fn set_viewport(&mut self, vp: Viewport) {
		let viewport = vk::Viewport {
			x: vp.x,
			y: vp.y,
			width: vp.width,
			height: vp.height,
			min_depth: vp.min_depth,
			max_depth: vp.max_depth,
		};
		unsafe {
			self.device.cmd_set_viewport(self.handle, 0, &[viewport]);
		}
	}

	fn set_scissor(&mut self, s: Rect2D) {
		let scissor = vk::Rect2D {
			offset: vk::Offset2D { x: s.x, y: s.y },
			extent: vk::Extent2D {
				width: s.width,
				height: s.height,
			},
		};
		unsafe {
			self.device.cmd_set_scissor(self.handle, 0, &[scissor]);
		}
	}

	fn bind_pipeline(&mut self, pipeline: &AshPipeline) {
		self.current_pipeline_layout = pipeline.layout();
		let bind_point = to_vk_pipeline_bind_point(pipeline.kind());
		unsafe {
			self
				.device
				.cmd_bind_pipeline(self.handle, bind_point, pipeline.handle());
			if let Some(set) = pipeline.descriptor_set() {
				self.device.cmd_bind_descriptor_sets(
					self.handle,
					bind_point,
					pipeline.layout(),
					0,
					&[set],
					&[],
				);
			}
		}
	}

	fn bind_descriptor_heap(
		&mut self,
		_res_heap: Option<&AshBuffer>,
		_samp_heap: Option<&AshBuffer>,
		_res_sz: u64,
		_samp_sz: u64,
	) {
		// Descriptor buffers/heaps are bound via BDA or descriptor extensions
	}

	fn push_constant(&mut self, stages: ShaderStage, offset: u32, bytes: &[u8]) {
		if self.current_pipeline_layout != vk::PipelineLayout::null() && !bytes.is_empty() {
			unsafe {
				self.device.cmd_push_constants(
					self.handle,
					self.current_pipeline_layout,
					to_vk_shader_stage(stages),
					offset,
					bytes,
				);
			}
		}
	}

	fn bind_index_buffer(&mut self, buffer: &AshBuffer, offset: u64, index_type: IndexType) {
		unsafe {
			self.device.cmd_bind_index_buffer(
				self.handle,
				buffer.handle(),
				offset,
				to_vk_index_type(index_type),
			);
		}
	}

	fn draw(
		&mut self,
		vertex_count: u32,
		instance_count: u32,
		first_vertex: u32,
		first_instance: u32,
	) {
		unsafe {
			self.device.cmd_draw(
				self.handle,
				vertex_count,
				instance_count,
				first_vertex,
				first_instance,
			);
		}
	}

	fn draw_indexed(
		&mut self,
		index_count: u32,
		instance_count: u32,
		first_index: u32,
		vertex_offset: i32,
		first_instance: u32,
	) {
		unsafe {
			self.device.cmd_draw_indexed(
				self.handle,
				index_count,
				instance_count,
				first_index,
				vertex_offset,
				first_instance,
			);
		}
	}

	fn begin_rendering(&mut self, target: &AshRenderTarget, clear_color: ClearColor) {
		let depth_layout = if self.compat.caps.use_dynamic_rendering() {
			vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL
		} else {
			vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
		};

		// 1. Image barriers: Color to COLOR_ATTACHMENT_OPTIMAL, Depth to depth_layout
		let mut barriers = Vec::with_capacity(4);

		if target.color_image != vk::Image::null() {
			let color_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(
					vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
						| vk::PipelineStageFlags2::FRAGMENT_SHADER,
				)
				.src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE | vk::AccessFlags2::SHADER_READ)
				.dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.image(target.color_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::COLOR)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(color_barrier);
		}

		if target.resolve_image != vk::Image::null() {
			let resolve_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.src_access_mask(vk::AccessFlags2::NONE)
				.dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.image(target.resolve_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::COLOR)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(resolve_barrier);
		}

		if target.depth_image != vk::Image::null() {
			let depth_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(
					vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::FRAGMENT_SHADER,
				)
				.src_access_mask(
					vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE | vk::AccessFlags2::SHADER_READ,
				)
				.dst_stage_mask(
					vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
				)
				.dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(depth_layout)
				.image(target.depth_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::DEPTH)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(depth_barrier);
		}

		if target.resolve_depth_image != vk::Image::null() {
			let resolve_depth_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(
					vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::FRAGMENT_SHADER,
				)
				.src_access_mask(
					vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE | vk::AccessFlags2::SHADER_READ,
				)
				.dst_stage_mask(
					vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
						| vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
				)
				.dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(depth_layout)
				.image(target.resolve_depth_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::DEPTH)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(resolve_depth_barrier);
		}

		self.cmd_barriers(&[], &barriers, vk::DependencyFlags::BY_REGION);

		// 2. Begin Rendering (Dynamic Rendering or legacy VkRenderPass fallback)
		let is_depth_only = target.color_image == vk::Image::null();
		let render_area = vk::Rect2D {
			offset: vk::Offset2D { x: 0, y: 0 },
			extent: target.extent,
		};

		if self.compat.caps.use_dynamic_rendering() {
			let color_attachments = if !is_depth_only {
				let clear_val = vk::ClearValue {
					color: vk::ClearColorValue {
						float32: clear_color.0,
					},
				};

				let has_resolve = target.resolve_view != vk::ImageView::null();
				let mut color_attachment = vk::RenderingAttachmentInfo::default()
					.image_view(target.color_view)
					.image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(if has_resolve {
						vk::AttachmentStoreOp::DONT_CARE
					} else {
						vk::AttachmentStoreOp::STORE
					})
					.clear_value(clear_val);

				if has_resolve {
					color_attachment = color_attachment
						.resolve_mode(vk::ResolveModeFlags::AVERAGE)
						.resolve_image_view(target.resolve_view)
						.resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
				}

				vec![color_attachment]
			} else {
				Vec::new()
			};

			let has_depth_resolve = target.resolve_depth_view != vk::ImageView::null();
			let depth_store_op = if is_depth_only
				|| (!has_depth_resolve
					&& target.final_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
			{
				vk::AttachmentStoreOp::STORE
			} else {
				vk::AttachmentStoreOp::DONT_CARE
			};

			let mut depth_attachment = vk::RenderingAttachmentInfo::default()
				.image_view(target.depth_view)
				.image_layout(depth_layout)
				.load_op(vk::AttachmentLoadOp::CLEAR)
				.store_op(depth_store_op)
				.clear_value(vk::ClearValue {
					depth_stencil: vk::ClearDepthStencilValue {
						depth: 1.0,
						stencil: 0,
					},
				});

			if has_depth_resolve {
				depth_attachment = depth_attachment
					.resolve_mode(vk::ResolveModeFlags::SAMPLE_ZERO)
					.resolve_image_view(target.resolve_depth_view)
					.resolve_image_layout(depth_layout);
			}

			let mut rendering_info = vk::RenderingInfo::default()
				.render_area(render_area)
				.layer_count(1)
				.color_attachments(&color_attachments);

			if target.depth_image != vk::Image::null() {
				rendering_info = rendering_info.depth_attachment(&depth_attachment);
			}

			if self.compat.caps.has_dynamic_rendering_core {
				unsafe {
					self.device.cmd_begin_rendering(self.handle, &rendering_info);
				}
			} else if let Some(dr) = &self.compat.dynamic_rendering_khr {
				unsafe {
					dr.cmd_begin_rendering(self.handle, &rendering_info);
				}
			}
		} else {
			let rp_key = RenderPassKey::for_target(target);
			let render_pass = self
				.compat
				.render_pass_cache
				.get_or_create_render_pass(&self.device, rp_key)
				.expect("Failed to get/create fallback render pass");
			let fb_key = FramebufferKey {
				render_pass,
				color_view: if is_depth_only {
					vk::ImageView::null()
				} else {
					target.color_view
				},
				resolve_view: if is_depth_only {
					vk::ImageView::null()
				} else {
					target.resolve_view
				},
				depth_view: if target.depth_image != vk::Image::null() {
					target.depth_view
				} else {
					vk::ImageView::null()
				},
				width: target.extent.width,
				height: target.extent.height,
			};
			let framebuffer = self
				.compat
				.render_pass_cache
				.get_or_create_framebuffer(&self.device, fb_key)
				.expect("Failed to get/create fallback framebuffer");

			let mut clear_values = Vec::with_capacity(3);
			if !is_depth_only {
				clear_values.push(vk::ClearValue {
					color: vk::ClearColorValue {
						float32: clear_color.0,
					},
				});
				if target.resolve_view != vk::ImageView::null() {
					clear_values.push(vk::ClearValue {
						color: vk::ClearColorValue { float32: [0.0; 4] },
					});
				}
			}
			if target.depth_image != vk::Image::null() {
				clear_values.push(vk::ClearValue {
					depth_stencil: vk::ClearDepthStencilValue {
						depth: 1.0,
						stencil: 0,
					},
				});
			}

			let rp_begin = vk::RenderPassBeginInfo::default()
				.render_pass(render_pass)
				.framebuffer(framebuffer)
				.render_area(render_area)
				.clear_values(&clear_values);

			unsafe {
				self.device
					.cmd_begin_render_pass(self.handle, &rp_begin, vk::SubpassContents::INLINE);
			}
		}
	}

	fn end_rendering(&mut self, target: &AshRenderTarget) {
		if self.compat.caps.has_dynamic_rendering_core {
			unsafe {
				self.device.cmd_end_rendering(self.handle);
			}
		} else if let Some(dr) = &self.compat.dynamic_rendering_khr {
			unsafe {
				dr.cmd_end_rendering(self.handle);
			}
		} else {
			unsafe {
				self.device.cmd_end_render_pass(self.handle);
			}
		}

		let depth_layout = if self.compat.caps.use_dynamic_rendering() {
			vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL
		} else {
			vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
		};

		let mut barriers = Vec::new();
		let is_depth_only = target.color_image == vk::Image::null();

		if !is_depth_only {
			let present_image = if target.resolve_image != vk::Image::null() {
				target.resolve_image
			} else {
				target.color_image
			};

			let (dst_stage, dst_access) =
				if target.final_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
					(
						vk::PipelineStageFlags2::FRAGMENT_SHADER,
						vk::AccessFlags2::SHADER_READ,
					)
				} else {
					(
						vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
						vk::AccessFlags2::NONE,
					)
				};

			// Transition back to target.final_layout
			let color_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.dst_stage_mask(dst_stage)
				.dst_access_mask(dst_access)
				.old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.new_layout(target.final_layout)
				.image(present_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::COLOR)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);

			barriers.push(color_barrier);
		}

		let depth_image_to_transition = if target.resolve_depth_image != vk::Image::null() {
			target.resolve_depth_image
		} else {
			target.depth_image
		};

		if depth_image_to_transition != vk::Image::null()
			&& (is_depth_only || target.final_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
		{
			let depth_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
				.src_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
				.dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
				.dst_access_mask(vk::AccessFlags2::SHADER_READ)
				.old_layout(depth_layout)
				.new_layout(target.final_layout)
				.image(depth_image_to_transition)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::DEPTH)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);

			barriers.push(depth_barrier);
		}

		if !barriers.is_empty() {
			self.cmd_barriers(&[], &barriers, vk::DependencyFlags::empty());
		}
	}

	fn dispatch(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
		unsafe {
			self
				.device
				.cmd_dispatch(self.handle, group_count_x, group_count_y, group_count_z);
		}
	}

	fn compute_barrier(&mut self) {
		let memory_barrier = vk::MemoryBarrier2::default()
			.src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
			.src_access_mask(vk::AccessFlags2::SHADER_WRITE)
			.dst_stage_mask(
				vk::PipelineStageFlags2::COMPUTE_SHADER | vk::PipelineStageFlags2::FRAGMENT_SHADER,
			)
			.dst_access_mask(vk::AccessFlags2::SHADER_READ | vk::AccessFlags2::SHADER_WRITE);

		let memory_barriers = [memory_barrier];
		self.cmd_barriers(&memory_barriers, &[], vk::DependencyFlags::empty());
	}
}


pub struct CommandQueueInner {
	device: crate::device::AshDevice,
	command_pool: vk::CommandPool,
	queue: vk::Queue,
	submit_lock: Mutex<()>,
}

unsafe impl Send for CommandQueueInner {}
unsafe impl Sync for CommandQueueInner {}

impl Drop for CommandQueueInner {
	fn drop(&mut self) {
		if self.command_pool != vk::CommandPool::null() {
			unsafe {
				self.device.device().destroy_command_pool(self.command_pool, None);
			}
			self.command_pool = vk::CommandPool::null();
		}
	}
}

#[derive(Clone)]
pub struct AshCommandQueue {
	inner: Arc<CommandQueueInner>,
}

unsafe impl Send for AshCommandQueue {}
unsafe impl Sync for AshCommandQueue {}

impl AshCommandQueue {
	pub fn new(device: crate::device::AshDevice, queue_family: u32, queue: vk::Queue) -> RhiResult<Self> {
		let pool_info = vk::CommandPoolCreateInfo::default()
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
			.queue_family_index(queue_family);

		let command_pool = unsafe {
			device
				.device()
				.create_command_pool(&pool_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create command pool: {e}")))?
		};

		Ok(Self {
			inner: Arc::new(CommandQueueInner {
				device,
				command_pool,
				queue,
				submit_lock: Mutex::new(()),
			}),
		})
	}
}

impl RhiCommandQueue<AshVulkanBackend> for AshCommandQueue {
	fn allocate_graphic_command(&self) -> RhiResult<AshCommandBuffer> {
		let alloc_info = vk::CommandBufferAllocateInfo::default()
			.command_pool(self.inner.command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(1);

		let bufs = unsafe {
			self
				.inner
				.device
				.device()
				.allocate_command_buffers(&alloc_info)
				.map_err(|e| RhiError::BackendError(format!("Failed to allocate command buffer: {e}")))?
		};

		Ok(AshCommandBuffer::new(
			self.inner.device.device(),
			bufs[0],
			self.inner.device.compat().clone(),
		))
	}

	fn submit(&self, cmd: &AshCommandBuffer) -> RhiResult<()> {
		let cmds = [cmd.handle()];
		let submit_info = vk::SubmitInfo::default().command_buffers(&cmds);
		let submits = [submit_info];

		let _guard = self.inner.submit_lock.lock().unwrap();
		unsafe {
			self
				.inner
				.device
				.device()
				.queue_submit(self.inner.queue, &submits, vk::Fence::null())
				.map_err(|e| RhiError::BackendError(format!("queue_submit failed: {e}")))?;
		}
		Ok(())
	}

	fn submit_with_sync(
		&self,
		cmd: &AshCommandBuffer,
		wait_semaphore: Option<&AshSemaphore>,
		signal_semaphore: Option<&AshSemaphore>,
		fence: Option<&AshFence>,
	) -> RhiResult<()> {
		let cmds = [cmd.handle()];

		let wait_sems = match wait_semaphore {
			Some(s) => vec![s.handle()],
			None => vec![],
		};
		let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_sems.len()];

		let sig_sems = match signal_semaphore {
			Some(s) => vec![s.handle()],
			None => vec![],
		};

		let fence_handle = fence.map(|f| f.handle()).unwrap_or(vk::Fence::null());

		let submit_info = vk::SubmitInfo::default()
			.command_buffers(&cmds)
			.wait_semaphores(&wait_sems)
			.wait_dst_stage_mask(&wait_stages)
			.signal_semaphores(&sig_sems);

		let submits = [submit_info];

		let _guard = self.inner.submit_lock.lock().unwrap();
		unsafe {
			self
				.inner
				.device
				.device()
				.queue_submit(self.inner.queue, &submits, fence_handle)
				.map_err(|e| RhiError::BackendError(format!("queue_submit with sync failed: {e}")))?;
		}
		Ok(())
	}
}
