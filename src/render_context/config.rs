use std::time::{Duration, Instant};

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
#[derive(Debug, Clone, Copy)]
struct FramerateConfig {
    present_mode: PresentMode,
    mode: FramerateMode,
    framerate: f32,
}
impl Default for FramerateConfig {
    fn default() -> Self {
        Self {
            present_mode: PresentMode::Fifo,
            mode: FramerateMode::Unlimited,
            framerate: 120.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolutionMode {
    Fixed,
    FitWindow,
}

#[derive(Debug, Clone, Copy)]
struct ResolutionConfig {
    mode: ResolutionMode,
    target_res: [u32; 2],
    fixed_res: [u32; 2],
    res_scale: f32,
}
impl ResolutionConfig {
    fn should_resize(&self, current_res: &[u32; 3]) -> bool {
        let scaled_res = self.scaled_resolution();
        scaled_res[0] != current_res[0] || scaled_res[1] != current_res[1]
    }

    fn scaled_resolution(&self) -> [u32; 2] {
        [
            (self.target_res[0] as f32 * self.res_scale) as u32,
            (self.target_res[1] as f32 * self.res_scale) as u32,
        ]
    }
}
impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            mode: ResolutionMode::Fixed,
            target_res: [1920, 1080],
            fixed_res: [1920, 1080],
            res_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    // DEFAULT
    // CURRENT
    // UNSAVED
    // unsaved_changes;
    framerate: FramerateConfig,
    resolution: ResolutionConfig,
    gui_scale: f32,
}
impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            framerate: FramerateConfig::default(),
            gui_scale: 1.0,
            resolution: ResolutionConfig::default(),
        }
    }
}
impl RenderConfig {
    pub fn wait_till_render(&self, last_frame: Instant) {
        match self.framerate.mode {
            FramerateMode::Unlimited => {}
            FramerateMode::Capped => {
                // while last_frame.elapsed().as_secs_f32() < 1.0 / self.framerate.framerate {}
                let target_frame_time =
                    Duration::from_secs_f32(1.0 / self.framerate.framerate as f32);

                let elapsed = last_frame.elapsed();

                if elapsed < target_frame_time {
                    std::thread::sleep(target_frame_time - elapsed);
                }
            }
        }
    }

    pub fn window_resized(&mut self, size: [u32; 2]) {
        if self.resolution.mode == ResolutionMode::FitWindow {
            self.set_resolution(size);
        }
    }

    pub fn should_resize(&self, current_resolution: &[u32; 3]) -> bool {
        self.resolution.should_resize(current_resolution)
    }

    pub fn present_mode(&self) -> PresentMode {
        self.framerate.present_mode
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn extent(&self) -> [u32; 3] {
        [
            self.resolution.scaled_resolution()[0],
            self.resolution.scaled_resolution()[1],
            1,
        ]
    }

    pub fn set_resolution(&mut self, resolution: [u32; 2]) {
        self.resolution.target_res = resolution;
    }

    pub fn fallback_present_mode(&mut self, supported: &[PresentMode]) {
        if supported.contains(&self.framerate.present_mode) {
            return;
        }
        self.framerate.present_mode = match self.framerate.present_mode {
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

// ============================================================================================================== UI STUFF
impl FramerateConfig {
    fn ui(&mut self, ui: &Ui) {
        ui.text("Framerate");
        ui.radio_button("Unlimited", &mut self.mode, FramerateMode::Unlimited);
        ui.radio_button("Capped", &mut self.mode, FramerateMode::Capped);
        ui.disabled(!(self.mode == FramerateMode::Capped), || {
            let step = 15.0;
            let min = 30.0;
            let max = 300.0;
            let value = &mut self.framerate;

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
    }
}
impl ResolutionConfig {
    fn ui(&mut self, ui: &Ui, window: &winit::window::Window) {
        ui.text("Game Resolution");
        ui.text(format!(
            "Current Resolution: {:?}",
            self.scaled_resolution()
        ));
        if ui.radio_button("Fit Window", &mut self.mode, ResolutionMode::FitWindow) {
            self.target_res = window.inner_size().into();
        }
        if ui.radio_button("Fixed", &mut self.mode, ResolutionMode::Fixed) {
            self.target_res = self.fixed_res;
        }

        ui.disabled(!(self.mode == ResolutionMode::Fixed), || {
            let values = [[16, 9], [160, 90], [320, 180], [1920, 1080], [2560, 1440]];
            let labels = &["16x9", "160x90", "320x180", "1980x1080", "2560x1440"];
            let mut current = values
                .binary_search(&[self.fixed_res[0], self.fixed_res[1]])
                .unwrap_or(0);
            if ui.combo_simple_string("Resolution", &mut current, labels) {
                self.fixed_res = values[current];
                if self.mode == ResolutionMode::Fixed {
                    self.target_res = self.fixed_res;
                }
            }
        });
        ui.slider("Resolution Scale", 0.25, 2.0, &mut self.res_scale);
    }
}
impl RenderConfig {
    pub fn ui(
        &mut self,
        ui: &Ui,
        window: &winit::window::Window,
        swapchain: &Swapchain,
        receate_swapchain: &mut bool,
    ) {
        if ui.collapsing_header("Render Config", TreeNodeFlags::DEFAULT_OPEN) {
            // ================================================================================================== PRESENT MODE
            ui.text("Present Modes");
            let modes = swapchain
                .device()
                .physical_device()
                .surface_present_modes(swapchain.surface(), SurfaceInfo::default())
                .unwrap();
            let prev = self.framerate.present_mode;
            for mode in &modes {
                ui.radio_button(
                    format!("{:?}", mode),
                    &mut self.framerate.present_mode,
                    *mode,
                );
            }
            if prev != self.framerate.present_mode {
                *receate_swapchain = true;
            }
            // ================================================================================================== FPS
            self.framerate.ui(ui);
            ui.separator();
            // ================================================================================================== GUI SCALE
            ui.input_scalar("Gui Scale", &mut self.gui_scale)
                .step(0.25)
                .build();

            // ================================================================================================== RESOLUTON
            self.resolution.ui(ui, window);
        }
    }
}
