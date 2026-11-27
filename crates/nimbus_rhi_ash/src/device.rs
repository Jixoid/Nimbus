/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use std::sync::{Arc, Mutex};
use ash::vk::{self, Handle, PhysicalDeviceType};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::allocator::AshAllocator;
use crate::buffer::AshBuffer;
use crate::command::AshCommandQueue;
use crate::image::{AshImage2, AshImageView2};
use crate::pipeline::{AshPipeline, AshPipelineMeta};
use crate::render_pass_cache::RenderPassCache;
use crate::sampler::AshSampler;
use crate::shader::AshShader;
use crate::swapchain::AshRenderTarget;
use crate::sync::{AshFence, AshSemaphore};
use crate::types::{to_vk_address_mode, to_vk_filter, to_vk_format};
use crate::AshVulkanBackend;
use nimbus_rhi::error::{RhiError, RhiResult};
use nimbus_rhi::format::Format;
use nimbus_rhi::pipeline::{PolygonMode, PrimitiveTopology};
use nimbus_rhi::sampler::{SamplerAddressMode, SamplerFilter};
use nimbus_rhi::shader::ShaderStage;
use nimbus_rhi::traits::RhiDevice;
use nimbus_except::*;


pub struct AshBindlessHeap {
	pub layout: vk::DescriptorSetLayout,
	pub pool: vk::DescriptorPool,
	pub set: vk::DescriptorSet,
	pub next_id: Mutex<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceCapabilities {
	pub api_version: u32,
	pub has_dynamic_rendering_core: bool,
	pub has_dynamic_rendering_khr: bool,
	pub has_synchronization2_core: bool,
	pub has_synchronization2_khr: bool,
	pub has_buffer_device_address_core: bool,
	pub has_buffer_device_address_khr: bool,
	pub has_descriptor_indexing: bool,
	pub has_scalar_block_layout: bool,
	pub has_shader_int64: bool,
	pub has_depth_clamp: bool,
	pub has_sampler_anisotropy: bool,
}

impl DeviceCapabilities {
	pub fn use_dynamic_rendering(&self) -> bool {
		self.has_dynamic_rendering_core || self.has_dynamic_rendering_khr
	}

	pub fn use_synchronization2(&self) -> bool {
		self.has_synchronization2_core || self.has_synchronization2_khr
	}
}

pub struct DeviceCompat {
	pub caps: DeviceCapabilities,
	pub dynamic_rendering_khr: Option<ash::khr::dynamic_rendering::Device>,
	pub synchronization2_khr: Option<ash::khr::synchronization2::Device>,
	pub buffer_device_address_khr: Option<ash::khr::buffer_device_address::Device>,
	pub render_pass_cache: RenderPassCache,
}

pub struct DeviceInner {
	entry: ash::Entry,
	surface_loader: Option<ash::khr::surface::Instance>,
	debug_utils_loader: Option<ash::ext::debug_utils::Instance>,
	debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
	allocator: Option<Arc<Mutex<Allocator>>>,
	device: Option<ash::Device>,
	instance: Option<ash::Instance>,
	physical_device: vk::PhysicalDevice,
	queue: vk::Queue,
	queue_family: u32,
	bindless_heap: Option<AshBindlessHeap>,
	compat: Arc<DeviceCompat>,
}

unsafe impl Send for DeviceInner {}
unsafe impl Sync for DeviceInner {}

impl DeviceInner {
	pub fn device(&self) -> &ash::Device {
		self
			.device
			.as_ref()
			.expect("Ash logical device already destroyed")
	}

	pub fn instance(&self) -> &ash::Instance {
		self
			.instance
			.as_ref()
			.expect("Ash instance already destroyed")
	}

	pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
		self
			.surface_loader
			.as_ref()
			.expect("Ash surface loader already destroyed")
	}

	pub fn allocator(&self) -> &Arc<Mutex<Allocator>> {
		self
			.allocator
			.as_ref()
			.expect("Ash allocator already destroyed")
	}

	pub fn bindless_layout(&self) -> Option<vk::DescriptorSetLayout> {
		self.bindless_heap.as_ref().map(|h| h.layout)
	}

	pub fn bindless_set(&self) -> Option<vk::DescriptorSet> {
		self.bindless_heap.as_ref().map(|h| h.set)
	}
}

impl Drop for DeviceInner {
	fn drop(&mut self) {
		unsafe {
			if let Some(device) = &self.device {
				let _ = device.device_wait_idle();
				self.compat.render_pass_cache.destroy_all(device);
				if let Some(heap) = &self.bindless_heap {
					device.destroy_descriptor_pool(heap.pool, None);
					device.destroy_descriptor_set_layout(heap.layout, None);
				}
			}
			drop(self.allocator.take());
			if let Some(device) = self.device.take() {
				device.destroy_device(None);
			}
			if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
				loader.destroy_debug_utils_messenger(messenger, None);
			}
			self.surface_loader = None;
			self.debug_utils_loader = None;
			if let Some(instance) = self.instance.take() {
				instance.destroy_instance(None);
			}
		}
	}
}

