use std::sync::Arc;

use imgui::{TreeNodeFlags, Ui};
use vulkano::device::physical::PhysicalDevice;

#[derive(Debug, Clone)]
pub struct VulkanConfig {
    current_gpu: Option<Arc<PhysicalDevice>>,
    target_gpu: Option<Arc<PhysicalDevice>>,
    gpus: Vec<Arc<PhysicalDevice>>,

    needs_reset: bool,
}
impl Default for VulkanConfig {
    fn default() -> Self {
        Self {
            current_gpu: None,
            target_gpu: None,
            gpus: Vec::new(),

            needs_reset: false,
        }
    }
}
impl VulkanConfig {
    pub fn needs_devices(&self) -> bool {
        self.current_gpu.is_none()
    }

    pub fn clear_needs_reset(&mut self) {
        self.needs_reset = false;
        self.target_gpu = None;
    }

    pub fn needs_reset(&self) -> bool {
        self.needs_reset
    }

    pub fn target_device(&self) -> Option<u32> {
        if let Some(device) = &self.target_gpu {
            return Some(device.properties().device_id);
        }
        return None;
    }

    pub fn set_current_device(&mut self, device: Arc<PhysicalDevice>) {
        self.current_gpu = Some(device);
    }

    pub fn clear_devices(&mut self) {
        self.gpus = Vec::new();
    }

    pub fn add_device(&mut self, device: Arc<PhysicalDevice>) {
        self.gpus.push(device);
    }
}

impl VulkanConfig {
    pub fn ui(&mut self, ui: &Ui) {
        if ui.collapsing_header("Vulkan Config", TreeNodeFlags::DEFAULT_OPEN) {
            ui.text("Current Device");
            if let Some(device) = &self.current_gpu {
                ui.text(format!("{}", format_device(device)));
            } else {
                ui.text("None");
            }
            ui.text("All Devices");
            for device in &self.gpus {
                let active = self.current_gpu.as_ref().unwrap().properties().device_id
                    == device.properties().device_id;
                if ui.radio_button_bool(format!("{}", format_device(device)), active) {
                    if !active {
                        self.target_gpu = Some(device.clone());
                        self.needs_reset = true;
                    }
                }
            }
        }
    }
}

fn ui_checkbox_changed(ui: &Ui, label: impl AsRef<str>, value: &mut bool) -> bool {
    let prev = value.clone();
    ui.checkbox(label, value);
    return prev != *value;
}

fn format_device(device: &Arc<PhysicalDevice>) -> String {
    format!(
        "{} ({:?})",
        device.properties().device_name,
        device.properties().device_type,
    )
}
