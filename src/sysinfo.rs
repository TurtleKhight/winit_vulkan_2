use std::time::{Duration, Instant};

use imgui::{TreeNodeFlags, Ui};
use sysinfo::{Pid, System};

pub struct SysInfo {
    average_fps: ArrayDeque<f32, 50>,
    sysinfo: System,
    pid: Pid,

    timer: Instant,
    mem_usage: u64,
    vir_mem_usage: u64,
}
impl SysInfo {
    pub fn new() -> Self {
        let mut sysinfo = System::new_all();
        sysinfo.refresh_all();
        let pid = Pid::from(std::process::id() as usize);
        let timer = Instant::now();
        Self {
            average_fps: ArrayDeque::new(),
            sysinfo,
            pid,
            timer,
            mem_usage: 0,
            vir_mem_usage: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.average_fps.push(dt);
        if self.timer.elapsed() > Duration::from_millis(500) {
            self.timer = Instant::now();
            self.sysinfo.refresh_all();
            if let Some(process) = self.sysinfo.process(self.pid) {
                self.mem_usage = process.memory();
                self.vir_mem_usage = process.virtual_memory();
            }
        }
    }
}

impl SysInfo {
    pub fn ui(&self, ui: &Ui) {
        if ui.collapsing_header("System Info", TreeNodeFlags::DEFAULT_OPEN) {
            let avg = self.average_fps.sum() / self.average_fps.buf.len() as f32;
            ui.text(format!("Avg FPS: {}", (1.0 / avg).round() as u32));
            let mem_usage = self.mem_usage / (1024 * 1024);
            let vir_mem_usage = self.vir_mem_usage / (1024 * 1024);
            ui.text(format!("Memory usage: {} MB", mem_usage));
            ui.text(format!("Virtual memory: {} MB", vir_mem_usage));
        }
    }
}

struct ArrayDeque<T, const N: usize> {
    buf: [T; N],
    head: usize,
    len: usize,
}
impl<T: Default + Copy + std::ops::Add<Output = T>, const N: usize> ArrayDeque<T, N> {
    fn new() -> Self {
        let buf = [T::default(); N];
        let head = 0;
        let len = 0;
        Self { buf, head, len }
    }

    fn push(&mut self, val: T) {
        self.buf[self.head] = val;
        self.head = (self.head + 1) % self.buf.len();
        self.len += 1;
    }

    fn sum(&self) -> T {
        let mut avg = T::default();
        for val in self.buf {
            avg = avg + val;
        }
        avg
    }

    // fn linear(&self) -> [T; N] {
    //     let mut out = [T::default(); N];

    //     let start = (self.head + N - self.len.min(N)) % N;
    //     let len = self.len.min(N);

    //     for i in 0..len {
    //         out[i] = self.buf[(start + i) % N];
    //     }

    //     out
    // }
}