unsafe extern "system" fn vulkan_debug_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_types: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
	let message = if !p_callback_data.is_null() && unsafe { !(*p_callback_data).p_message.is_null() }
	{
		unsafe { std::ffi::CStr::from_ptr((*p_callback_data).p_message) }.to_string_lossy()
	} else {
		std::borrow::Cow::Borrowed("unknown")
	};

	let type_str = if message_types.contains(vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE) {
		"PERF"
	} else if message_types.contains(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION) {
		"VALIDATION"
	} else {
		"GENERAL"
	};

	if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
		eprintln!("\x1b[1;31m[Vulkan {type_str} ERROR]\x1b[0m {message}");
	} else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
		eprintln!("\x1b[1;33m[Vulkan {type_str} WARNING]\x1b[0m {message}");
	} else {
		eprintln!("[Vulkan {type_str} INFO] {message}");
	}

	vk::FALSE
}

#[derive(Clone)]
pub struct AshDevice {
	inner: Arc<DeviceInner>,
}

unsafe impl Send for AshDevice {}
unsafe impl Sync for AshDevice {}

impl AshDevice {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> RhiResult<Self> {
		Self::new_with_options(app_name, major, minor, patch, build, false)
	}

	pub fn new_with_options(
		app_name: &str,
		major: u16,
		minor: u16,
		patch: u16,
		_build: u16,
		validation: bool,
	) -> RhiResult<Self> {
		let entry =
			unsafe { ash::Entry::load().map_err(|e| RhiError::InitializationFailed(format!("{e}")))? };

		let instance_api_version = unsafe { entry.try_enumerate_instance_version() }
			.unwrap_or(None)
			.unwrap_or(vk::API_VERSION_1_1)
			.clamp(vk::API_VERSION_1_1, vk::API_VERSION_1_3);

		let app_cname =
			CString::new(app_name).map_err(|e| RhiError::InitializationFailed(e.to_string()))?;
		let app_info = vk::ApplicationInfo::default()
			.application_name(&app_cname)
			.application_version(vk::make_api_version(
				0,
				major as u32,
				minor as u32,
				patch as u32,
			))
			.engine_name(c"Nimbus")
			.engine_version(vk::make_api_version(0, 0, 1, 0))
			.api_version(instance_api_version);

		let mut extension_names = vec![ash::khr::surface::NAME.as_ptr()];

		#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
		{
			extension_names.push(ash::khr::wayland_surface::NAME.as_ptr());
			extension_names.push(ash::khr::xlib_surface::NAME.as_ptr());
			extension_names.push(ash::khr::xcb_surface::NAME.as_ptr());
		}

		#[cfg(windows)]
		{
			extension_names.push(ash::khr::win32_surface::NAME.as_ptr());
		}

		#[cfg(target_os = "android")]
		{
			extension_names.push(ash::khr::android_surface::NAME.as_ptr());
		}

		let supported_inst_exts = unsafe {
			entry
				.enumerate_instance_extension_properties(None)
				.unwrap_or_default()
		};
		let mut enabled_inst_extensions = Vec::new();
		for req in &extension_names {
			let req_cstr = unsafe { std::ffi::CStr::from_ptr(*req) };
			if supported_inst_exts.iter().any(|ext| {
				let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
				name == req_cstr
			}) {
				enabled_inst_extensions.push(*req);
			}
		}

		let available_layers = unsafe {
			entry
				.enumerate_instance_layer_properties()
				.unwrap_or_default()
		};

		let khronos_validation_layer = c"VK_LAYER_KHRONOS_validation";
		let has_validation_layer = available_layers.iter().any(|layer| {
			let name = unsafe { std::ffi::CStr::from_ptr(layer.layer_name.as_ptr()) };
			name == khronos_validation_layer
		});

		let mut enabled_layers = Vec::new();
		let mut debug_enabled = false;

		if validation {
			if has_validation_layer {
				enabled_layers.push(khronos_validation_layer.as_ptr());
				nimbus_disp!("Vulkan validation layer enabled (VK_LAYER_KHRONOS_validation)");

				let debug_utils_ext_name = ash::ext::debug_utils::NAME;
				let has_debug_utils = supported_inst_exts.iter().any(|ext| {
					let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
					name == debug_utils_ext_name
				});

				if has_debug_utils {
					if !enabled_inst_extensions.contains(&debug_utils_ext_name.as_ptr()) {
						enabled_inst_extensions.push(debug_utils_ext_name.as_ptr());
					}
					debug_enabled = true;
				}
			} else {
				nimbus_warn!(
					"Vulkan validation layer requested, but 'VK_LAYER_KHRONOS_validation' was not found on this system"
				);
			}
		}

		let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
					| vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
			)
			.message_type(
				vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
					| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
					| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
			)
			.pfn_user_callback(Some(vulkan_debug_callback));

		let mut instance_create_info = vk::InstanceCreateInfo::default()
			.application_info(&app_info)
			.enabled_extension_names(&enabled_inst_extensions)
			.enabled_layer_names(&enabled_layers);

		if debug_enabled {
			instance_create_info = instance_create_info.push_next(&mut debug_info);
		}

		let instance = unsafe {
			entry
				.create_instance(&instance_create_info, None)
				.map_err(|e| {
					RhiError::InitializationFailed(format!("Failed to create Vulkan instance: {e}"))
				})?
		};

