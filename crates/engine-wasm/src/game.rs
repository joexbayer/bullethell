use schema::CompiledContent;

use crate::rng::Rng64;
use crate::runtime::Runtime;
use crate::types::{AtlasMeta, GameConfig, InputSnapshot, FrameMeta};

#[derive(Clone)]
pub struct Game {
    pub content: CompiledContent,
    pub _atlas_meta: AtlasMeta,
    pub _config: GameConfig,
    pub rng: Rng64,
    pub runtime: Runtime,
    pub replay: ReplayState,
}

#[derive(Clone)]
pub struct ReplayState {
    pub recording_seed: u64,
    pub recorded_inputs: Vec<InputSnapshot>,
    pub playback: Option<crate::types::ReplayBlob>,
    pub playback_cursor: usize,
}

impl ReplayState {
    pub fn new(seed: u64) -> Self {
        Self {
            recording_seed: seed,
            recorded_inputs: Vec::new(),
            playback: None,
            playback_cursor: 0,
        }
    }
}

impl Game {
    pub fn step(&mut self, input: InputSnapshot) -> FrameMeta {
        if input.pause_pressed {
            self.runtime.paused = !self.runtime.paused;
        }
        if input.slow_mo_pressed {
            self.runtime.slow_mo = !self.runtime.slow_mo;
        }
        if input.debug_toggle_pressed {
            self.runtime.debug_enabled = !self.runtime.debug_enabled;
        }
        if self.runtime.paused && !input.frame_step_pressed {
            return self.runtime.frame_meta();
        }
        if self.runtime.paused {
            self.runtime.step_frame(input, &mut self.rng);
        } else if self.runtime.slow_mo {
            self.runtime.accumulator_frames = (self.runtime.accumulator_frames + 1) % 4;
            if self.runtime.accumulator_frames == 0 {
                self.runtime.step_frame(input, &mut self.rng);
            }
        } else {
            self.runtime.step_frame(input, &mut self.rng);
        }
        self.runtime.frame_meta()
    }
}
