use imgui::{TreeNodeFlags, Ui};
use vulkano::{
    device::DeviceOwned,
    swapchain::{PresentMode, SurfaceInfo, Swapchain},
};

use crate::msgln;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FramerateMode {
    Unlimited,
    Capped,
}

enum ResolutionMode {
    Fixed,
    FitWindow,
}

struct Resolution {
    mode: ResolutionMode,
    resolution: [u32; 2],
}

struct Framerate {
    mode: FramerateMode,
    framerate: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    // DEFAULT
    // CURRENT
    // UNSAVED
    // unsaved_changes;
    present_mode: PresentMode,
    framerate_mode: FramerateMode,
    target_framerate: f32,
    gui_scale: f32,
    resolution: [u32; 2],
}
impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            present_mode: PresentMode::Fifo,
            framerate_mode: FramerateMode::Unlimited,
            target_framerate: 120.0,
            gui_scale: 1.0,
            resolution: [1920, 1080],
        }
    }
}
impl RenderConfig {
    pub fn should_render(&self, last_frame: std::time::Instant) -> bool {
        match self.framerate_mode {
            FramerateMode::Unlimited => return true,
            FramerateMode::Capped => {
                return last_frame.elapsed().as_secs_f32() > (1.0 / self.target_framerate);
            }
        }
    }

    pub fn should_resize(&self, current_resolution: &[u32; 3]) -> bool {
        !(self.resolution[0] == current_resolution[0]
            && self.resolution[1] == current_resolution[1])
    }

    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn extent(&self) -> [u32; 3] {
        [self.resolution[0], self.resolution[1], 1]
    }

    pub fn set_resolution(&mut self, resolution: [u32; 2]) {
        self.resolution = resolution;
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
            // ================================================================================================== PRESENT MODE
            ui.text("Present Modes");
            let modes = swapchain
                .device()
                .physical_device()
                .surface_present_modes(swapchain.surface(), SurfaceInfo::default())
                .unwrap();
            let prev = self.present_mode;
            for mode in &modes {
                ui.radio_button(format!("{:?}", mode), &mut self.present_mode, *mode);
            }
            if prev != self.present_mode {
                *receate_swapchain = true;
            }
            // ================================================================================================== FPS
            ui.text("Framerate");
            ui.radio_button(
                "Unlimited",
                &mut self.framerate_mode,
                FramerateMode::Unlimited,
            );
            ui.radio_button("Capped", &mut self.framerate_mode, FramerateMode::Capped);
            ui.disabled(!(self.framerate_mode == FramerateMode::Capped), || {
                let step = 15.0;
                let min = 30.0;
                let max = 300.0;
                let value = &mut self.target_framerate;

                let mut index: i32 = ((*value - min) / step).round() as i32;
                let max_index = ((max - min) / step) as i32;
                if ui
                    .slider_config("Fps", 0, max_index)
                    .flags(imgui::SliderFlags::NO_INPUT | imgui::SliderFlags::NO_ROUND_TO_FORMAT)
                    .display_format(format!("{}", *value as i32))
                    .build(&mut index)
                {
                    *value = index as f32 * step + min;
                }
                ui.input_float("Fps Manual", value).build();
            });
            ui.separator();
            // ================================================================================================== GUI SCALE
            ui.input_scalar("Gui Scale", &mut self.gui_scale)
                .step(0.25)
                .build();

            // ================================================================================================== RESOLUTON
            ui.text("Game Resolution");
            ui.text(format!(
                "Current Resolution: [{}, {}]",
                self.resolution[0], self.resolution[1],
            ));
            let values = [[16, 9], [160, 90], [320, 180], [1920, 1080], [2560, 1440]];
            let labels = &["16x9", "160x90", "320x180", "1980x1080", "2560x1440"];

            let mut current = values
                .binary_search(&[self.resolution[0], self.resolution[1]])
                .unwrap_or(0);
            if ui.combo_simple_string("Resolution", &mut current, labels) {
                self.resolution = values[current];
            }
        }
    }
}
