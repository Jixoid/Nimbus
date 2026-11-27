/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use std::sync::Mutex;
use ash::vk;

use crate::swapchain::AshRenderTarget;
use crate::types::{to_vk_format, to_vk_sample_count};
use nimbus_rhi::error::{RhiError, RhiResult};
use nimbus_rhi::format::Format;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderPassKey {
	pub color_format: vk::Format,
	pub depth_format: vk::Format,
	pub samples: vk::SampleCountFlags,
	pub has_color_resolve: bool,
	pub depth_store: bool,
}

impl RenderPassKey {
	pub fn for_pipeline(
		color_format: vk::Format,
		depth_format: vk::Format,
		samples: vk::SampleCountFlags,
	) -> Self {
		Self {
			color_format,
			depth_format,
			samples,
			has_color_resolve: samples != vk::SampleCountFlags::TYPE_1
				&& color_format != vk::Format::UNDEFINED,
			depth_store: color_format == vk::Format::UNDEFINED,
		}
	}

	pub fn for_target(target: &AshRenderTarget) -> Self {
		let is_depth_only = target.color_image == vk::Image::null();
		let color_format = if is_depth_only {
			vk::Format::UNDEFINED
		} else {
			to_vk_format(target.color_format)
		};
		let depth_format = if target.depth_image == vk::Image::null() {
			vk::Format::UNDEFINED
		} else {
			to_vk_format(target.depth_format)
		};
		let has_depth_resolve = target.resolve_depth_view != vk::ImageView::null();
		let depth_store = is_depth_only
			|| (!has_depth_resolve && target.final_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

		Self {
			color_format,
			depth_format,
			samples: to_vk_sample_count(target.samples),
			has_color_resolve: target.resolve_view != vk::ImageView::null(),
			depth_store,
		}
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FramebufferKey {
	pub render_pass: vk::RenderPass,
	pub color_view: vk::ImageView,
	pub resolve_view: vk::ImageView,
	pub depth_view: vk::ImageView,
	pub width: u32,
	pub height: u32,
}


#[derive(Default)]
struct RenderPassCacheInner {
	render_passes: HashMap<RenderPassKey, vk::RenderPass>,
	framebuffers: HashMap<FramebufferKey, vk::Framebuffer>,
}

#[derive(Default)]
pub struct RenderPassCache {
	inner: Mutex<RenderPassCacheInner>,
}

impl RenderPassCache {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get_or_create_render_pass(
		&self,
		device: &ash::Device,
		key: RenderPassKey,
	) -> RhiResult<vk::RenderPass> {
		let mut guard = self.inner.lock().unwrap();
		if let Some(&rp) = guard.render_passes.get(&key) {
			return Ok(rp);
		}

		let mut attachments = Vec::with_capacity(3);
		let mut color_ref = None;
		let mut resolve_ref = None;
		let mut depth_ref = None;

		if key.color_format != vk::Format::UNDEFINED {
			let idx = attachments.len() as u32;
			attachments.push(
				vk::AttachmentDescription::default()
					.format(key.color_format)
					.samples(key.samples)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(if key.has_color_resolve {
						vk::AttachmentStoreOp::DONT_CARE
					} else {
						vk::AttachmentStoreOp::STORE
					})
					.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
					.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
					.final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
			);
			color_ref = Some(
				vk::AttachmentReference::default()
					.attachment(idx)
					.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
			);
		}

		if key.has_color_resolve {
			let idx = attachments.len() as u32;
			let resolve_fmt = if key.color_format != vk::Format::UNDEFINED {
				key.color_format
			} else {
				to_vk_format(Format::V4Norm8U)
			};
			attachments.push(
				vk::AttachmentDescription::default()
					.format(resolve_fmt)
					.samples(vk::SampleCountFlags::TYPE_1)
					.load_op(vk::AttachmentLoadOp::DONT_CARE)
					.store_op(vk::AttachmentStoreOp::STORE)
					.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
					.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
					.final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
			);
			resolve_ref = Some(
				vk::AttachmentReference::default()
					.attachment(idx)
					.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
			);
		}

		if key.depth_format != vk::Format::UNDEFINED {
			let idx = attachments.len() as u32;
			attachments.push(
				vk::AttachmentDescription::default()
					.format(key.depth_format)
					.samples(key.samples)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(if key.depth_store {
						vk::AttachmentStoreOp::STORE
					} else {
						vk::AttachmentStoreOp::DONT_CARE
					})
					.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
					.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
					.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
			);
			depth_ref = Some(
				vk::AttachmentReference::default()
					.attachment(idx)
					.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
			);
		}

		let color_refs = color_ref.into_iter().collect::<Vec<_>>();
		let resolve_refs = resolve_ref.into_iter().collect::<Vec<_>>();

		let mut subpass = vk::SubpassDescription::default()
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.color_attachments(&color_refs);
		if !resolve_refs.is_empty() {
			subpass = subpass.resolve_attachments(&resolve_refs);
		}
		if let Some(d_ref) = &depth_ref {
			subpass = subpass.depth_stencil_attachment(d_ref);
		}

		let subpasses = [subpass];
		let rp_info = vk::RenderPassCreateInfo::default()
			.attachments(&attachments)
			.subpasses(&subpasses);

		let rp = unsafe {
			device
				.create_render_pass(&rp_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create render pass: {e}")))?
		};

		guard.render_passes.insert(key, rp);
		Ok(rp)
	}

	pub fn get_or_create_framebuffer(
		&self,
		device: &ash::Device,
		key: FramebufferKey,
	) -> RhiResult<vk::Framebuffer> {
		let mut guard = self.inner.lock().unwrap();
		if let Some(&fb) = guard.framebuffers.get(&key) {
			return Ok(fb);
		}

		let mut views = Vec::with_capacity(3);
		if key.color_view != vk::ImageView::null() {
			views.push(key.color_view);
		}
		if key.resolve_view != vk::ImageView::null() {
			views.push(key.resolve_view);
		}
		if key.depth_view != vk::ImageView::null() {
			views.push(key.depth_view);
		}

		let fb_info = vk::FramebufferCreateInfo::default()
			.render_pass(key.render_pass)
			.attachments(&views)
			.width(key.width.max(1))
			.height(key.height.max(1))
			.layers(1);

		let fb = unsafe {
			device
				.create_framebuffer(&fb_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create framebuffer: {e}")))?
		};

		guard.framebuffers.insert(key, fb);
		Ok(fb)
	}

	pub fn clear_framebuffers(&self, device: &ash::Device) {
		let mut guard = self.inner.lock().unwrap();
		for (_, fb) in guard.framebuffers.drain() {
			unsafe {
				device.destroy_framebuffer(fb, None);
			}
		}
	}

	pub fn destroy_all(&self, device: &ash::Device) {
		let mut guard = self.inner.lock().unwrap();
		for (_, fb) in guard.framebuffers.drain() {
			unsafe {
				device.destroy_framebuffer(fb, None);
			}
		}
		for (_, rp) in guard.render_passes.drain() {
			unsafe {
				device.destroy_render_pass(rp, None);
			}
		}
	}
}
