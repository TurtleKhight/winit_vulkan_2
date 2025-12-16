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

    pub fn fallback_present_mode(&mut self, supported: &[PresentMode]) {
        if supported.contains(&self.present_mode) {
            return;
        }
        self.present_mode = match self.present_mode {
            PresentMode::Immediate => {
                if supported.contains(&PresentMode::Mailbox) {
                    PresentMode::Mailbox
                } else {
                    PresentMode::Fifo
                }
            }
            PresentMode::Mailbox => {
                if supported.contains(&PresentMode::Immediate) {
                    PresentMode::Immediate
                } else {
                    PresentMode::Fifo
                }
            }
            PresentMode::Fifo => PresentMode::Fifo,
            PresentMode::FifoRelaxed => {
                if supported.contains(&PresentMode::FifoRelaxed) {
                    PresentMode::FifoRelaxed
                } else {
                    PresentMode::Fifo
                }
            }
            _ => PresentMode::Fifo,
        }
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
            for mode in &modes {
                ui.radio_button(format!("{:?}", mode), &mut self.present_mode, *mode);
            }

            if prev != self.present_mode {
                *receate_swapchain = true;
            }
        }
    }
}
