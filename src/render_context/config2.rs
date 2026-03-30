use vulkano::{
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    swapchain::PresentMode,
};

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
struct GBufferConfig {
    store_op: AttachmentStoreOp,
    load_op: AttachmentLoadOp,
}

struct Config {
    framerate: FramerateConfig,
    resolution: ResolutionConfig,
}
