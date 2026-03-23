use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u32,
    pub height: u32,
    pub debug_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtlasMeta {
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputSnapshot {
    pub move_x: f32,
    pub move_y: f32,
    pub aim_x: f32,
    pub aim_y: f32,
    pub fire_held: bool,
    pub ability_pressed: bool,
    pub pause_pressed: bool,
    pub slow_mo_pressed: bool,
    pub frame_step_pressed: bool,
    pub debug_toggle_pressed: bool,
    pub world_rotation_deg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMeta {
    pub frame: u32,
    pub dt: f32,
    pub fps_estimate: f32,
    pub checksum: String,
    pub message: String,
    pub phase: String,
    pub pattern: String,
    pub active_enemy_bullets: usize,
    pub active_player_shots: usize,
    pub active_helpers: usize,
    pub active_objects: usize,
    pub active_generators: usize,
    pub player_x: f32,
    pub player_y: f32,
    pub player_max_hp: f32,
    pub player_hp: f32,
    pub player_mp: f32,
    pub player_max_mp: f32,
    pub player_status_mask: u32,
    pub player_statuses: Vec<StatusView>,
    pub boss_x: f32,
    pub boss_y: f32,
    pub boss_max_hp: f32,
    pub boss_hp: f32,
    pub boss_status_mask: u32,
    pub boss_statuses: Vec<StatusView>,
    pub boss_invulnerable: bool,
    pub boss_armored: bool,
    pub stagger_frames: u16,
    pub shake_amplitude: f32,
    pub shake_frames: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusView {
    pub id: String,
    pub label: String,
    pub frames_left: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderViews {
    pub instance_ptr: u32,
    pub instance_len: u32,
    pub tile_ptr: u32,
    pub tile_len: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub debug_ptr: u32,
    pub debug_len: u32,
    pub event_ptr: u32,
    pub event_len: u32,
    pub floats_per_instance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DebugCommand {
    ToggleOverlay,
    Pause(bool),
    SlowMo(bool),
    Step,
    ToggleHitboxes,
    SeekReplayFrame(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBlob {
    pub version: u32,
    pub seed: u64,
    pub encounter_id: String,
    pub inputs: Vec<InputSnapshot>,
}