		let (debug_utils_loader, debug_messenger) = if debug_enabled {
			let loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
			let messenger = unsafe { loader.create_debug_utils_messenger(&debug_info, None).ok() };
			(Some(loader), messenger)
		} else {
			(None, None)
		};

		let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

		let physical_devices = unsafe {
			instance
				.enumerate_physical_devices()
				.map_err(|e| RhiError::InitializationFailed(format!("{e}")))?
		};

		let mut selected_phys = None;
		let mut selected_q_family = None;

		// Prioritize discrete GPU, then integrated GPU
		for phys in &physical_devices {
			let props = unsafe { instance.get_physical_device_properties(*phys) };
			if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
				let q_props = unsafe { instance.get_physical_device_queue_family_properties(*phys) };
				for (idx, q) in q_props.iter().enumerate() {
					if q
						.queue_flags
						.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
					{
						selected_phys = Some(*phys);
						selected_q_family = Some(idx as u32);
						break;
					}
				}
				if selected_phys.is_some() {
					break;
				}
			}
		}

		if selected_phys.is_none() {
			for phys in &physical_devices {
				let q_props = unsafe { instance.get_physical_device_queue_family_properties(*phys) };
				for (idx, q) in q_props.iter().enumerate() {
					if q
						.queue_flags
						.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
					{
						selected_phys = Some(*phys);
						selected_q_family = Some(idx as u32);
						break;
					}
				}
				if selected_phys.is_some() {
					break;
				}
			}
		}

		let physical_device = selected_phys.ok_or_else(|| {
			RhiError::InitializationFailed("No suitable Vulkan physical device found".into())
		})?;
		let queue_family = selected_q_family.unwrap();

		let queue_priorities = [1.0f32];
		let queue_create_info = vk::DeviceQueueCreateInfo::default()
			.queue_family_index(queue_family)
			.queue_priorities(&queue_priorities);

		let queue_create_infos = [queue_create_info];

		let props = unsafe { instance.get_physical_device_properties(physical_device) };
		let device_api_version = props.api_version.min(instance_api_version);
		let force_legacy =
			std::env::var("MOCHI_FORCE_VK11_FALLBACK").ok().as_deref() == Some("1");

		let supported_dev_exts = unsafe {
			instance
				.enumerate_device_extension_properties(physical_device)
				.unwrap_or_default()
		};
		let has_ext = |target: &std::ffi::CStr| {
			supported_dev_exts.iter().any(|ext| {
				let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
				name == target
			})
		};

		let mut enabled_dev_extensions = Vec::new();
		if has_ext(ash::khr::swapchain::NAME) {
			enabled_dev_extensions.push(ash::khr::swapchain::NAME.as_ptr());
		}
		#[cfg(unix)]
		if has_ext(ash::khr::external_memory_fd::NAME) {
			enabled_dev_extensions.push(ash::khr::external_memory_fd::NAME.as_ptr());
		}

		let has_dyn_khr_ext = !force_legacy && has_ext(ash::khr::dynamic_rendering::NAME);
		let has_sync2_khr_ext = !force_legacy && has_ext(ash::khr::synchronization2::NAME);
		let has_bda_khr_ext = has_ext(ash::khr::buffer_device_address::NAME);
		let has_di_ext = has_ext(ash::ext::descriptor_indexing::NAME);
		let has_sbl_ext = has_ext(ash::ext::scalar_block_layout::NAME);

		let mut sup_f12 = vk::PhysicalDeviceVulkan12Features::default();
		let mut sup_f13 = vk::PhysicalDeviceVulkan13Features::default();
		let mut sup_dyn_khr = vk::PhysicalDeviceDynamicRenderingFeaturesKHR::default();
		let mut sup_sync2_khr = vk::PhysicalDeviceSynchronization2FeaturesKHR::default();
		let mut sup_bda_khr = vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::default();
		let mut sup_di_ext = vk::PhysicalDeviceDescriptorIndexingFeaturesEXT::default();
		let mut sup_sbl_ext = vk::PhysicalDeviceScalarBlockLayoutFeaturesEXT::default();

		{
			let mut sup_f2 = vk::PhysicalDeviceFeatures2::default();
			if device_api_version >= vk::API_VERSION_1_3 {
				sup_f2 = sup_f2.push_next(&mut sup_f12).push_next(&mut sup_f13);
			} else if device_api_version >= vk::API_VERSION_1_2 {
				sup_f2 = sup_f2.push_next(&mut sup_f12);
				if has_dyn_khr_ext {
					sup_f2 = sup_f2.push_next(&mut sup_dyn_khr);
				}
				if has_sync2_khr_ext {
					sup_f2 = sup_f2.push_next(&mut sup_sync2_khr);
				}
			} else {
				if has_bda_khr_ext {
					sup_f2 = sup_f2.push_next(&mut sup_bda_khr);
				}
				if has_di_ext {
					sup_f2 = sup_f2.push_next(&mut sup_di_ext);
				}
				if has_sbl_ext {
					sup_f2 = sup_f2.push_next(&mut sup_sbl_ext);
				}
				if has_dyn_khr_ext {
					sup_f2 = sup_f2.push_next(&mut sup_dyn_khr);
				}
				if has_sync2_khr_ext {
					sup_f2 = sup_f2.push_next(&mut sup_sync2_khr);
				}
			}
			unsafe {
				instance.get_physical_device_features2(physical_device, &mut sup_f2);
			}
		}

		let sup_f10 = {
			let mut f2 = vk::PhysicalDeviceFeatures2::default();
			unsafe {
				instance.get_physical_device_features2(physical_device, &mut f2);
			}
			f2.features
		};

		let mut caps = DeviceCapabilities {
			api_version: device_api_version,
			has_dynamic_rendering_core: false,
			has_dynamic_rendering_khr: false,
			has_synchronization2_core: false,
			has_synchronization2_khr: false,
			has_buffer_device_address_core: false,
			has_buffer_device_address_khr: false,
			has_descriptor_indexing: false,
			has_scalar_block_layout: false,
			has_shader_int64: sup_f10.shader_int64 == vk::TRUE,
			has_depth_clamp: sup_f10.depth_clamp == vk::TRUE,
			has_sampler_anisotropy: sup_f10.sampler_anisotropy == vk::TRUE,
		};

		let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
		let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
		let mut features_dyn_khr = vk::PhysicalDeviceDynamicRenderingFeaturesKHR::default();
		let mut features_sync2_khr = vk::PhysicalDeviceSynchronization2FeaturesKHR::default();
		let mut features_bda_khr = vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::default();
		let mut features_di_ext = vk::PhysicalDeviceDescriptorIndexingFeaturesEXT::default();
		let mut features_sbl_ext = vk::PhysicalDeviceScalarBlockLayoutFeaturesEXT::default();

		if device_api_version >= vk::API_VERSION_1_2 {
			caps.has_buffer_device_address_core = sup_f12.buffer_device_address == vk::TRUE;
			caps.has_descriptor_indexing = sup_f12.descriptor_indexing == vk::TRUE;
			caps.has_scalar_block_layout = sup_f12.scalar_block_layout == vk::TRUE;

			features12 = features12
				.buffer_device_address(caps.has_buffer_device_address_core)
				.descriptor_indexing(caps.has_descriptor_indexing)
				.shader_sampled_image_array_non_uniform_indexing(
					sup_f12.shader_sampled_image_array_non_uniform_indexing == vk::TRUE,
				)
				.descriptor_binding_partially_bound(
					sup_f12.descriptor_binding_partially_bound == vk::TRUE,
				)
				.descriptor_binding_sampled_image_update_after_bind(
					sup_f12.descriptor_binding_sampled_image_update_after_bind == vk::TRUE,
				)
				.runtime_descriptor_array(sup_f12.runtime_descriptor_array == vk::TRUE)
				.scalar_block_layout(caps.has_scalar_block_layout);
		} else {
			if has_bda_khr_ext && sup_bda_khr.buffer_device_address == vk::TRUE {
				caps.has_buffer_device_address_khr = true;
				enabled_dev_extensions.push(ash::khr::buffer_device_address::NAME.as_ptr());
				features_bda_khr = features_bda_khr.buffer_device_address(true);
			}
			if has_di_ext && sup_di_ext.descriptor_binding_partially_bound == vk::TRUE {
				caps.has_descriptor_indexing = true;
				enabled_dev_extensions.push(ash::ext::descriptor_indexing::NAME.as_ptr());
				features_di_ext = features_di_ext
					.shader_sampled_image_array_non_uniform_indexing(
						sup_di_ext.shader_sampled_image_array_non_uniform_indexing == vk::TRUE,
					)
					.descriptor_binding_partially_bound(
						sup_di_ext.descriptor_binding_partially_bound == vk::TRUE,
					)
					.descriptor_binding_sampled_image_update_after_bind(
						sup_di_ext.descriptor_binding_sampled_image_update_after_bind == vk::TRUE,
					)
					.runtime_descriptor_array(sup_di_ext.runtime_descriptor_array == vk::TRUE);
			}
			if has_sbl_ext && sup_sbl_ext.scalar_block_layout == vk::TRUE {
				caps.has_scalar_block_layout = true;
				enabled_dev_extensions.push(ash::ext::scalar_block_layout::NAME.as_ptr());
				features_sbl_ext = features_sbl_ext.scalar_block_layout(true);
			}
		}

		if device_api_version >= vk::API_VERSION_1_3 && !force_legacy {
			caps.has_dynamic_rendering_core = sup_f13.dynamic_rendering == vk::TRUE;
			caps.has_synchronization2_core = sup_f13.synchronization2 == vk::TRUE;
			features13 = features13
				.dynamic_rendering(caps.has_dynamic_rendering_core)
				.synchronization2(caps.has_synchronization2_core);
		} else if !force_legacy {
			if has_dyn_khr_ext && sup_dyn_khr.dynamic_rendering == vk::TRUE {
				caps.has_dynamic_rendering_khr = true;
				enabled_dev_extensions.push(ash::khr::dynamic_rendering::NAME.as_ptr());
				if has_ext(ash::khr::create_renderpass2::NAME) {
					enabled_dev_extensions.push(ash::khr::create_renderpass2::NAME.as_ptr());
				}
				if has_ext(ash::khr::depth_stencil_resolve::NAME) {
					enabled_dev_extensions.push(ash::khr::depth_stencil_resolve::NAME.as_ptr());
				}
				features_dyn_khr = features_dyn_khr.dynamic_rendering(true);
			}
			if has_sync2_khr_ext && sup_sync2_khr.synchronization2 == vk::TRUE {
				caps.has_synchronization2_khr = true;
				enabled_dev_extensions.push(ash::khr::synchronization2::NAME.as_ptr());
				features_sync2_khr = features_sync2_khr.synchronization2(true);
			}
		}

		let features = vk::PhysicalDeviceFeatures::default()
			.sampler_anisotropy(caps.has_sampler_anisotropy)
			.shader_int64(caps.has_shader_int64)
			.depth_clamp(caps.has_depth_clamp);

		let mut device_create_info = vk::DeviceCreateInfo::default()
			.queue_create_infos(&queue_create_infos)
			.enabled_extension_names(&enabled_dev_extensions)
			.enabled_features(&features);

		if device_api_version >= vk::API_VERSION_1_3 {
			device_create_info = device_create_info
				.push_next(&mut features12)
				.push_next(&mut features13);
		} else if device_api_version >= vk::API_VERSION_1_2 {
			device_create_info = device_create_info.push_next(&mut features12);
			if caps.has_dynamic_rendering_khr {
				device_create_info = device_create_info.push_next(&mut features_dyn_khr);
			}
			if caps.has_synchronization2_khr {
				device_create_info = device_create_info.push_next(&mut features_sync2_khr);
			}
		} else {
			if caps.has_buffer_device_address_khr {
				device_create_info = device_create_info.push_next(&mut features_bda_khr);
			}
			if caps.has_descriptor_indexing {
				device_create_info = device_create_info.push_next(&mut features_di_ext);
			}
			if caps.has_scalar_block_layout {
				device_create_info = device_create_info.push_next(&mut features_sbl_ext);
			}
			if caps.has_dynamic_rendering_khr {
				device_create_info = device_create_info.push_next(&mut features_dyn_khr);
			}
			if caps.has_synchronization2_khr {
				device_create_info = device_create_info.push_next(&mut features_sync2_khr);
			}
		}

		let device = unsafe {
			instance
				.create_device(physical_device, &device_create_info, None)
				.map_err(|e| {
					RhiError::InitializationFailed(format!("Failed to create Vulkan logical device: {e}"))
				})?
		};

		let queue = unsafe { device.get_device_queue(queue_family, 0) };

		let dynamic_rendering_khr = if caps.has_dynamic_rendering_khr && !caps.has_dynamic_rendering_core
		{
			Some(ash::khr::dynamic_rendering::Device::new(&instance, &device))
		} else {
			None
		};
		let synchronization2_khr = if caps.has_synchronization2_khr && !caps.has_synchronization2_core {
			Some(ash::khr::synchronization2::Device::new(&instance, &device))
		} else {
			None
		};
		let buffer_device_address_khr =
			if caps.has_buffer_device_address_khr && !caps.has_buffer_device_address_core {
				Some(ash::khr::buffer_device_address::Device::new(
					&instance, &device,
				))
			} else {
				None
			};

		let compat = Arc::new(DeviceCompat {
			caps,
			dynamic_rendering_khr,
			synchronization2_khr,
			buffer_device_address_khr,
			render_pass_cache: RenderPassCache::new(),
		});

		let allocator_desc = AllocatorCreateDesc {
			instance: instance.clone(),
			device: device.clone(),
			physical_device,
			debug_settings: Default::default(),
			buffer_device_address: caps.has_buffer_device_address_core
				|| caps.has_buffer_device_address_khr,
			allocation_sizes: Default::default(),
		};

		let allocator = Allocator::new(&allocator_desc).map_err(|e| {
			RhiError::InitializationFailed(format!("Failed to create GPU allocator: {e}"))
		})?;

		let props = unsafe { instance.get_physical_device_properties(physical_device) };
		let dev_name =
			unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }.to_string_lossy();
		let pdt = match props.device_type {
			PhysicalDeviceType::INTEGRATED_GPU => "integrated",
			PhysicalDeviceType::DISCRETE_GPU => "discrete",
			PhysicalDeviceType::CPU => "cpu",
			PhysicalDeviceType::VIRTUAL_GPU => "virtual",
			_ => "other",
		};
		nimbus_disp!("Device initialized: {dev_name} ({pdt})");

		const MAX_BINDLESS_TEXTURES: u32 = 4096;

		let binding = vk::DescriptorSetLayoutBinding::default()
			.binding(0)
			.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(MAX_BINDLESS_TEXTURES)
			.stage_flags(vk::ShaderStageFlags::ALL);

		let binding_flags =
			[vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND];

		let mut binding_flags_info =
			vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

		let bindings = [binding];
		let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
			.flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
			.bindings(&bindings)
			.push_next(&mut binding_flags_info);

		let bindless_layout = unsafe {
			device
				.create_descriptor_set_layout(&layout_info, None)
				.map_err(|e| {
					RhiError::InitializationFailed(format!("Failed to create bindless set layout: {e}"))
				})?
		};

		let pool_size = [vk::DescriptorPoolSize::default()
			.ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(MAX_BINDLESS_TEXTURES)];

		let pool_info = vk::DescriptorPoolCreateInfo::default()
			.flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
			.max_sets(1)
			.pool_sizes(&pool_size);

		let bindless_pool = unsafe {
			device
				.create_descriptor_pool(&pool_info, None)
				.map_err(|e| {
					RhiError::InitializationFailed(format!("Failed to create bindless pool: {e}"))
				})?
		};

		let set_layouts = [bindless_layout];
		let alloc_info = vk::DescriptorSetAllocateInfo::default()
			.descriptor_pool(bindless_pool)
			.set_layouts(&set_layouts);

		let bindless_sets = unsafe {
			device.allocate_descriptor_sets(&alloc_info).map_err(|e| {
				RhiError::InitializationFailed(format!("Failed to allocate bindless descriptor set: {e}"))
			})?
		};

		let bindless_heap = AshBindlessHeap {
			layout: bindless_layout,
			pool: bindless_pool,
			set: bindless_sets[0],
			next_id: Mutex::new(0),
		};

		let inner = DeviceInner {
			entry,
			instance: Some(instance),
			physical_device,
			device: Some(device),
			queue,
			queue_family,
			allocator: Some(Arc::new(Mutex::new(allocator))),
			surface_loader: Some(surface_loader),
			debug_utils_loader,
			debug_messenger,
			bindless_heap: Some(bindless_heap),
			compat,
		};

		Ok(Self {
			inner: Arc::new(inner),
		})
	}

	pub fn capabilities(&self) -> DeviceCapabilities {
		self.inner.compat.caps
	}

	pub fn compat(&self) -> &Arc<DeviceCompat> {
		&self.inner.compat
	}

	pub fn bindless_layout(&self) -> Option<vk::DescriptorSetLayout> {
		self.inner.bindless_layout()
	}

	pub fn bindless_set(&self) -> Option<vk::DescriptorSet> {
		self.inner.bindless_set()
	}

	pub fn device(&self) -> &ash::Device {
		self.inner.device()
	}

	pub fn instance(&self) -> &ash::Instance {
		self.inner.instance()
	}

	pub fn physical_device(&self) -> vk::PhysicalDevice {
		self.inner.physical_device
	}

	pub fn queue(&self) -> vk::Queue {
		self.inner.queue
	}

	pub fn allocator(&self) -> &Arc<Mutex<Allocator>> {
		self.inner.allocator()
	}

	pub fn entry(&self) -> &ash::Entry {
		&self.inner.entry
	}

	pub fn surface_loader(&self) -> &ash::khr::surface::Instance {
		self.inner.surface_loader()
	}

	pub fn create_shader(
		&self,
		stage: ShaderStage,
		code: &[u8],
		entry: &str,
	) -> RhiResult<AshShader> {
		if code.len() % 4 != 0 {
			return Err(RhiError::ShaderCompilation(
				"SPIR-V code length must be a multiple of 4 bytes".into(),
			));
		}
		let words: Vec<u32> = if (code.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u32>()) {
			unsafe { std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4).to_vec() }
		} else {
			let mut words = vec![0u32; code.len() / 4];
			unsafe {
				std::ptr::copy_nonoverlapping(code.as_ptr(), words.as_mut_ptr() as *mut u8, code.len());
			}
			words
		};
		let info = vk::ShaderModuleCreateInfo::default().code(&words);
		let module = unsafe {
			self
				.inner
				.device()
				.create_shader_module(&info, None)
				.map_err(|e| RhiError::ShaderCompilation(format!("Failed to create shader module: {e}")))?
		};
		Ok(AshShader::new(self.inner.device(), module, stage, entry))
	}

	pub fn create_pipeline_meta(
		&self,
		push_stages: ShaderStage,
		push_size: u32,
	) -> RhiResult<AshPipelineMeta> {
		AshPipelineMeta::create(
			self.inner.device(),
			self.inner.bindless_layout(),
			push_stages,
			push_size,
		)
	}

	#[allow(clippy::too_many_arguments)]
	pub fn create_graphics_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		blend_enable: bool,
		depth_write: bool,
		samples: nimbus_rhi::MsaaSamples,
	) -> RhiResult<AshPipeline> {
		AshPipeline::create_graphics(
			self.inner.device(),
			&self.inner.compat,
			meta,
			shaders,
			polymode,
			primitive_topology,
			color_format,
			depth_format,
			self.inner.bindless_set(),
			blend_enable,
			depth_write,
			samples,
		)
	}

	pub fn create_compute_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shader: &AshShader,
	) -> RhiResult<AshPipeline> {
		AshPipeline::create_compute(self.inner.device(), meta, shader, self.inner.bindless_set())
	}

	pub fn write_raw_texture_descriptor(&self, view: vk::ImageView, sampler: vk::Sampler) -> u32 {
		if let Some(heap) = &self.inner.bindless_heap {
			let mut lock = heap.next_id.lock().unwrap();
			let id = *lock;
			*lock += 1;
			drop(lock);
			self.update_raw_texture_descriptor(id, view, sampler);
			id
		} else {
			0
		}
	}

	pub fn update_raw_texture_descriptor(
		&self,
		id: u32,
		view: vk::ImageView,
		sampler: vk::Sampler,
	) {
		if let Some(heap) = &self.inner.bindless_heap {
			let image_info = [vk::DescriptorImageInfo::default()
				.sampler(sampler)
				.image_view(view)
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];

			let write = [vk::WriteDescriptorSet::default()
				.dst_set(heap.set)
				.dst_binding(0)
				.dst_array_element(id)
				.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
				.image_info(&image_info)];

			unsafe {
				self.inner.device().update_descriptor_sets(&write, &[]);
			}
		}
	}
}

