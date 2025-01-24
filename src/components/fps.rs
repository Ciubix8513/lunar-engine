use lunar_engine_derive::as_any;

use crate::{delta_time, ecs::Component};

///Records fps over the runtime of the program
///
///Logs  average fps at the end of the program
#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct FpsRecorder {
    frames: u64,
    delta: f64,
}

impl Component for FpsRecorder {
    #[as_any]
    fn mew() -> Self {
        Self::default()
    }

    fn update(&mut self) {
        self.delta += f64::from(delta_time());
        self.frames += 1;
    }

    fn decatification(&mut self) {
        let avg_delta = self.delta / self.frames as f64;
        let avg_fps = 1.0 / avg_delta as f32;

        log::info!("Avg fps: {avg_fps}");
    }
}
