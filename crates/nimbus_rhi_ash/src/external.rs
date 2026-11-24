/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[cfg(unix)]
use std::os::unix::io::RawFd;

use ash::vk;
use nimbus_rhi::error::{RhiError, RhiResult};
use nimbus_rhi::format::Format;
use nimbus_rhi::traits::RhiCommandQueue;

use crate::command::AshCommandQueue;
#[cfg(unix)]
use crate::device::AshDevice;
#[cfg(unix)]
use crate::types::to_vk_format;


pub const EXTERNAL_SHARED_IMAGE_USAGE: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
	vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw()
		| vk::ImageUsageFlags::SAMPLED.as_raw()
		| vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
);


pub struct ImportedExternalTexture {
	device: ash::Device,
	image: vk::Image,
	memory: vk::DeviceMemory,
	view: vk::ImageView,
	sampler: vk::Sampler,
	texture_id: u32,
	width: u32,
	height: u32,
	format: Format,
}

unsafe impl Send for ImportedExternalTexture {}
unsafe impl Sync for ImportedExternalTexture {}

impl ImportedExternalTexture {
	#[cfg(unix)]
	pub fn import_opaque_fd(
		ash_device: &AshDevice,
		fd: RawFd,
		allocation_size: u64,
		width: u32,
		height: u32,
		format: Format,
		existing_texture_id: Option<u32>,
	) -> RhiResult<Self> {
		let vk_dev = ash_device.device();
		let vk_format = to_vk_format(format);

		let mut ext_image_info = vk::ExternalMemoryImageCreateInfo::default()
			.handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);

		let image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.format(vk_format)
			.extent(vk::Extent3D {
				width: width.max(1),
				height: height.max(1),
				depth: 1,
			})
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(EXTERNAL_SHARED_IMAGE_USAGE)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.push_next(&mut ext_image_info);

		let image = unsafe {
			vk_dev
				.create_image(&image_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create imported image: {e}")))?
		};

		let mem_reqs = unsafe { vk_dev.get_image_memory_requirements(image) };
		let mem_props = unsafe {
			ash_device
				.instance()
				.get_physical_device_memory_properties(ash_device.physical_device())
		};

		let mem_type_index = find_memory_type(
			&mem_props,
			mem_reqs.memory_type_bits,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
		)
		.ok_or_else(|| {
			unsafe { vk_dev.destroy_image(image, None) };
			RhiError::ResourceCreation("No compatible DEVICE_LOCAL memory type for imported image".into())
		})?;

		let mut import_fd_info = vk::ImportMemoryFdInfoKHR::default()
			.handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD)
			.fd(fd);
		let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(image);

		let alloc_info = vk::MemoryAllocateInfo::default()
			.allocation_size(allocation_size.max(mem_reqs.size))
			.memory_type_index(mem_type_index)
			.push_next(&mut import_fd_info)
			.push_next(&mut dedicated_info);

		let memory = unsafe {
			vk_dev.allocate_memory(&alloc_info, None).map_err(|e| {
				vk_dev.destroy_image(image, None);
				RhiError::ResourceCreation(format!("Failed to import external memory fd: {e}"))
			})?
		};

		if let Err(e) = unsafe { vk_dev.bind_image_memory(image, memory, 0) } {
			unsafe {
				vk_dev.destroy_image(image, None);
				vk_dev.free_memory(memory, None);
			}
			return Err(RhiError::ResourceCreation(format!(
				"Failed to bind imported image memory: {e}"
			)));
		}

		let view_info = vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk_format)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			});

		let view = unsafe {
			vk_dev.create_image_view(&view_info, None).map_err(|e| {
				vk_dev.destroy_image(image, None);
				vk_dev.free_memory(memory, None);
				RhiError::ResourceCreation(format!("Failed to create imported image view: {e}"))
			})?
		};

		let sampler_info = vk::SamplerCreateInfo::default()
			.mag_filter(vk::Filter::LINEAR)
			.min_filter(vk::Filter::LINEAR)
			.mipmap_mode(vk::SamplerMipmapMode::LINEAR)
			.address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
			.address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
			.address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
			.max_lod(1.0);

		let sampler = unsafe {
			vk_dev.create_sampler(&sampler_info, None).map_err(|e| {
				vk_dev.destroy_image_view(view, None);
				vk_dev.destroy_image(image, None);
				vk_dev.free_memory(memory, None);
				RhiError::ResourceCreation(format!("Failed to create imported sampler: {e}"))
			})?
		};

		let texture_id = match existing_texture_id {
			Some(id) => {
				ash_device.update_raw_texture_descriptor(id, view, sampler);
				id
			}
			None => ash_device.write_raw_texture_descriptor(view, sampler),
		};

		Ok(Self {
			device: vk_dev.clone(),
			image,
			memory,
			view,
			sampler,
			texture_id,
			width,
			height,
			format,
		})
	}

	pub fn texture_id(&self) -> u32 {
		self.texture_id
	}

	pub fn image(&self) -> vk::Image {
		self.image
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn format(&self) -> Format {
		self.format
	}

	pub fn transition_to_shader_read(&self, cmd_queue: &AshCommandQueue) -> RhiResult<()> {
		let cmd = cmd_queue.allocate_graphic_command()?;
		let begin_info =
			vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			self
				.device
				.begin_command_buffer(cmd.handle(), &begin_info)
				.map_err(|e| RhiError::BackendError(format!("begin_command_buffer failed: {e}")))?;

			let barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
				.src_access_mask(vk::AccessFlags2::NONE)
				.dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
				.dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(self.image)
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				});

			let barriers = [barrier];
			let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);
			self.device.cmd_pipeline_barrier2(cmd.handle(), &dep_info);

			self
				.device
				.end_command_buffer(cmd.handle())
				.map_err(|e| RhiError::BackendError(format!("end_command_buffer failed: {e}")))?;
		}

		cmd_queue.submit(&cmd)?;
		unsafe {
			let _ = self.device.device_wait_idle();
		}
		Ok(())
	}
}

impl Drop for ImportedExternalTexture {
	fn drop(&mut self) {
		unsafe {
			let _ = self.device.device_wait_idle();
			self.device.destroy_sampler(self.sampler, None);
			self.device.destroy_image_view(self.view, None);
			self.device.destroy_image(self.image, None);
			self.device.free_memory(self.memory, None);
		}
	}
}


#[cfg(unix)]
fn find_memory_type(
	mem_props: &vk::PhysicalDeviceMemoryProperties,
	type_filter: u32,
	required: vk::MemoryPropertyFlags,
) -> Option<u32> {
	(0..mem_props.memory_type_count).find(|&i| {
		(type_filter & (1 << i)) != 0
			&& mem_props.memory_types[i as usize]
				.property_flags
				.contains(required)
	})
}
