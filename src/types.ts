export interface InputSnapshot {
  move_x: number;
  move_y: number;
  aim_x: number;
  aim_y: number;
  fire_held: boolean;
  ability_pressed: boolean;
  pause_pressed: boolean;
  slow_mo_pressed: boolean;
  frame_step_pressed: boolean;
  debug_toggle_pressed: boolean;
  world_rotation_deg: number;
}

export interface FrameMeta {
  frame: number;
  dt: number;
  fps_estimate: number;
  checksum: string;
  message: string;
  phase: string;
  pattern: string;
  active_enemy_bullets: number;
  active_player_shots: number;
  active_helpers: number;
  active_objects: number;
  active_generators: number;
  player_x: number;
  player_y: number;
  player_max_hp: number;
  player_hp: number;
  player_mp: number;
  player_max_mp: number;
  player_status_mask: number;
  player_statuses: StatusView[];
  boss_x: number;
  boss_y: number;
  boss_max_hp: number;
  boss_hp: number;
  boss_status_mask: number;
  boss_statuses: StatusView[];
  boss_invulnerable: boolean;
  boss_armored: boolean;
  stagger_frames: number;
  shake_amplitude: number;
  shake_frames: number;
}

export interface StatusView {
  id: string;
  label: string;
  frames_left: number;
}

export interface RenderViews {
  instance_ptr: number;
  instance_len: number;
  tile_ptr: number;
  tile_len: number;
  tile_width: number;
  tile_height: number;
  debug_ptr: number;
  debug_len: number;
  event_ptr: number;
  event_len: number;
  floats_per_instance: number;
}

export interface AtlasMeta {
  cols: number;
  rows: number;
}