impl RhiDevice<AshVulkanBackend> for AshDevice {
	fn wait_idle(&self) -> RhiResult<()> {
		unsafe {
			self
				.inner
				.device()
				.device_wait_idle()
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}
		Ok(())
	}

	fn main_queue_family(&self) -> u32 {
		self.inner.queue_family
	}

	fn create_semaphore(&self) -> RhiResult<AshSemaphore> {
		let sem = unsafe {
			self
				.inner
				.device()
				.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};
		Ok(AshSemaphore::new(self.inner.device(), sem))
	}

	fn create_fence(&self, signaled: bool) -> RhiResult<AshFence> {
		let mut info = vk::FenceCreateInfo::default();
		if signaled {
			info = info.flags(vk::FenceCreateFlags::SIGNALED);
		}
		let fence = unsafe {
			self
				.inner
				.device()
				.create_fence(&info, None)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?
		};
		Ok(AshFence::new(self.inner.device(), fence))
	}

	fn wait_fence(&self, fence: &AshFence, timeout_ns: u64) -> RhiResult<()> {
		unsafe {
			self
				.inner
				.device()
				.wait_for_fences(&[fence.handle()], true, timeout_ns)
				.map_err(|e| match e {
					vk::Result::TIMEOUT => RhiError::Timeout,
					other => RhiError::BackendError(format!("{other}")),
				})?;
		}
		Ok(())
	}

