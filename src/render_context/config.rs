use imgui::{TreeNodeFlags, Ui};
use vulkano::{
    device::DeviceOwned,
    swapchain::{PresentMode, SurfaceInfo, Swapchain},
};

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    present_mode: PresentMode,
}
impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            present_mode: PresentMode::Fifo,
        }
    }
}
impl RenderConfig {
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }
}

impl RenderConfig {
    pub fn ui(&mut self, ui: &Ui, swapchain: &Swapchain, receate_swapchain: &mut bool) {
        if ui.collapsing_header("Render Config", TreeNodeFlags::DEFAULT_OPEN) {
            let modes = swapchain
                .device()
                .physical_device()
                .surface_present_modes(swapchain.surface(), SurfaceInfo::default())
                .unwrap();
            ui.text("Present Modes");
            let prev = self.present_mode;
            for mode in modes {
                ui.radio_button(format!("{:?}", mode), &mut self.present_mode, mode);
            }
            if prev != self.present_mode {
                *receate_swapchain = true;
            }
        }
    }
}
