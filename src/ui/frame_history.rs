use eframe::{
    egui::{util::History, Context},
    Frame,
};

pub struct FrameHistory {
    frame_times: History<f32>,
}

impl Default for FrameHistory {
    fn default() -> Self {
        Self {
            frame_times: History::new(2..100, 1.0),
        }
    }
}

impl FrameHistory {
    pub fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let prev_time = frame.info().cpu_usage.unwrap_or_default();
        self.frame_times.add(ctx.input().time, prev_time);
    }

    pub fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }
}