	fn reset_fence(&self, fence: &AshFence) -> RhiResult<()> {
		unsafe {
			self
				.inner
				.device()
				.reset_fences(&[fence.handle()])
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}
		Ok(())
	}

	fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> RhiResult<u64> {
		let raw_disp = window
			.display_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?
			.as_raw();

		let raw_win = window
			.window_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?
			.as_raw();

		let surface = unsafe {
			ash_window::create_surface(
				&self.inner.entry,
				self.inner.instance(),
				raw_disp,
				raw_win,
				None,
			)
			.map_err(|e| RhiError::SurfaceCreation(format!("Failed to create window surface: {e}")))?
		};

		Ok(surface.as_raw())
	}

	fn destroy_surface(&self, surface_handle: u64) {
		let surface = vk::SurfaceKHR::from_raw(surface_handle);
		if surface != vk::SurfaceKHR::null()
			&& let Some(loader) = &self.inner.surface_loader
		{
			unsafe {
				loader.destroy_surface(surface, None);
			}
		}
	}

	fn init_descriptor_heap(&self, _allocator: &AshAllocator) -> RhiResult<()> {
		Ok(())
	}

	fn fini_descriptor_heap(&self) {}

	fn write_texture_descriptor(&self, view: &AshImageView2, sampler: &AshSampler) -> u32 {
		if let Some(heap) = &self.inner.bindless_heap {
			let mut lock = heap.next_id.lock().unwrap();
			let id = *lock;
			*lock += 1;
			drop(lock);

			self.update_texture_descriptor(id, view, sampler);
			id
		} else {
			0
		}
	}

	fn update_texture_descriptor(&self, id: u32, view: &AshImageView2, sampler: &AshSampler) {
		if let Some(heap) = &self.inner.bindless_heap {
			let image_info = [vk::DescriptorImageInfo::default()
				.sampler(sampler.handle())
				.image_view(view.handle())
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];

			let write = [vk::WriteDescriptorSet::default()
				.dst_set(heap.set)
				.dst_binding(0)
				.dst_array_element(id)
				.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
				.image_info(&image_info)];

			unsafe {
				self.inner.device().update_descriptor_sets(&write, &[]);
			}
		}
	}

	fn descriptor_heap(&self) -> Option<AshBuffer> {
		None
	}

	fn sampler_heap(&self) -> Option<AshBuffer> {
		None
	}

	fn create_image_view(&self, image: &AshImage2, format: Format) -> RhiResult<AshImageView2> {
		let is_depth = format == Format::Depth32f
			|| format == Format::Depth24Stencil8
			|| format == Format::Depth16
			|| format == Format::Depth32fStencil8;

		let aspect_mask = if is_depth {
			vk::ImageAspectFlags::DEPTH
		} else {
			vk::ImageAspectFlags::COLOR
		};

		let subresource_range = vk::ImageSubresourceRange {
			aspect_mask,
			base_mip_level: 0,
			level_count: 1,
			base_array_layer: 0,
			layer_count: 1,
		};

		let view_info = vk::ImageViewCreateInfo::default()
			.image(image.handle())
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(to_vk_format(format))
			.subresource_range(subresource_range);

		let view = unsafe {
			self
				.inner
				.device()
				.create_image_view(&view_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create image view: {e}")))?
		};

		Ok(AshImageView2::new(self.inner.device(), view))
	}

	fn create_sampler(
		&self,
		filter: SamplerFilter,
		address_mode: SamplerAddressMode,
	) -> RhiResult<AshSampler> {
		let vk_filter = to_vk_filter(filter);
		let vk_addr = to_vk_address_mode(address_mode);
		let is_clamp_to_border = address_mode == SamplerAddressMode::ClampToBorder;
		let border_color = if is_clamp_to_border {
			vk::BorderColor::FLOAT_OPAQUE_WHITE
		} else {
			vk::BorderColor::INT_OPAQUE_BLACK
		};
		let sampler_info = vk::SamplerCreateInfo::default()
			.mag_filter(vk_filter)
			.min_filter(vk_filter)
			.address_mode_u(vk_addr)
			.address_mode_v(vk_addr)
			.address_mode_w(vk_addr)
			.anisotropy_enable(!is_clamp_to_border)
			.max_anisotropy(if is_clamp_to_border { 1.0 } else { 16.0 })
			.border_color(border_color)
			.unnormalized_coordinates(false)
			.compare_enable(false)
			.compare_op(vk::CompareOp::ALWAYS)
			.mipmap_mode(vk::SamplerMipmapMode::LINEAR);

		let sampler = unsafe {
			self
				.inner
				.device()
				.create_sampler(&sampler_info, None)
				.map_err(|e| RhiError::ResourceCreation(format!("Failed to create sampler: {e}")))?
		};

		Ok(AshSampler::new(self.inner.device(), sampler))
	}

	fn create_render_target(
		&self,
		desc: &nimbus_rhi::RenderTargetDesc<AshVulkanBackend>,
	) -> AshRenderTarget {
		AshRenderTarget::new_from_desc(desc)
	}

	fn upload_image_data(
		&self,
		alloc: &AshAllocator,
		queue: &AshCommandQueue,
		image: &AshImage2,
		width: u32,
		height: u32,
		data: &[u8],
	) -> RhiResult<()> {
		use nimbus_rhi::traits::{RhiAllocator, RhiBuffer, RhiCommandQueue};

		let mut staging_buf = alloc.alloc_buffer(
			data.len() as u64,
			nimbus_rhi::BufferUsage::TransferSrc,
			nimbus_rhi::manager::AllocationCreate::HostSequentialWrite,
			nimbus_rhi::manager::AllocationLocation::PreferHost,
		)?;
		let ptr = staging_buf.map()?;
		unsafe {
			std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
		}
		staging_buf.unmap();

		let cmd = queue.allocate_graphic_command()?;
		let begin_info =
			vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		let raw_dev = self.inner.device();
		unsafe {
			raw_dev
				.begin_command_buffer(cmd.handle(), &begin_info)
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;

			let barrier_to_dst = vk::ImageMemoryBarrier::default()
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(image.handle())
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				})
				.src_access_mask(vk::AccessFlags::empty())
				.dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

			raw_dev.cmd_pipeline_barrier(
				cmd.handle(),
				vk::PipelineStageFlags::TOP_OF_PIPE,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier_to_dst],
			);

			let region = vk::BufferImageCopy::default()
				.buffer_offset(0)
				.buffer_row_length(0)
				.buffer_image_height(0)
				.image_subresource(vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					mip_level: 0,
					base_array_layer: 0,
					layer_count: 1,
				})
				.image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
				.image_extent(vk::Extent3D {
					width,
					height,
					depth: 1,
				});

			raw_dev.cmd_copy_buffer_to_image(
				cmd.handle(),
				staging_buf.handle(),
				image.handle(),
				vk::ImageLayout::TRANSFER_DST_OPTIMAL,
				&[region],
			);

			let barrier_to_shader = vk::ImageMemoryBarrier::default()
				.old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(image.handle())
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				})
				.src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
				.dst_access_mask(vk::AccessFlags::SHADER_READ);

			raw_dev.cmd_pipeline_barrier(
				cmd.handle(),
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::VERTEX_SHADER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier_to_shader],
			);

			raw_dev
				.end_command_buffer(cmd.handle())
				.map_err(|e| RhiError::BackendError(format!("{e}")))?;
		}

		queue.submit(&cmd)?;
		self.wait_idle()?;

		Ok(())
	}

	fn create_shader(&self, stage: ShaderStage, code: &[u8], entry: &str) -> RhiResult<AshShader> {
		self.create_shader(stage, code, entry)
	}

	fn create_pipeline_meta(
		&self,
		push_stages: ShaderStage,
		push_size: u32,
	) -> RhiResult<AshPipelineMeta> {
		self.create_pipeline_meta(push_stages, push_size)
	}

	fn create_graphics_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shaders: &[&AshShader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
		blend_enable: bool,
		depth_write: bool,
		samples: nimbus_rhi::MsaaSamples,
	) -> RhiResult<AshPipeline> {
		self.create_graphics_pipeline(
			meta,
			shaders,
			polymode,
			primitive_topology,
			color_format,
			depth_format,
			blend_enable,
			depth_write,
			samples,
		)
	}

	fn create_compute_pipeline(
		&self,
		meta: &AshPipelineMeta,
		shader: &AshShader,
	) -> RhiResult<AshPipeline> {
		self.create_compute_pipeline(meta, shader)
	}
}
