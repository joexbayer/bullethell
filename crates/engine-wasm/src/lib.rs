use std::collections::HashMap;

use schema::{
    AngleMode, ArenaDef, BulletArchetypeDef, BulletBehavior, CommandDef, CompiledContent,
    CONTENT_VERSION, EmitterDef, EmitterSource, EncounterDef, GeneratorElement, HelperMotion,
    ObjectMotion, PatternDef, PatternFamily, PhaseDef, RenderLayer, STATUS_EXPOSED, STATUS_SICK,
    STATUS_SILENCED, STATUS_SLOW,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const MAX_STATUS_SLOTS: usize = 8;
const PLAYER_FIRE_COOLDOWN_FRAMES: u16 = 9;
const PLAYER_SHOT_SPEED: f32 = 14.0;
const PLAYER_SHOT_RADIUS: f32 = 0.18;
const PLAYER_SHOT_TTL_FRAMES: u16 = 48;
const PLAYER_RADIUS: f32 = 0.24;
const PLAYER_RENDER_RADIUS: f32 = 0.18;
const PLAYER_MAX_HP: f32 = 500.0;
const PLAYER_MAX_MP: f32 = 180.0;
const PLAYER_DEF: f32 = 24.0;
const PLAYER_VIT_REGEN: f32 = 6.0;
const PLAYER_WIS_REGEN: f32 = 3.0;
const INSTANCE_FLOATS: usize = 14;

#[derive(Default)]
struct GlobalState {
    game: Option<Game>,
}

thread_local! {
    static STATE: std::cell::RefCell<GlobalState> = std::cell::RefCell::new(GlobalState::default());
}

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

#[derive(Clone)]
struct Game {
    content: CompiledContent,
    _atlas_meta: AtlasMeta,
    _config: GameConfig,
    rng: Rng64,
    runtime: Runtime,
    replay: ReplayState,
}

#[derive(Clone)]
struct ReplayState {
    recording_seed: u64,
    recorded_inputs: Vec<InputSnapshot>,
    playback: Option<ReplayBlob>,
    playback_cursor: usize,
}

impl ReplayState {
    fn new(seed: u64) -> Self {
        Self {
            recording_seed: seed,
            recorded_inputs: Vec::new(),
            playback: None,
            playback_cursor: 0,
        }
    }
}

#[derive(Clone)]
struct Runtime {
    frame: u32,
    accumulator_frames: u32,
    arena: ArenaRuntime,
    patterns: Vec<PatternDef>,
    bullet_archetypes: Vec<BulletArchetypeDef>,
    player: PlayerState,
    boss: BossRuntime,
    encounter_id: String,
    encounter: EncounterDef,
    phase_lookup: HashMap<String, usize>,
    pattern_lookup: HashMap<String, usize>,
    bullet_lookup: HashMap<String, usize>,
    instances: Vec<f32>,
    debug_lines: Vec<f32>,
    debug_enabled: bool,
    debug_hitboxes: bool,
    paused: bool,
    slow_mo: bool,
    fps_estimate: f32,
    current_message: String,
}

#[derive(Clone)]
struct ArenaRuntime {
    arena: ArenaDef,
}

#[derive(Clone)]
struct PlayerState {
    pos_x: f32,
    pos_y: f32,
    hp: f32,
    mp: f32,
    fire_cooldown: u16,
    status_mask: u32,
    statuses: [StatusTimer; MAX_STATUS_SLOTS],
    in_combat_frames: u16,
}

#[derive(Clone)]
struct BossRuntime {
    pos_x: f32,
    pos_y: f32,
    hp: f32,
    max_hp: f32,
    radius: f32,
    status_mask: u32,
    statuses: [StatusTimer; MAX_STATUS_SLOTS],
    phase_index: usize,
    phase_pattern_counter: u32,
    phase_timer: u32,
    fire_pattern_index: usize,
    ice_pattern_index: usize,
    fire_nuke_index: usize,
    ice_nuke_index: usize,
    neutral_index: usize,
    active_pattern: Option<ActivePattern>,
    stagger_frames: u16,
    invulnerable_override: bool,
    armored_override: bool,
    fire_locks: u8,
    ice_locks: u8,
    helper_gates_damage: bool,
    generators: GeneratorPool,
    helpers: HelperPool,
    objects: EncounterObjectPool,
    enemy_bullets: BulletPool,
    player_shots: BulletPool,
}

#[derive(Clone)]
struct ActivePattern {
    pattern_id: String,
    pattern_index: usize,
    frame: u16,
    damage_taken: f32,
}

#[derive(Clone, Copy, Default)]
struct StatusTimer {
    mask: u32,
    frames_left: u16,
}

#[derive(Clone)]
struct HelperPool {
    ids: Vec<String>,
    sprite: Vec<u32>,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    hp: Vec<f32>,
    max_hp: Vec<f32>,
    radius: Vec<f32>,
    motion: Vec<HelperMotion>,
    orbit_radius: Vec<f32>,
    orbit_speed_deg: Vec<f32>,
    angle_deg: Vec<f32>,
    bullet_pattern: Vec<Option<usize>>,
    color_rgba: Vec<[f32; 4]>,
}

impl HelperPool {
    fn new() -> Self {
        Self {
            ids: Vec::new(),
            sprite: Vec::new(),
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            hp: Vec::new(),
            max_hp: Vec::new(),
            radius: Vec::new(),
            motion: Vec::new(),
            orbit_radius: Vec::new(),
            orbit_speed_deg: Vec::new(),
            angle_deg: Vec::new(),
            bullet_pattern: Vec::new(),
            color_rgba: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn find_index(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|existing| existing == id)
    }

    fn contains_id(&self, id: &str) -> bool {
        self.find_index(id).is_some()
    }

    fn remove_id(&mut self, id: &str) -> bool {
        let Some(index) = self.find_index(id) else {
            return false;
        };
        self.swap_remove(index);
        true
    }

    fn push(&mut self, helper: HelperSpawn) {
        self.ids.push(helper.id);
        self.sprite.push(helper.sprite);
        self.pos_x.push(helper.pos_x);
        self.pos_y.push(helper.pos_y);
        self.hp.push(helper.hp);
        self.max_hp.push(helper.hp);
        self.radius.push(helper.radius);
        self.motion.push(helper.motion);
        self.orbit_radius.push(helper.orbit_radius);
        self.orbit_speed_deg.push(helper.orbit_speed_deg);
        self.angle_deg.push(helper.angle_deg);
        self.bullet_pattern.push(helper.bullet_pattern);
        self.color_rgba.push(helper.color_rgba);
    }

    fn swap_remove(&mut self, index: usize) {
        self.ids.swap_remove(index);
        self.sprite.swap_remove(index);
        self.pos_x.swap_remove(index);
        self.pos_y.swap_remove(index);
        self.hp.swap_remove(index);
        self.max_hp.swap_remove(index);
        self.radius.swap_remove(index);
        self.motion.swap_remove(index);
        self.orbit_radius.swap_remove(index);
        self.orbit_speed_deg.swap_remove(index);
        self.angle_deg.swap_remove(index);
        self.bullet_pattern.swap_remove(index);
        self.color_rgba.swap_remove(index);
    }
}

struct HelperSpawn {
    id: String,
    sprite: u32,
    pos_x: f32,
    pos_y: f32,
    hp: f32,
    radius: f32,
    motion: HelperMotion,
    orbit_radius: f32,
    orbit_speed_deg: f32,
    angle_deg: f32,
    bullet_pattern: Option<usize>,
    color_rgba: [f32; 4],
}

#[derive(Clone)]
struct EncounterObjectPool {
    ids: Vec<String>,
    sprite: Vec<u32>,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    hp: Vec<f32>,
    max_hp: Vec<f32>,
    radius: Vec<f32>,
    motion: Vec<ObjectMotion>,
    anchor_x: Vec<f32>,
    anchor_y: Vec<f32>,
    orbit_radius: Vec<f32>,
    orbit_speed_deg: Vec<f32>,
    angle_deg: Vec<f32>,
    bullet_pattern: Vec<Option<usize>>,
    color_rgba: Vec<[f32; 4]>,
}

impl EncounterObjectPool {
    fn new() -> Self {
        Self {
            ids: Vec::new(),
            sprite: Vec::new(),
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            hp: Vec::new(),
            max_hp: Vec::new(),
            radius: Vec::new(),
            motion: Vec::new(),
            anchor_x: Vec::new(),
            anchor_y: Vec::new(),
            orbit_radius: Vec::new(),
            orbit_speed_deg: Vec::new(),
            angle_deg: Vec::new(),
            bullet_pattern: Vec::new(),
            color_rgba: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn find_index(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|existing| existing == id)
    }

    fn contains_id(&self, id: &str) -> bool {
        self.find_index(id).is_some()
    }

    fn remove_id(&mut self, id: &str) -> bool {
        let Some(index) = self.find_index(id) else {
            return false;
        };
        self.swap_remove(index);
        true
    }

    fn push(&mut self, object: EncounterObjectSpawn) {
        self.ids.push(object.id);
        self.sprite.push(object.sprite);
        self.pos_x.push(object.pos_x);
        self.pos_y.push(object.pos_y);
        self.hp.push(object.hp);
        self.max_hp.push(object.hp);
        self.radius.push(object.radius);
        self.motion.push(object.motion);
        self.anchor_x.push(object.anchor_x);
        self.anchor_y.push(object.anchor_y);
        self.orbit_radius.push(object.orbit_radius);
        self.orbit_speed_deg.push(object.orbit_speed_deg);
        self.angle_deg.push(object.angle_deg);
        self.bullet_pattern.push(object.bullet_pattern);
        self.color_rgba.push(object.color_rgba);
    }

    fn swap_remove(&mut self, index: usize) {
        self.ids.swap_remove(index);
        self.sprite.swap_remove(index);
        self.pos_x.swap_remove(index);
        self.pos_y.swap_remove(index);
        self.hp.swap_remove(index);
        self.max_hp.swap_remove(index);
        self.radius.swap_remove(index);
        self.motion.swap_remove(index);
        self.anchor_x.swap_remove(index);
        self.anchor_y.swap_remove(index);
        self.orbit_radius.swap_remove(index);
        self.orbit_speed_deg.swap_remove(index);
        self.angle_deg.swap_remove(index);
        self.bullet_pattern.swap_remove(index);
        self.color_rgba.swap_remove(index);
    }
}

struct EncounterObjectSpawn {
    id: String,
    sprite: u32,
    pos_x: f32,
    pos_y: f32,
    hp: f32,
    radius: f32,
    motion: ObjectMotion,
    anchor_x: f32,
    anchor_y: f32,
    orbit_radius: f32,
    orbit_speed_deg: f32,
    angle_deg: f32,
    bullet_pattern: Option<usize>,
    color_rgba: [f32; 4],
}

#[derive(Clone)]
struct GeneratorPool {
    ids: Vec<String>,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    hp: Vec<f32>,
    max_hp: Vec<f32>,
    radius: Vec<f32>,
    element: Vec<GeneratorElement>,
    sealed: Vec<bool>,
    vulnerable: Vec<bool>,
}

impl GeneratorPool {
    fn new() -> Self {
        Self {
            ids: Vec::new(),
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            hp: Vec::new(),
            max_hp: Vec::new(),
            radius: Vec::new(),
            element: Vec::new(),
            sealed: Vec::new(),
            vulnerable: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.ids.len()
    }

    fn push(&mut self, generator: GeneratorSpawn) {
        self.ids.push(generator.id);
        self.pos_x.push(generator.pos_x);
        self.pos_y.push(generator.pos_y);
        self.hp.push(generator.hp);
        self.max_hp.push(generator.hp);
        self.radius.push(generator.radius);
        self.element.push(generator.element);
        self.sealed.push(generator.sealed);
        self.vulnerable.push(generator.vulnerable);
    }

    fn find_index(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|existing| existing == id)
    }

    fn sealed_count(&self) -> usize {
        self.sealed.iter().filter(|sealed| **sealed).count()
    }
}

struct GeneratorSpawn {
    id: String,
    pos_x: f32,
    pos_y: f32,
    hp: f32,
    radius: f32,
    element: GeneratorElement,
    sealed: bool,
    vulnerable: bool,
}

#[derive(Clone)]
struct BulletPool {
    sprite: Vec<u32>,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    vel_x: Vec<f32>,
    vel_y: Vec<f32>,
    accel_x: Vec<f32>,
    accel_y: Vec<f32>,
    radius: Vec<f32>,
    ttl_frames: Vec<u16>,
    angle_deg: Vec<f32>,
    angular_vel_deg: Vec<f32>,
    archetype_id: Vec<usize>,
    status_mask: Vec<u32>,
    status_duration_frames: Vec<u16>,
    color_rgba: Vec<[f32; 4]>,
    flags: Vec<u32>,
    delay_frames: Vec<u16>,
    damage: Vec<f32>,
    render_layer: Vec<RenderLayer>,
}

impl BulletPool {
    fn new() -> Self {
        Self {
            sprite: Vec::new(),
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            vel_x: Vec::new(),
            vel_y: Vec::new(),
            accel_x: Vec::new(),
            accel_y: Vec::new(),
            radius: Vec::new(),
            ttl_frames: Vec::new(),
            angle_deg: Vec::new(),
            angular_vel_deg: Vec::new(),
            archetype_id: Vec::new(),
            status_mask: Vec::new(),
            status_duration_frames: Vec::new(),
            color_rgba: Vec::new(),
            flags: Vec::new(),
            delay_frames: Vec::new(),
            damage: Vec::new(),
            render_layer: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.pos_x.len()
    }

    fn push(&mut self, bullet: SpawnedBullet) {
        self.sprite.push(bullet.sprite);
        self.pos_x.push(bullet.pos_x);
        self.pos_y.push(bullet.pos_y);
        self.vel_x.push(bullet.vel_x);
        self.vel_y.push(bullet.vel_y);
        self.accel_x.push(bullet.accel_x);
        self.accel_y.push(bullet.accel_y);
        self.radius.push(bullet.radius);
        self.ttl_frames.push(bullet.ttl_frames);
        self.angle_deg.push(bullet.angle_deg);
        self.angular_vel_deg.push(bullet.angular_vel_deg);
        self.archetype_id.push(bullet.archetype_id);
        self.status_mask.push(bullet.status_mask);
        self.status_duration_frames.push(bullet.status_duration_frames);
        self.color_rgba.push(bullet.color_rgba);
        self.flags.push(bullet.flags);
        self.delay_frames.push(bullet.delay_frames);
        self.damage.push(bullet.damage);
        self.render_layer.push(bullet.render_layer);
    }

    fn swap_remove(&mut self, index: usize) {
        self.sprite.swap_remove(index);
        self.pos_x.swap_remove(index);
        self.pos_y.swap_remove(index);
        self.vel_x.swap_remove(index);
        self.vel_y.swap_remove(index);
        self.accel_x.swap_remove(index);
        self.accel_y.swap_remove(index);
        self.radius.swap_remove(index);
        self.ttl_frames.swap_remove(index);
        self.angle_deg.swap_remove(index);
        self.angular_vel_deg.swap_remove(index);
        self.archetype_id.swap_remove(index);
        self.status_mask.swap_remove(index);
        self.status_duration_frames.swap_remove(index);
        self.color_rgba.swap_remove(index);
        self.flags.swap_remove(index);
        self.delay_frames.swap_remove(index);
        self.damage.swap_remove(index);
        self.render_layer.swap_remove(index);
    }
}

struct SpawnedBullet {
    sprite: u32,
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    accel_x: f32,
    accel_y: f32,
    radius: f32,
    ttl_frames: u16,
    angle_deg: f32,
    angular_vel_deg: f32,
    archetype_id: usize,
    status_mask: u32,
    status_duration_frames: u16,
    color_rgba: [f32; 4],
    flags: u32,
    delay_frames: u16,
    damage: f32,
    render_layer: RenderLayer,
}

#[derive(Clone)]
struct Rng64 {
    state: u64,
}

impl Rng64 {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as u32 as f32) / ((1_u32 << 24) as f32)
    }

}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn init_game(content_blob: &[u8], atlas_meta: JsValue, config: JsValue) -> Result<(), JsValue> {
    let content = CompiledContent::decode(content_blob)
        .map_err(|error| JsValue::from_str(&format!("failed to decode content blob: {error}")))?;
    if content.version != CONTENT_VERSION {
        return Err(JsValue::from_str("content version mismatch"));
    }
    let atlas_meta: AtlasMeta = serde_wasm_bindgen::from_value(atlas_meta)
        .map_err(|error| JsValue::from_str(&format!("invalid atlas meta: {error}")))?;
    let config: GameConfig = serde_wasm_bindgen::from_value(config)
        .map_err(|error| JsValue::from_str(&format!("invalid config: {error}")))?;
    let encounter = content
        .encounters
        .first()
        .cloned()
        .ok_or_else(|| JsValue::from_str("no encounters in content"))?;
    let seed = 0xA5A5_4D3C_2B1A_9087;
    let runtime = Runtime::new(&content, encounter.id.clone())?;
    let replay = ReplayState::new(seed);
    STATE.with(|state| {
        state.borrow_mut().game = Some(Game {
            content,
            _atlas_meta: atlas_meta,
            _config: config,
            rng: Rng64::new(seed),
            runtime,
            replay,
        });
    });
    Ok(())
}

#[wasm_bindgen]
pub fn load_encounter(encounter_id: &str) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        game.runtime = Runtime::new(&game.content, encounter_id.to_string())?;
        game.replay.recorded_inputs.clear();
        game.replay.playback = None;
        game.replay.playback_cursor = 0;
        Ok(())
    })
}

#[wasm_bindgen]
pub fn step(input_snapshot: JsValue) -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let mut input: InputSnapshot = serde_wasm_bindgen::from_value(input_snapshot)
            .map_err(|error| JsValue::from_str(&format!("invalid input snapshot: {error}")))?;
        if let Some(playback) = game.replay.playback.as_ref() {
            if game.replay.playback_cursor < playback.inputs.len() {
                input = playback.inputs[game.replay.playback_cursor].clone();
                game.replay.playback_cursor += 1;
            }
        } else {
            game.replay.recorded_inputs.push(input.clone());
        }
        let meta = game.step(input);
        serde_wasm_bindgen::to_value(&meta)
            .map_err(|error| JsValue::from_str(&format!("failed to serialize frame meta: {error}")))
    })
}

#[wasm_bindgen]
pub fn get_render_views() -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        let game = state
            .game
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let views = RenderViews {
            instance_ptr: game.runtime.instances.as_ptr() as u32,
            instance_len: game.runtime.instances.len() as u32,
            tile_ptr: game.runtime.arena.arena.tiles.as_ptr() as u32,
            tile_len: game.runtime.arena.arena.tiles.len() as u32,
            tile_width: game.runtime.arena.arena.width,
            tile_height: game.runtime.arena.arena.height,
            debug_ptr: game.runtime.debug_lines.as_ptr() as u32,
            debug_len: game.runtime.debug_lines.len() as u32,
            floats_per_instance: INSTANCE_FLOATS as u32,
        };
        serde_wasm_bindgen::to_value(&views)
            .map_err(|error| JsValue::from_str(&format!("failed to serialize render views: {error}")))
    })
}

#[wasm_bindgen]
pub fn debug_command(cmd: JsValue) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let cmd: DebugCommand = serde_wasm_bindgen::from_value(cmd)
            .map_err(|error| JsValue::from_str(&format!("invalid debug command: {error}")))?;
        match cmd {
            DebugCommand::ToggleOverlay => game.runtime.debug_enabled = !game.runtime.debug_enabled,
            DebugCommand::Pause(value) => game.runtime.paused = value,
            DebugCommand::SlowMo(value) => game.runtime.slow_mo = value,
            DebugCommand::Step => {
                if game.runtime.paused {
                    game.runtime.advance_one_frame(&mut game.rng);
                }
            }
            DebugCommand::ToggleHitboxes => game.runtime.debug_hitboxes = !game.runtime.debug_hitboxes,
            DebugCommand::SeekReplayFrame(frame) => {
                if let Some(playback) = game.replay.playback.clone() {
                    let encounter_id = playback.encounter_id.clone();
                    game.runtime = Runtime::new(&game.content, encounter_id)?;
                    game.replay.playback = None;
                    game.replay.playback_cursor = 0;
                    for input in playback.inputs.iter().take(frame as usize).cloned() {
                        let _ = game.step(input);
                    }
                    game.replay.playback = Some(playback);
                    game.replay.playback_cursor = frame as usize;
                }
            }
        }
        Ok(())
    })
}

#[wasm_bindgen]
pub fn start_replay(seed: u64) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        game.rng = Rng64::new(seed);
        game.replay = ReplayState::new(seed);
        game.runtime = Runtime::new(&game.content, game.runtime.encounter_id.clone())?;
        Ok(())
    })
}

#[wasm_bindgen]
pub fn load_replay(replay_blob: JsValue) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let replay: ReplayBlob = serde_wasm_bindgen::from_value(replay_blob)
            .map_err(|error| JsValue::from_str(&format!("invalid replay blob: {error}")))?;
        game.rng = Rng64::new(replay.seed);
        game.runtime = Runtime::new(&game.content, replay.encounter_id.clone())?;
        game.replay.playback_cursor = 0;
        game.replay.playback = Some(replay);
        Ok(())
    })
}

#[wasm_bindgen]
pub fn export_replay() -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        let game = state
            .game
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let replay = ReplayBlob {
            version: CONTENT_VERSION,
            seed: game.replay.recording_seed,
            encounter_id: game.runtime.encounter_id.clone(),
            inputs: game.replay.recorded_inputs.clone(),
        };
        serde_wasm_bindgen::to_value(&replay)
            .map_err(|error| JsValue::from_str(&format!("failed to serialize replay: {error}")))
    })
}

impl Game {
    fn step(&mut self, input: InputSnapshot) -> FrameMeta {
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

impl Runtime {
    fn new(content: &CompiledContent, encounter_id: String) -> Result<Self, JsValue> {
        let encounter = content
            .encounters
            .iter()
            .find(|encounter| encounter.id == encounter_id)
            .cloned()
            .ok_or_else(|| JsValue::from_str("encounter not found"))?;
        let arena = content
            .arenas
            .iter()
            .find(|arena| arena.id == encounter.arena_id)
            .cloned()
            .ok_or_else(|| JsValue::from_str("arena not found"))?;
        let phase_lookup = encounter
            .boss
            .phases
            .iter()
            .enumerate()
            .map(|(index, phase)| (phase.id.clone(), index))
            .collect();
        let pattern_lookup = content
            .patterns
            .iter()
            .enumerate()
            .map(|(index, pattern)| (pattern.id.clone(), index))
            .collect();
        let bullet_lookup = content
            .bullet_archetypes
            .iter()
            .enumerate()
            .map(|(index, bullet)| (bullet.id.clone(), index))
            .collect();
        let mut runtime = Self {
            frame: 0,
            accumulator_frames: 0,
            arena: ArenaRuntime { arena },
            patterns: content.patterns.clone(),
            bullet_archetypes: content.bullet_archetypes.clone(),
            player: PlayerState {
                pos_x: encounter
                    .clone()
                    .arena_id
                    .len() as f32, // overwritten below
                pos_y: 0.0,
                hp: PLAYER_MAX_HP,
                mp: PLAYER_MAX_MP,
                fire_cooldown: 0,
                status_mask: 0,
                statuses: [StatusTimer::default(); MAX_STATUS_SLOTS],
                in_combat_frames: 0,
            },
            boss: BossRuntime {
                pos_x: 0.0,
                pos_y: 0.0,
                hp: encounter.boss.hp,
                max_hp: encounter.boss.hp,
                radius: encounter.boss.radius,
                status_mask: 0,
                statuses: [StatusTimer::default(); MAX_STATUS_SLOTS],
                phase_index: 0,
                phase_pattern_counter: 0,
                phase_timer: 0,
                fire_pattern_index: 0,
                ice_pattern_index: 0,
                fire_nuke_index: 0,
                ice_nuke_index: 0,
                neutral_index: 0,
                active_pattern: None,
                stagger_frames: 0,
                invulnerable_override: false,
                armored_override: false,
                fire_locks: 0,
                ice_locks: 0,
                helper_gates_damage: false,
                generators: GeneratorPool::new(),
                helpers: HelperPool::new(),
                objects: EncounterObjectPool::new(),
                enemy_bullets: BulletPool::new(),
                player_shots: BulletPool::new(),
            },
            encounter_id,
            encounter,
            phase_lookup,
            pattern_lookup,
            bullet_lookup,
            instances: Vec::with_capacity(16 * 1024 * INSTANCE_FLOATS),
            debug_lines: Vec::with_capacity(4096),
            debug_enabled: false,
            debug_hitboxes: false,
            paused: false,
            slow_mo: false,
            fps_estimate: 60.0,
            current_message: String::new(),
        };
        runtime.player.pos_x = runtime.arena.arena.player_spawn.x;
        runtime.player.pos_y = runtime.arena.arena.player_spawn.y;
        runtime.boss.pos_x = runtime.arena.arena.boss_spawn.x;
        runtime.boss.pos_y = runtime.arena.arena.boss_spawn.y;
        for generator in runtime.encounter.boss.generators.iter() {
            runtime.boss.generators.push(GeneratorSpawn {
                id: generator.id.clone(),
                pos_x: generator.anchor.x,
                pos_y: generator.anchor.y,
                hp: generator.hp,
                radius: generator.radius,
                element: GeneratorElement::Fire,
                sealed: false,
                vulnerable: false,
            });
        }
        runtime.apply_phase_enter_commands();
        runtime.build_render_data();
        Ok(runtime)
    }

    fn frame_meta(&self) -> FrameMeta {
        FrameMeta {
            frame: self.frame,
            dt: 1.0 / 60.0,
            fps_estimate: self.fps_estimate,
            checksum: format!("{:016x}", self.checksum()),
            message: self.current_message.clone(),
            phase: self.current_phase().id.clone(),
            pattern: self
                .boss
                .active_pattern
                .as_ref()
                .map(|pattern| pattern.pattern_id.clone())
                .unwrap_or_else(|| "idle".to_string()),
            active_enemy_bullets: self.boss.enemy_bullets.len(),
            active_player_shots: self.boss.player_shots.len(),
            active_helpers: self.boss.helpers.len(),
            active_objects: self.boss.objects.len(),
            active_generators: self.boss.generators.len(),
            player_x: self.player.pos_x,
            player_y: self.player.pos_y,
            player_max_hp: PLAYER_MAX_HP,
            player_hp: self.player.hp,
            player_mp: self.player.mp,
            player_max_mp: PLAYER_MAX_MP,
            player_status_mask: self.player.status_mask,
            player_statuses: collect_status_views(&self.player.statuses),
            boss_x: self.boss.pos_x,
            boss_y: self.boss.pos_y,
            boss_max_hp: self.boss.max_hp,
            boss_hp: self.boss.hp,
            boss_status_mask: self.boss.status_mask,
            boss_statuses: collect_status_views(&self.boss.statuses),
            boss_invulnerable: self.boss_is_invulnerable(),
            boss_armored: self.boss_is_armored(),
            stagger_frames: self.boss.stagger_frames,
        }
    }

    fn current_phase(&self) -> &PhaseDef {
        &self.encounter.boss.phases[self.boss.phase_index]
    }

    fn step_frame(&mut self, input: InputSnapshot, rng: &mut Rng64) {
        self.advance_frame_inner(input, rng);
    }

    fn advance_one_frame(&mut self, rng: &mut Rng64) {
        self.advance_frame_inner(InputSnapshot::default(), rng);
    }

    fn advance_frame_inner(&mut self, input: InputSnapshot, rng: &mut Rng64) {
        self.frame += 1;
        self.fps_estimate = 60.0;
        self.phase_tick();
        self.tick_statuses();
        self.update_player(input);
        self.spawn_player_shots();
        self.update_pattern(rng);
        self.update_helpers(rng);
        self.update_objects(rng);
        self.update_bullets();
        self.resolve_collisions();
        self.apply_transitions();
        self.build_render_data();
    }

    fn boss_is_invulnerable(&self) -> bool {
        self.current_phase().invulnerable || self.boss.invulnerable_override
    }

    fn boss_is_armored(&self) -> bool {
        self.current_phase().armored || self.boss.armored_override
    }

    fn phase_tick(&mut self) {
        self.boss.phase_timer += 1;
        if self.boss.stagger_frames > 0 {
            self.boss.stagger_frames -= 1;
        }
    }

    fn tick_statuses(&mut self) {
        tick_status_array(&mut self.player.statuses, &mut self.player.status_mask);
        tick_status_array(&mut self.boss.statuses, &mut self.boss.status_mask);
        if self.player.in_combat_frames > 0 {
            self.player.in_combat_frames -= 1;
        }
        let vit_scale = if self.player.in_combat_frames > 0 { 0.5 } else { 1.0 };
        let can_heal = self.player.status_mask & STATUS_SICK == 0;
        if can_heal {
            self.player.hp = (self.player.hp + PLAYER_VIT_REGEN * vit_scale / 60.0).min(PLAYER_MAX_HP);
        }
        if self.player.status_mask & STATUS_SILENCED == 0 {
            self.player.mp = (self.player.mp + PLAYER_WIS_REGEN * vit_scale / 60.0).min(PLAYER_MAX_MP);
        }
    }

    fn update_player(&mut self, input: InputSnapshot) {
        let mut move_x = input.move_x;
        let mut move_y = input.move_y;
        let length = (move_x * move_x + move_y * move_y).sqrt();
        if length > 1.0 {
            move_x /= length;
            move_y /= length;
        }
        let mut speed = 8.2;
        if self.player.status_mask & STATUS_SLOW != 0 {
            speed = 4.0;
        }
        if self.player.status_mask & STATUS_SILENCED == 0 && input.ability_pressed && self.player.mp >= 16.0 {
            self.player.mp -= 16.0;
            apply_status(&mut self.player.statuses, &mut self.player.status_mask, STATUS_SLOW, 0);
            apply_status(&mut self.player.statuses, &mut self.player.status_mask, STATUS_EXPOSED, 12);
        }
        self.player.pos_x += move_x * speed / 60.0;
        self.player.pos_y += move_y * speed / 60.0;
        resolve_actor_vs_tiles(
            &self.arena.arena,
            &mut self.player.pos_x,
            &mut self.player.pos_y,
            PLAYER_RADIUS,
        );
        if input.fire_held {
            if self.player.fire_cooldown == 0 {
                let aim_dx = input.aim_x - self.player.pos_x;
                let aim_dy = input.aim_y - self.player.pos_y;
                let angle = aim_dy.atan2(aim_dx);
                self.boss.player_shots.push(SpawnedBullet {
                    sprite: 7,
                    pos_x: self.player.pos_x,
                    pos_y: self.player.pos_y,
                    vel_x: angle.cos() * PLAYER_SHOT_SPEED,
                    vel_y: angle.sin() * PLAYER_SHOT_SPEED,
                    accel_x: 0.0,
                    accel_y: 0.0,
                    radius: PLAYER_SHOT_RADIUS,
                    ttl_frames: PLAYER_SHOT_TTL_FRAMES,
                    angle_deg: angle.to_degrees(),
                    angular_vel_deg: 0.0,
                    archetype_id: usize::MAX,
                    status_mask: 0,
                    status_duration_frames: 0,
                    color_rgba: [0.95, 0.95, 1.0, 1.0],
                    flags: 1,
                    delay_frames: 0,
                    damage: 16.0,
                    render_layer: RenderLayer::PlayerShots,
                });
                self.player.fire_cooldown = PLAYER_FIRE_COOLDOWN_FRAMES;
            }
        }
        if self.player.fire_cooldown > 0 {
            self.player.fire_cooldown -= 1;
        }
    }

    fn spawn_player_shots(&mut self) {}

    fn update_pattern(&mut self, rng: &mut Rng64) {
        if self.boss.stagger_frames > 0 {
            return;
        }
        if self.boss.active_pattern.is_none() {
            let pattern_index = self.select_pattern_index(rng);
            self.boss.active_pattern = Some(ActivePattern {
                pattern_id: self.patterns[pattern_index].id.clone(),
                pattern_index,
                frame: 0,
                damage_taken: 0.0,
            });
            self.boss.phase_pattern_counter += 1;
        }
        let Some(mut active) = self.boss.active_pattern.take() else {
            return;
        };
        let pattern_index = active.pattern_index;
        let mut pending_commands = Vec::new();
        {
            let pattern = &self.patterns[pattern_index];
            for command in pattern.commands.iter().filter(|command| command.frame == active.frame) {
                pending_commands.push(command.command.clone());
            }
        }
        for command in pending_commands {
            self.execute_command(command);
        }
        let emit_len = self.patterns[pattern_index].emitters.len();
        for emitter_index in 0..emit_len {
            let should_fire = {
                let emitter = &self.patterns[pattern_index].emitters[emitter_index];
                active.frame >= emitter.start_frame
                    && active.frame <= emitter.end_frame
                    && (active.frame - emitter.start_frame) % emitter.cadence_frames == 0
            };
            if should_fire {
                let emitter = self.patterns[pattern_index].emitters[emitter_index].clone();
                self.fire_emitter(&emitter, active.frame, rng);
            }
        }
        active.frame += 1;
        let interrupted = self.patterns[pattern_index]
            .interruption_damage
            .map(|limit| active.damage_taken >= limit)
            .unwrap_or(false);
        if active.frame >= self.patterns[pattern_index].duration_frames || interrupted {
            self.boss.invulnerable_override = false;
            self.boss.armored_override = false;
            self.boss.active_pattern = None;
        } else {
            self.boss.active_pattern = Some(active);
        }
    }

    fn update_helpers(&mut self, rng: &mut Rng64) {
        for index in 0..self.boss.helpers.len() {
            self.boss.helpers.angle_deg[index] += self.boss.helpers.orbit_speed_deg[index] / 60.0;
            match self.boss.helpers.motion[index] {
                HelperMotion::OrbitBoss => {
                    let radians = self.boss.helpers.angle_deg[index].to_radians();
                    self.boss.helpers.pos_x[index] =
                        self.boss.pos_x + radians.cos() * self.boss.helpers.orbit_radius[index];
                    self.boss.helpers.pos_y[index] =
                        self.boss.pos_y + radians.sin() * self.boss.helpers.orbit_radius[index];
                }
                HelperMotion::CircleArena => {
                    let radians = self.boss.helpers.angle_deg[index].to_radians();
                    self.boss.helpers.pos_x[index] =
                        self.boss.pos_x + radians.cos() * (self.boss.helpers.orbit_radius[index] + 4.5);
                    self.boss.helpers.pos_y[index] =
                        self.boss.pos_y + radians.sin() * (self.boss.helpers.orbit_radius[index] + 4.5);
                }
                HelperMotion::Hover => {}
            }
            if let Some(pattern_index) = self.boss.helpers.bullet_pattern[index] {
                let helper_frame = (self.frame % self.patterns[pattern_index].duration_frames as u32) as u16;
                let emit_len = self.patterns[pattern_index].emitters.len();
                for emitter_index in 0..emit_len {
                    let should_fire = {
                        let emitter = &self.patterns[pattern_index].emitters[emitter_index];
                        emitter.source == EmitterSource::Helper
                            && helper_frame >= emitter.start_frame
                            && helper_frame <= emitter.end_frame
                            && (helper_frame - emitter.start_frame) % emitter.cadence_frames == 0
                    };
                    if should_fire {
                        let emitter = self.patterns[pattern_index].emitters[emitter_index].clone();
                        self.fire_emitter_from_helper(index, &emitter, helper_frame, rng);
                    }
                }
            }
        }
        let mut index = 0;
        while index < self.boss.helpers.len() {
            if self.boss.helpers.hp[index] <= 0.0 {
                self.boss.helpers.swap_remove(index);
                if self.current_phase().helper_gates_damage && !self.has_phase_blockers() {
                    self.boss.stagger_frames = 180;
                    self.current_message = "Helper down: stagger window".to_string();
                }
            } else {
                index += 1;
            }
        }
    }

    fn update_objects(&mut self, rng: &mut Rng64) {
        for index in 0..self.boss.objects.len() {
            self.boss.objects.angle_deg[index] += self.boss.objects.orbit_speed_deg[index] / 60.0;
            match self.boss.objects.motion[index] {
                ObjectMotion::Fixed => {
                    self.boss.objects.pos_x[index] = self.boss.objects.anchor_x[index];
                    self.boss.objects.pos_y[index] = self.boss.objects.anchor_y[index];
                }
                ObjectMotion::OrbitBoss => {
                    let radians = self.boss.objects.angle_deg[index].to_radians();
                    self.boss.objects.pos_x[index] =
                        self.boss.pos_x + radians.cos() * self.boss.objects.orbit_radius[index];
                    self.boss.objects.pos_y[index] =
                        self.boss.pos_y + radians.sin() * self.boss.objects.orbit_radius[index];
                }
                ObjectMotion::CircleArena => {
                    let radians = self.boss.objects.angle_deg[index].to_radians();
                    self.boss.objects.pos_x[index] =
                        self.boss.objects.anchor_x[index] + radians.cos() * self.boss.objects.orbit_radius[index];
                    self.boss.objects.pos_y[index] =
                        self.boss.objects.anchor_y[index] + radians.sin() * self.boss.objects.orbit_radius[index];
                }
            }
            if let Some(pattern_index) = self.boss.objects.bullet_pattern[index] {
                let object_frame = (self.frame % self.patterns[pattern_index].duration_frames as u32) as u16;
                let emit_len = self.patterns[pattern_index].emitters.len();
                for emitter_index in 0..emit_len {
                    let should_fire = {
                        let emitter = &self.patterns[pattern_index].emitters[emitter_index];
                        emitter.source == EmitterSource::Object
                            && object_frame >= emitter.start_frame
                            && object_frame <= emitter.end_frame
                            && (object_frame - emitter.start_frame) % emitter.cadence_frames == 0
                    };
                    if should_fire {
                        let emitter = self.patterns[pattern_index].emitters[emitter_index].clone();
                        self.fire_emitter_from_object(index, &emitter, object_frame, rng);
                    }
                }
            }
        }
        let mut index = 0;
        while index < self.boss.objects.len() {
            if self.boss.objects.hp[index] <= 0.0 {
                self.boss.objects.swap_remove(index);
                if self.current_phase().helper_gates_damage && !self.has_phase_blockers() {
                    self.boss.stagger_frames = 180;
                    self.current_message = "Barrier down: stagger window".to_string();
                }
            } else {
                index += 1;
            }
        }
    }

    fn update_bullets(&mut self) {
        update_bullet_pool(&mut self.boss.enemy_bullets, &self.arena.arena);
        update_bullet_pool(&mut self.boss.player_shots, &self.arena.arena);
    }

    fn has_phase_blockers(&self) -> bool {
        self.boss.helpers.len() > 0 || self.boss.objects.len() > 0
    }

    fn seal_generator(&mut self, index: usize) {
        self.boss.generators.sealed[index] = true;
        self.boss.generators.vulnerable[index] = false;
        self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
        let label = match self.boss.generators.element[index] {
            GeneratorElement::Fire => "Fire",
            GeneratorElement::Ice => "Ice",
        };
        self.current_message = format!("Generator sealed: {label} locked");
    }

    fn apply_legacy_lock_counts_to_generators(&mut self) {
        let mut remaining_fire = self.boss.fire_locks as usize;
        let mut remaining_ice = self.boss.ice_locks as usize;
        for index in 0..self.boss.generators.len() {
            if remaining_fire > 0 {
                self.boss.generators.element[index] = GeneratorElement::Fire;
                self.boss.generators.sealed[index] = true;
                self.boss.generators.vulnerable[index] = false;
                remaining_fire -= 1;
            } else if remaining_ice > 0 {
                self.boss.generators.element[index] = GeneratorElement::Ice;
                self.boss.generators.sealed[index] = true;
                self.boss.generators.vulnerable[index] = false;
                remaining_ice -= 1;
            } else {
                self.boss.generators.sealed[index] = false;
                self.boss.generators.vulnerable[index] = false;
                self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
            }
        }
    }

    fn select_generator_family(&mut self, rng: &mut Rng64) -> (PatternFamily, bool) {
        if self.boss.generators.len() == 0 {
            return select_family(
                self.encounter.boss.generator_count,
                self.boss.fire_locks,
                self.boss.ice_locks,
                rng,
            );
        }
        let mut fire = 0_u8;
        let mut ice = 0_u8;
        for index in 0..self.boss.generators.len() {
            if !self.boss.generators.sealed[index] {
                self.boss.generators.element[index] = if rng.next_f32() > 0.5 {
                    GeneratorElement::Fire
                } else {
                    GeneratorElement::Ice
                };
            }
            match self.boss.generators.element[index] {
                GeneratorElement::Fire => fire += 1,
                GeneratorElement::Ice => ice += 1,
            }
        }
        self.boss.fire_locks = self
            .boss
            .generators
            .element
            .iter()
            .zip(self.boss.generators.sealed.iter())
            .filter(|(element, sealed)| **sealed && **element == GeneratorElement::Fire)
            .count() as u8;
        self.boss.ice_locks = self
            .boss
            .generators
            .element
            .iter()
            .zip(self.boss.generators.sealed.iter())
            .filter(|(element, sealed)| **sealed && **element == GeneratorElement::Ice)
            .count() as u8;
        if fire == ice {
            (PatternFamily::Neutral, false)
        } else if fire > ice {
            (PatternFamily::Fire, fire as usize == self.boss.generators.len())
        } else {
            (PatternFamily::Ice, ice as usize == self.boss.generators.len())
        }
    }

    fn resolve_collisions(&mut self) {
        let mut index = 0;
        while index < self.boss.enemy_bullets.len() {
            if circles_overlap(
                self.boss.enemy_bullets.pos_x[index],
                self.boss.enemy_bullets.pos_y[index],
                self.boss.enemy_bullets.radius[index],
                self.player.pos_x,
                self.player.pos_y,
                PLAYER_RADIUS,
            ) {
                let damage = apply_defense(
                    self.boss.enemy_bullets.damage[index],
                    PLAYER_DEF,
                    self.boss.enemy_bullets.flags[index] & 2 != 0,
                );
                self.player.hp = (self.player.hp - damage).max(0.0);
                self.player.in_combat_frames = 180;
                apply_status(
                    &mut self.player.statuses,
                    &mut self.player.status_mask,
                    self.boss.enemy_bullets.status_mask[index],
                    self.boss.enemy_bullets.status_duration_frames[index],
                );
                self.boss.enemy_bullets.swap_remove(index);
            } else {
                index += 1;
            }
        }

        let boss_is_damageable = self.boss.stagger_frames > 0
            || (!self.current_phase().invulnerable
                && !self.boss.invulnerable_override
                && !(self.current_phase().helper_gates_damage && self.has_phase_blockers()));
        let mut shot_index = 0;
        while shot_index < self.boss.player_shots.len() {
            let mut hit = false;
            let mut generator_index = 0;
            while !hit && generator_index < self.boss.generators.len() {
                if self.boss.generators.vulnerable[generator_index]
                    && !self.boss.generators.sealed[generator_index]
                    && circles_overlap(
                        self.boss.player_shots.pos_x[shot_index],
                        self.boss.player_shots.pos_y[shot_index],
                        self.boss.player_shots.radius[shot_index],
                        self.boss.generators.pos_x[generator_index],
                        self.boss.generators.pos_y[generator_index],
                        self.boss.generators.radius[generator_index],
                    )
                {
                    self.boss.generators.hp[generator_index] -= self.boss.player_shots.damage[shot_index];
                    if self.boss.generators.hp[generator_index] <= 0.0 {
                        self.seal_generator(generator_index);
                    }
                    hit = true;
                }
                generator_index += 1;
            }
            let mut helper_index = 0;
            while helper_index < self.boss.helpers.len() {
                if circles_overlap(
                    self.boss.player_shots.pos_x[shot_index],
                    self.boss.player_shots.pos_y[shot_index],
                    self.boss.player_shots.radius[shot_index],
                    self.boss.helpers.pos_x[helper_index],
                    self.boss.helpers.pos_y[helper_index],
                    self.boss.helpers.radius[helper_index],
                ) {
                    self.boss.helpers.hp[helper_index] -= self.boss.player_shots.damage[shot_index];
                    hit = true;
                    break;
                }
                helper_index += 1;
            }
            let mut object_index = 0;
            while !hit && object_index < self.boss.objects.len() {
                if circles_overlap(
                    self.boss.player_shots.pos_x[shot_index],
                    self.boss.player_shots.pos_y[shot_index],
                    self.boss.player_shots.radius[shot_index],
                    self.boss.objects.pos_x[object_index],
                    self.boss.objects.pos_y[object_index],
                    self.boss.objects.radius[object_index],
                ) {
                    self.boss.objects.hp[object_index] -= self.boss.player_shots.damage[shot_index];
                    hit = true;
                }
                object_index += 1;
            }
            if !hit
                && boss_is_damageable
                && circles_overlap(
                    self.boss.player_shots.pos_x[shot_index],
                    self.boss.player_shots.pos_y[shot_index],
                    self.boss.player_shots.radius[shot_index],
                    self.boss.pos_x,
                    self.boss.pos_y,
                    self.boss.radius,
                )
            {
                let armored = self.current_phase().armored || self.boss.armored_override;
                let exposed = self.boss.status_mask & STATUS_EXPOSED != 0;
                let mut damage = self.boss.player_shots.damage[shot_index];
                if armored {
                    damage *= 0.65;
                }
                if exposed {
                    damage += 6.0;
                }
                self.boss.hp = (self.boss.hp - damage).max(0.0);
                if let Some(active_pattern) = self.boss.active_pattern.as_mut() {
                    active_pattern.damage_taken += damage;
                }
                hit = true;
            }
            if hit {
                self.boss.player_shots.swap_remove(shot_index);
            } else {
                shot_index += 1;
            }
        }
    }

    fn apply_transitions(&mut self) {
        if self.boss.hp <= 0.0 {
            self.current_message = "Boss defeated".to_string();
            self.boss.active_pattern = None;
            self.boss.enemy_bullets = BulletPool::new();
            self.boss.helpers.clear();
            self.boss.objects.clear();
            return;
        }
        let phase = self.current_phase().clone();
        for transition in phase.transitions {
            let matches = match transition.condition {
                schema::TransitionConditionDef::HpBelowRatio(ratio) => {
                    self.boss.hp / self.boss.max_hp <= ratio
                }
                schema::TransitionConditionDef::PatternCountAtLeast(count) => {
                    self.boss.phase_pattern_counter >= count
                }
                schema::TransitionConditionDef::TimerAtLeast(frames) => self.boss.phase_timer >= frames,
                schema::TransitionConditionDef::SealedGeneratorsAtLeast(count) => {
                    self.boss.generators.sealed_count() >= count as usize
                }
                schema::TransitionConditionDef::HelpersDead => self.boss.helpers.len() == 0,
                schema::TransitionConditionDef::ObjectsDead => self.boss.objects.len() == 0,
                schema::TransitionConditionDef::HelperDead(helper_id) => {
                    !self.boss.helpers.contains_id(&helper_id)
                }
                schema::TransitionConditionDef::ObjectDead(object_id) => {
                    !self.boss.objects.contains_id(&object_id)
                }
            };
            if matches {
                self.boss.phase_index = self.phase_lookup[&transition.to_phase];
                self.boss.phase_pattern_counter = 0;
                self.boss.phase_timer = 0;
                self.boss.active_pattern = None;
                self.apply_phase_enter_commands();
                break;
            }
        }
    }

    fn apply_phase_enter_commands(&mut self) {
        self.current_message = format!("Phase: {}", self.current_phase().id);
        self.boss.invulnerable_override = self.current_phase().invulnerable;
        self.boss.armored_override = self.current_phase().armored;
        self.boss.helper_gates_damage = self.current_phase().helper_gates_damage;
        let commands = self.current_phase().enter_commands.clone();
        for command in commands {
            self.execute_command(command);
        }
    }

    fn execute_command(&mut self, command: CommandDef) {
        match command {
            CommandDef::SpawnHelper {
                helper_id,
                sprite,
                hp,
                radius,
                motion,
                orbit_radius,
                orbit_speed_deg,
                bullet_pattern,
                color_rgba,
            } => {
                let bullet_pattern = bullet_pattern.and_then(|id| self.pattern_lookup.get(&id).copied());
                self.boss.helpers.remove_id(&helper_id);
                let spawn = HelperSpawn {
                    id: helper_id,
                    sprite,
                    pos_x: self.boss.pos_x + orbit_radius,
                    pos_y: self.boss.pos_y,
                    hp,
                    radius,
                    motion,
                    orbit_radius,
                    orbit_speed_deg,
                    angle_deg: 0.0,
                    bullet_pattern,
                    color_rgba,
                };
                self.boss.helpers.push(spawn);
            }
            CommandDef::DespawnHelper { helper_id } => {
                self.boss.helpers.remove_id(&helper_id);
            }
            CommandDef::DespawnHelpers => self.boss.helpers.clear(),
            CommandDef::SpawnObject {
                object_id,
                sprite,
                hp,
                radius,
                motion,
                anchor,
                orbit_radius,
                orbit_speed_deg,
                bullet_pattern,
                color_rgba,
            } => {
                let bullet_pattern = bullet_pattern.and_then(|id| self.pattern_lookup.get(&id).copied());
                self.boss.objects.remove_id(&object_id);
                let spawn = EncounterObjectSpawn {
                    id: object_id,
                    sprite,
                    pos_x: anchor.x,
                    pos_y: anchor.y,
                    hp,
                    radius,
                    motion,
                    anchor_x: anchor.x,
                    anchor_y: anchor.y,
                    orbit_radius,
                    orbit_speed_deg,
                    angle_deg: 0.0,
                    bullet_pattern,
                    color_rgba,
                };
                self.boss.objects.push(spawn);
            }
            CommandDef::DespawnObject { object_id } => {
                self.boss.objects.remove_id(&object_id);
            }
            CommandDef::SetGeneratorsVulnerable(value) => {
                for index in 0..self.boss.generators.len() {
                    if !self.boss.generators.sealed[index] {
                        self.boss.generators.vulnerable[index] = value;
                        if value {
                            self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
                        }
                    }
                }
            }
            CommandDef::SetGeneratorElement {
                generator_id,
                element,
            } => {
                if let Some(index) = self.boss.generators.find_index(&generator_id) {
                    self.boss.generators.element[index] = element;
                }
            }
            CommandDef::DespawnObjects => self.boss.objects.clear(),
            CommandDef::SetBossInvulnerable(value) => self.boss.invulnerable_override = value,
            CommandDef::SetBossArmored(value) => self.boss.armored_override = value,
            CommandDef::SetElementLocks {
                fire_locks,
                ice_locks,
            } => {
                self.boss.fire_locks = fire_locks;
                self.boss.ice_locks = ice_locks;
                self.apply_legacy_lock_counts_to_generators();
            }
            CommandDef::SetMessage(message) => self.current_message = message,
            CommandDef::StartStagger { frames } => self.boss.stagger_frames = frames,
            CommandDef::SetArenaShake { amplitude, frames } => {
                self.current_message = format!("shake {:.1} for {}", amplitude, frames);
            }
        }
    }

    fn select_pattern_index(&mut self, rng: &mut Rng64) -> usize {
        let selector = self.current_phase().selector.clone();
        let (family, nuke) = self.select_generator_family(rng);
        let patterns = match (family, nuke) {
            (PatternFamily::Fire, false) if !selector.fire_patterns.is_empty() => {
                let index = self.boss.fire_pattern_index % selector.fire_patterns.len();
                self.boss.fire_pattern_index += 1;
                &selector.fire_patterns[index]
            }
            (PatternFamily::Ice, false) if !selector.ice_patterns.is_empty() => {
                let index = self.boss.ice_pattern_index % selector.ice_patterns.len();
                self.boss.ice_pattern_index += 1;
                &selector.ice_patterns[index]
            }
            (PatternFamily::Fire, true) if !selector.fire_nuke_patterns.is_empty() => {
                let index = self.boss.fire_nuke_index % selector.fire_nuke_patterns.len();
                self.boss.fire_nuke_index += 1;
                &selector.fire_nuke_patterns[index]
            }
            (PatternFamily::Ice, true) if !selector.ice_nuke_patterns.is_empty() => {
                let index = self.boss.ice_nuke_index % selector.ice_nuke_patterns.len();
                self.boss.ice_nuke_index += 1;
                &selector.ice_nuke_patterns[index]
            }
            _ => {
                let index = self.boss.neutral_index % selector.neutral_patterns.len().max(1);
                self.boss.neutral_index += 1;
                selector
                    .neutral_patterns
                    .get(index)
                    .or_else(|| selector.fire_patterns.first())
                    .or_else(|| selector.ice_patterns.first())
                    .expect("phase selector must contain at least one pattern")
            }
        };
        self.pattern_lookup[patterns]
    }

    fn fire_emitter(&mut self, emitter: &EmitterDef, frame: u16, rng: &mut Rng64) {
        let origin = match emitter.source {
            EmitterSource::Boss => (self.boss.pos_x, self.boss.pos_y),
            EmitterSource::ArenaTop => (self.player.pos_x, self.arena.arena.camera_bounds.min_y + 1.0),
            EmitterSource::ArenaBottom => (self.player.pos_x, self.arena.arena.camera_bounds.max_y - 1.0),
            EmitterSource::ArenaLeft => (self.arena.arena.camera_bounds.min_x + 1.0, self.player.pos_y),
            EmitterSource::ArenaRight => (self.arena.arena.camera_bounds.max_x - 1.0, self.player.pos_y),
            EmitterSource::Helper | EmitterSource::Object => return,
        };
        self.spawn_emitter_burst(origin.0, origin.1, emitter, frame, rng);
    }

    fn fire_emitter_from_helper(
        &mut self,
        helper_index: usize,
        emitter: &EmitterDef,
        frame: u16,
        rng: &mut Rng64,
    ) {
        let origin = (
            self.boss.helpers.pos_x[helper_index],
            self.boss.helpers.pos_y[helper_index],
        );
        self.spawn_emitter_burst(origin.0, origin.1, emitter, frame, rng);
    }

    fn fire_emitter_from_object(
        &mut self,
        object_index: usize,
        emitter: &EmitterDef,
        frame: u16,
        rng: &mut Rng64,
    ) {
        let origin = (
            self.boss.objects.pos_x[object_index],
            self.boss.objects.pos_y[object_index],
        );
        self.spawn_emitter_burst(origin.0, origin.1, emitter, frame, rng);
    }

    fn spawn_emitter_burst(
        &mut self,
        origin_x: f32,
        origin_y: f32,
        emitter: &EmitterDef,
        frame: u16,
        _rng: &mut Rng64,
    ) {
        let archetype_index = self.bullet_lookup[&emitter.bullet_id];
        let archetype = self.bullet_archetypes[archetype_index].clone();
        let burst_count = emitter.burst_count.max(1) as usize;
        let spread_total = emitter.spread_deg;
        for shot_index in 0..burst_count {
            let angle_deg = compute_angle_deg(
                emitter,
                shot_index,
                burst_count,
                spread_total,
                frame,
                self.player.pos_x - origin_x,
                self.player.pos_y - origin_y,
            );
            let speed_scale = match emitter.speed_mode {
                schema::SpeedMode::Constant => 1.0,
                schema::SpeedMode::RampByBurstIndex => {
                    1.0 + emitter.speed_scale_step * shot_index as f32
                }
            };
            let angle_rad = angle_deg.to_radians();
            let mut angular_vel_deg = 0.0;
            let mut flags = 0_u32;
            match archetype.behavior {
                BulletBehavior::TurnAfterDelay => angular_vel_deg = archetype.turn_rate_deg,
                BulletBehavior::CircleAfterDelay => angular_vel_deg = archetype.turn_rate_deg,
                BulletBehavior::Orbit => {
                    angular_vel_deg = archetype.turn_rate_deg;
                    flags |= 4;
                }
                BulletBehavior::Boomerang => flags |= 8,
                BulletBehavior::AccelerateAfterDelay | BulletBehavior::Default => {}
            }
            if archetype.armor_piercing {
                flags |= 2;
            }
            if archetype.die_on_wall {
                flags |= 1;
            }
            let color_rgba = projectile_color(archetype.status_mask, archetype.color_rgba);
            self.boss.enemy_bullets.push(SpawnedBullet {
                sprite: archetype.sprite,
                pos_x: origin_x,
                pos_y: origin_y,
                vel_x: angle_rad.cos() * archetype.speed * speed_scale,
                vel_y: angle_rad.sin() * archetype.speed * speed_scale,
                accel_x: angle_rad.cos() * archetype.accel,
                accel_y: angle_rad.sin() * archetype.accel,
                radius: archetype.radius,
                ttl_frames: archetype.lifetime_frames,
                angle_deg,
                angular_vel_deg,
                archetype_id: archetype_index,
                status_mask: archetype.status_mask,
                status_duration_frames: archetype.status_duration_frames,
                color_rgba,
                flags,
                delay_frames: archetype.delay_frames,
                damage: archetype.damage,
                render_layer: archetype.render_layer,
            });
        }
    }

    fn build_render_data(&mut self) {
        self.instances.clear();
        self.debug_lines.clear();
        for (index, tile_kind) in self.arena.arena.tiles.iter().enumerate() {
            if *tile_kind == 0 {
                continue;
            }
            let tile_x = index as u32 % self.arena.arena.width;
            let tile_y = index as u32 / self.arena.arena.width;
            let x = tile_x as f32 * self.arena.arena.tile_size;
            let y = tile_y as f32 * self.arena.arena.tile_size;
            let is_edge_wall = *tile_kind == 1
                && (tile_x == 0
                    || tile_y == 0
                    || tile_x == self.arena.arena.width - 1
                    || tile_y == self.arena.arena.height - 1);
            let color = match tile_kind {
                1 if is_edge_wall => [0.92, 0.95, 1.0, 1.0],
                1 => [0.62, 0.70, 0.84, 1.0],
                2 => [0.80, 0.42, 0.56, 1.0],
                _ => [0.40, 0.46, 0.54, 1.0],
            };
            let sprite = match tile_kind {
                1 if is_edge_wall => 9,
                1 => 0,
                2 => 9,
                _ => 0,
            };
            push_instance(
                &mut self.instances,
                x + self.arena.arena.tile_size * 0.5,
                y + self.arena.arena.tile_size * 0.5,
                self.arena.arena.tile_size,
                self.arena.arena.tile_size,
                0.0,
                sprite,
                color,
                0.0,
                1.0,
                1.0,
            );
        }
        for index in 0..self.boss.generators.len() {
            let base_color = match self.boss.generators.element[index] {
                GeneratorElement::Fire => [1.0, 0.56, 0.20, 1.0],
                GeneratorElement::Ice => [0.58, 0.88, 1.0, 1.0],
            };
            let alpha = if self.boss.generators.sealed[index] { 0.42 } else { 0.95 };
            let ring_color = [base_color[0], base_color[1], base_color[2], alpha];
            let core_scale = if self.boss.generators.sealed[index] { 0.70 } else { 0.92 };
            push_instance(
                &mut self.instances,
                self.boss.generators.pos_x[index],
                self.boss.generators.pos_y[index],
                self.boss.generators.radius[index] * 2.4,
                self.boss.generators.radius[index] * 2.4,
                0.0,
                13,
                ring_color,
                0.8,
                1.0,
                0.0,
            );
            push_instance(
                &mut self.instances,
                self.boss.generators.pos_x[index],
                self.boss.generators.pos_y[index],
                self.boss.generators.radius[index] * 2.0 * core_scale,
                self.boss.generators.radius[index] * 2.0 * core_scale,
                45.0,
                3,
                if self.boss.generators.vulnerable[index] && !self.boss.generators.sealed[index] {
                    [base_color[0], base_color[1], base_color[2], 1.0]
                } else {
                    [base_color[0] * 0.72, base_color[1] * 0.72, base_color[2] * 0.72, alpha]
                },
                0.82,
                1.0,
                0.0,
            );
            if self.boss.generators.vulnerable[index] && !self.boss.generators.sealed[index] {
                let hp_ratio = (self.boss.generators.hp[index] / self.boss.generators.max_hp[index]).clamp(0.0, 1.0);
                let bar_y = self.boss.generators.pos_y[index] + self.boss.generators.radius[index] + 0.36;
                let bar_width = 1.3;
                let bar_height = 0.16;
            push_instance(
                &mut self.instances,
                self.boss.generators.pos_x[index],
                bar_y,
                bar_width,
                bar_height,
                0.0,
                14,
                [0.10, 0.12, 0.18, 0.96],
                0.84,
                1.0,
                0.0,
            );
                let fill_width = (bar_width - 0.08) * hp_ratio;
                if fill_width > 0.0 {
                    push_instance(
                        &mut self.instances,
                        self.boss.generators.pos_x[index] - (bar_width - 0.08) * 0.5 + fill_width * 0.5,
                        bar_y,
                        fill_width,
                        bar_height - 0.04,
                        0.0,
                        14,
                        [0.94, 0.98, 0.88, 1.0],
                        0.86,
                        1.0,
                        0.0,
                    );
                }
            }
        }
        for index in 0..self.boss.enemy_bullets.len() {
            let render_diameter = self.boss.enemy_bullets.radius[index] * 2.7;
            push_instance(
                &mut self.instances,
                self.boss.enemy_bullets.pos_x[index],
                self.boss.enemy_bullets.pos_y[index],
                render_diameter,
                render_diameter,
                self.boss.enemy_bullets.angle_deg[index],
                self.boss.enemy_bullets.sprite[index],
                self.boss.enemy_bullets.color_rgba[index],
                1.0,
                1.0,
                0.0,
            );
        }
        for index in 0..self.boss.player_shots.len() {
            push_instance(
                &mut self.instances,
                self.boss.player_shots.pos_x[index],
                self.boss.player_shots.pos_y[index],
                self.boss.player_shots.radius[index] * 2.0,
                self.boss.player_shots.radius[index] * 2.0,
                self.boss.player_shots.angle_deg[index],
                self.boss.player_shots.sprite[index],
                self.boss.player_shots.color_rgba[index],
                2.0,
                1.0,
                0.0,
            );
        }
        for index in 0..self.boss.helpers.len() {
            let bar_width = 1.0;
            let bar_height = 0.12;
            let bar_inner_width = bar_width - 0.08;
            let helper_rotation = self.boss.helpers.angle_deg[index];
            let helper_rotation_rad = helper_rotation.to_radians();
            let bar_offset = self.boss.helpers.radius[index] + 0.28;
            let bar_center_x = self.boss.helpers.pos_x[index] - helper_rotation_rad.sin() * bar_offset;
            let bar_center_y = self.boss.helpers.pos_y[index] + helper_rotation_rad.cos() * bar_offset;
            let hp_ratio = (self.boss.helpers.hp[index] / self.boss.helpers.max_hp[index]).clamp(0.0, 1.0);
            push_instance(
                &mut self.instances,
                bar_center_x,
                bar_center_y,
                bar_width,
                bar_height,
                helper_rotation,
                14,
                [0.10, 0.12, 0.18, 0.95],
                3.06,
                1.0,
                0.0,
            );
            push_instance(
                &mut self.instances,
                bar_center_x,
                bar_center_y,
                bar_inner_width,
                bar_height - 0.03,
                helper_rotation,
                14,
                [0.08, 0.10, 0.12, 0.96],
                3.07,
                1.0,
                0.0,
            );
            let fill_width = bar_inner_width * hp_ratio;
            if fill_width > 0.0 {
                let fill_center_shift = -(bar_inner_width - fill_width) * 0.5;
                let fill_center_x = bar_center_x + helper_rotation_rad.cos() * fill_center_shift;
                let fill_center_y = bar_center_y + helper_rotation_rad.sin() * fill_center_shift;
                push_instance(
                    &mut self.instances,
                    fill_center_x,
                    fill_center_y,
                    fill_width,
                    bar_height - 0.03,
                    helper_rotation,
                    14,
                    [0.18, 0.92, 0.40, 1.0],
                    3.08,
                    1.0,
                    0.0,
                );
            }
            push_instance(
                &mut self.instances,
                self.boss.helpers.pos_x[index],
                self.boss.helpers.pos_y[index],
                self.boss.helpers.radius[index] * 2.0,
                self.boss.helpers.radius[index] * 2.0,
                self.boss.helpers.angle_deg[index],
                self.boss.helpers.sprite[index],
                self.boss.helpers.color_rgba[index],
                3.0,
                1.0,
                0.0,
            );
        }
        for index in 0..self.boss.objects.len() {
            let bar_width = 1.45;
            let bar_height = 0.18;
            let bar_inner_width = bar_width - 0.08;
            let object_rotation = self.boss.objects.angle_deg[index];
            let object_rotation_rad = object_rotation.to_radians();
            let bar_offset = self.boss.objects.radius[index] + 0.40;
            let bar_center_x = self.boss.objects.pos_x[index] - object_rotation_rad.sin() * bar_offset;
            let bar_center_y = self.boss.objects.pos_y[index] + object_rotation_rad.cos() * bar_offset;
            let hp_ratio = (self.boss.objects.hp[index] / self.boss.objects.max_hp[index]).clamp(0.0, 1.0);
            push_instance(
                &mut self.instances,
                bar_center_x,
                bar_center_y,
                bar_width,
                bar_height,
                object_rotation,
                14,
                [0.07, 0.12, 0.09, 0.98],
                3.34,
                1.0,
                0.0,
            );
            push_instance(
                &mut self.instances,
                bar_center_x,
                bar_center_y,
                bar_inner_width,
                bar_height - 0.05,
                object_rotation,
                14,
                [0.06, 0.09, 0.07, 0.98],
                3.35,
                1.0,
                0.0,
            );
            let fill_width = bar_inner_width * hp_ratio;
            if fill_width > 0.0 {
                let fill_center_shift = -(bar_inner_width - fill_width) * 0.5;
                let fill_center_x = bar_center_x + object_rotation_rad.cos() * fill_center_shift;
                let fill_center_y = bar_center_y + object_rotation_rad.sin() * fill_center_shift;
                push_instance(
                    &mut self.instances,
                    fill_center_x,
                    fill_center_y,
                    fill_width,
                    bar_height - 0.05,
                    object_rotation,
                    14,
                    [0.14, 0.96, 0.28, 1.0],
                    3.36,
                    1.0,
                    0.0,
                );
            }
            push_instance(
                &mut self.instances,
                self.boss.objects.pos_x[index],
                self.boss.objects.pos_y[index],
                self.boss.objects.radius[index] * 2.0,
                self.boss.objects.radius[index] * 2.0,
                self.boss.objects.angle_deg[index],
                self.boss.objects.sprite[index],
                self.boss.objects.color_rgba[index],
                3.3,
                1.0,
                0.0,
            );
        }
        if self.boss_is_invulnerable() || self.boss_is_armored() {
            let pulse = 1.0 + (self.frame as f32 / 8.0).sin() * 0.05;
            let ring_color = if self.boss_is_invulnerable() {
                [1.0, 0.94, 0.62, 0.92]
            } else {
                [0.74, 0.88, 1.0, 0.88]
            };
            push_instance(
                &mut self.instances,
                self.boss.pos_x,
                self.boss.pos_y,
                self.boss.radius * 2.9 * pulse,
                self.boss.radius * 2.9 * pulse,
                self.frame as f32 * 2.5,
                13,
                ring_color,
                3.95,
                1.0,
                0.0,
            );
        }
        push_instance(
            &mut self.instances,
            self.boss.pos_x,
            self.boss.pos_y,
            self.boss.radius * 2.0,
            self.boss.radius * 2.0,
            0.0,
            5,
            [0.42, 0.24, 0.60, 1.0],
            4.0,
            1.0,
            0.0,
        );
        push_instance(
            &mut self.instances,
            self.player.pos_x,
            self.player.pos_y,
            PLAYER_RENDER_RADIUS * 2.0,
            PLAYER_RENDER_RADIUS * 2.0,
            0.0,
            6,
            [0.90, 0.96, 1.0, 1.0],
            5.0,
            1.0,
            0.0,
        );
        if self.debug_enabled && self.debug_hitboxes {
            push_circle_debug(&mut self.debug_lines, self.player.pos_x, self.player.pos_y, PLAYER_RADIUS);
            push_circle_debug(&mut self.debug_lines, self.boss.pos_x, self.boss.pos_y, self.boss.radius);
            for index in 0..self.boss.helpers.len() {
                push_circle_debug(
                    &mut self.debug_lines,
                    self.boss.helpers.pos_x[index],
                    self.boss.helpers.pos_y[index],
                    self.boss.helpers.radius[index],
                );
            }
            for index in 0..self.boss.objects.len() {
                push_circle_debug(
                    &mut self.debug_lines,
                    self.boss.objects.pos_x[index],
                    self.boss.objects.pos_y[index],
                    self.boss.objects.radius[index],
                );
            }
            for index in 0..self.boss.generators.len() {
                if self.boss.generators.vulnerable[index] && !self.boss.generators.sealed[index] {
                    push_circle_debug(
                        &mut self.debug_lines,
                        self.boss.generators.pos_x[index],
                        self.boss.generators.pos_y[index],
                        self.boss.generators.radius[index],
                    );
                }
            }
        }
    }

    fn checksum(&self) -> u64 {
        let mut value = self.frame as u64 ^ self.boss.enemy_bullets.len() as u64;
        value ^= (self.player.hp.to_bits() as u64) << 1;
        value ^= (self.boss.hp.to_bits() as u64) << 2;
        for index in 0..self.boss.enemy_bullets.len().min(32) {
            value = value
                .wrapping_mul(0x9E3779B185EBCA87)
                .wrapping_add(self.boss.enemy_bullets.pos_x[index].to_bits() as u64)
                .wrapping_add((self.boss.enemy_bullets.pos_y[index].to_bits() as u64) << 1);
        }
        value
    }

}

fn resolve_actor_vs_tiles(arena: &ArenaDef, pos_x: &mut f32, pos_y: &mut f32, radius: f32) {
        let tile_size = arena.tile_size;
        let min_x = ((*pos_x - radius) / tile_size).floor().max(0.0) as i32;
        let max_x = ((*pos_x + radius) / tile_size)
            .floor()
            .min(arena.width as f32 - 1.0) as i32;
        let min_y = ((*pos_y - radius) / tile_size).floor().max(0.0) as i32;
        let max_y = ((*pos_y + radius) / tile_size)
            .floor()
            .min(arena.height as f32 - 1.0) as i32;
        for ty in min_y..=max_y {
            for tx in min_x..=max_x {
                let index = ty as usize * arena.width as usize + tx as usize;
                if tile_bit_is_set(&arena.collision_words, index) {
                    let tile_min_x = tx as f32 * tile_size;
                    let tile_min_y = ty as f32 * tile_size;
                    let tile_max_x = tile_min_x + tile_size;
                    let tile_max_y = tile_min_y + tile_size;
                    let nearest_x = (*pos_x).clamp(tile_min_x, tile_max_x);
                    let nearest_y = (*pos_y).clamp(tile_min_y, tile_max_y);
                    let delta_x = *pos_x - nearest_x;
                    let delta_y = *pos_y - nearest_y;
                    let distance_sq = delta_x * delta_x + delta_y * delta_y;
                    if distance_sq < radius * radius && distance_sq > 0.0 {
                        let distance = distance_sq.sqrt();
                        let push = radius - distance;
                        *pos_x += delta_x / distance * push;
                        *pos_y += delta_y / distance * push;
                    }
                }
            }
        }
        *pos_x = (*pos_x).clamp(
            arena.camera_bounds.min_x + radius,
            arena.camera_bounds.max_x - radius,
        );
        *pos_y = (*pos_y).clamp(
            arena.camera_bounds.min_y + radius,
            arena.camera_bounds.max_y - radius,
        );
    }

fn update_bullet_pool(pool: &mut BulletPool, arena: &ArenaDef) {
    let mut index = 0;
    while index < pool.len() {
        if pool.ttl_frames[index] > 0 {
            pool.ttl_frames[index] -= 1;
        }
        let delayed = pool.delay_frames[index] > 0;
        if delayed {
            pool.delay_frames[index] -= 1;
        } else {
            if pool.angular_vel_deg[index] != 0.0 {
                pool.angle_deg[index] += pool.angular_vel_deg[index] / 60.0;
                let speed = (pool.vel_x[index] * pool.vel_x[index] + pool.vel_y[index] * pool.vel_y[index]).sqrt();
                let angle_rad = pool.angle_deg[index].to_radians();
                pool.vel_x[index] = angle_rad.cos() * speed;
                pool.vel_y[index] = angle_rad.sin() * speed;
            }
            pool.vel_x[index] += pool.accel_x[index] / 60.0;
            pool.vel_y[index] += pool.accel_y[index] / 60.0;
        }
        pool.pos_x[index] += pool.vel_x[index] / 60.0;
        pool.pos_y[index] += pool.vel_y[index] / 60.0;
        let out_of_bounds = pool.pos_x[index] < arena.camera_bounds.min_x - 1.0
            || pool.pos_x[index] > arena.camera_bounds.max_x + 1.0
            || pool.pos_y[index] < arena.camera_bounds.min_y - 1.0
            || pool.pos_y[index] > arena.camera_bounds.max_y + 1.0;
        let wall_hit = bullet_hits_wall(pool.pos_x[index], pool.pos_y[index], pool.radius[index], arena);
        let die_on_wall = pool.flags[index] & 1 != 0;
        if pool.ttl_frames[index] == 0 || out_of_bounds || (wall_hit && die_on_wall) {
            pool.swap_remove(index);
        } else {
            index += 1;
        }
    }
}

fn tick_status_array(statuses: &mut [StatusTimer; MAX_STATUS_SLOTS], status_mask: &mut u32) {
    *status_mask = 0;
    for status in statuses.iter_mut() {
        if status.frames_left > 0 {
            status.frames_left -= 1;
            *status_mask |= status.mask;
        }
    }
}

fn apply_status(
    statuses: &mut [StatusTimer; MAX_STATUS_SLOTS],
    status_mask: &mut u32,
    incoming_mask: u32,
    duration_frames: u16,
) {
    if incoming_mask == 0 {
        return;
    }
    for bit in 0..32 {
        let mask = 1_u32 << bit;
        if incoming_mask & mask == 0 {
            continue;
        }
        if let Some(slot) = statuses.iter_mut().find(|slot| slot.mask == mask || slot.frames_left == 0) {
            slot.mask = mask;
            slot.frames_left = duration_frames.max(slot.frames_left);
        }
        *status_mask |= mask;
    }
}

fn projectile_color(status_mask: u32, base: [f32; 4]) -> [f32; 4] {
    if status_mask & STATUS_SLOW != 0 {
        return [0.44, 0.90, 1.0, base[3]];
    }
    if status_mask & STATUS_SICK != 0 {
        return [0.58, 1.0, 0.36, base[3]];
    }
    if status_mask & STATUS_SILENCED != 0 {
        return [0.84, 0.60, 1.0, base[3]];
    }
    if status_mask & STATUS_EXPOSED != 0 {
        return [1.0, 0.88, 0.36, base[3]];
    }
    base
}

fn collect_status_views(statuses: &[StatusTimer; MAX_STATUS_SLOTS]) -> Vec<StatusView> {
    let mut views = Vec::new();
    for status in statuses.iter().filter(|status| status.frames_left > 0) {
        views.push(StatusView {
            id: status_id(status.mask).to_string(),
            label: status_label(status.mask).to_string(),
            frames_left: status.frames_left,
        });
    }
    views.sort_by(|a, b| a.label.cmp(&b.label));
    views
}

fn status_id(mask: u32) -> &'static str {
    match mask {
        STATUS_SLOW => "slow",
        STATUS_SICK => "sick",
        STATUS_SILENCED => "silenced",
        schema::STATUS_ARMOR_BROKEN => "armor_broken",
        STATUS_EXPOSED => "exposed",
        schema::STATUS_INVULNERABLE => "invulnerable",
        schema::STATUS_ARMORED => "armored",
        _ => "unknown",
    }
}

fn status_label(mask: u32) -> &'static str {
    match mask {
        STATUS_SLOW => "Slow",
        STATUS_SICK => "Sick",
        STATUS_SILENCED => "Silenced",
        schema::STATUS_ARMOR_BROKEN => "Armor Broken",
        STATUS_EXPOSED => "Exposed",
        schema::STATUS_INVULNERABLE => "Invulnerable",
        schema::STATUS_ARMORED => "Armored",
        _ => "Unknown",
    }
}

fn push_instance(
    instances: &mut Vec<f32>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation_deg: f32,
    sprite: u32,
    color: [f32; 4],
    layer: f32,
    world_rotate: f32,
    world_spin: f32,
) {
    let screen_lock = if sprite == 6 { 1.0 } else { 0.0 };
    instances.extend_from_slice(&[
        x,
        y,
        width,
        height,
        rotation_deg,
        sprite as f32,
        color[0],
        color[1],
        color[2],
        color[3],
        layer,
        world_rotate,
        world_spin,
        screen_lock,
    ]);
}

fn push_circle_debug(lines: &mut Vec<f32>, center_x: f32, center_y: f32, radius: f32) {
    let segments = 24;
    for index in 0..segments {
        let a0 = index as f32 / segments as f32 * std::f32::consts::TAU;
        let a1 = (index + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        lines.extend_from_slice(&[
            center_x + a0.cos() * radius,
            center_y + a0.sin() * radius,
            center_x + a1.cos() * radius,
            center_y + a1.sin() * radius,
        ]);
    }
}

fn select_family(generator_count: u8, fire_locks: u8, ice_locks: u8, rng: &mut Rng64) -> (PatternFamily, bool) {
    let unlocked = generator_count.saturating_sub(fire_locks + ice_locks);
    let mut fire = fire_locks;
    let mut ice = ice_locks;
    for _ in 0..unlocked {
        if rng.next_f32() > 0.5 {
            fire += 1;
        } else {
            ice += 1;
        }
    }
    if fire == ice {
        (PatternFamily::Neutral, false)
    } else if fire > ice {
        (PatternFamily::Fire, fire == generator_count)
    } else {
        (PatternFamily::Ice, ice == generator_count)
    }
}

fn compute_angle_deg(
    emitter: &EmitterDef,
    shot_index: usize,
    burst_count: usize,
    spread_total: f32,
    frame: u16,
    player_dx: f32,
    player_dy: f32,
) -> f32 {
    let center_angle = match emitter.angle_mode {
        AngleMode::AimAtPlayer => player_dy.atan2(player_dx).to_degrees() + emitter.base_angle_deg,
        AngleMode::Fixed => emitter.base_angle_deg,
        AngleMode::Spin => emitter.base_angle_deg + frame as f32 * emitter.spin_speed_deg / 60.0,
        AngleMode::Radial => emitter.base_angle_deg + frame as f32 * emitter.spin_speed_deg / 120.0,
    };
    if burst_count == 1 {
        center_angle
    } else {
        let offset = shot_index as f32 / (burst_count - 1) as f32 - 0.5;
        center_angle + offset * spread_total
    }
}

fn circles_overlap(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    let radius = ar + br;
    dx * dx + dy * dy <= radius * radius
}

fn tile_bit_is_set(words: &[u64], index: usize) -> bool {
    words
        .get(index / 64)
        .map(|word| (word >> (index % 64)) & 1 == 1)
        .unwrap_or(false)
}

fn bullet_hits_wall(x: f32, y: f32, radius: f32, arena: &ArenaDef) -> bool {
    let tile_size = arena.tile_size;
    let min_x = ((x - radius) / tile_size).floor().max(0.0) as i32;
    let max_x = ((x + radius) / tile_size).floor().min(arena.width as f32 - 1.0) as i32;
    let min_y = ((y - radius) / tile_size).floor().max(0.0) as i32;
    let max_y = ((y + radius) / tile_size).floor().min(arena.height as f32 - 1.0) as i32;
    for ty in min_y..=max_y {
        for tx in min_x..=max_x {
            let index = ty as usize * arena.width as usize + tx as usize;
            if tile_bit_is_set(&arena.collision_words, index) {
                let tile_min_x = tx as f32 * tile_size;
                let tile_min_y = ty as f32 * tile_size;
                let tile_max_x = tile_min_x + tile_size;
                let tile_max_y = tile_min_y + tile_size;
                let nearest_x = x.clamp(tile_min_x, tile_max_x);
                let nearest_y = y.clamp(tile_min_y, tile_max_y);
                let delta_x = x - nearest_x;
                let delta_y = y - nearest_y;
                if delta_x * delta_x + delta_y * delta_y < radius * radius {
                    return true;
                }
            }
        }
    }
    false
}

fn apply_defense(damage: f32, defense: f32, armor_piercing: bool) -> f32 {
    if armor_piercing {
        return damage;
    }
    (damage - defense).max(damage * 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::{
        AttackSelectorDef, AuthorArenaDef, AuthorRoot, BossDef, BulletArchetypeDef, EncounterDef,
        GeneratorDef, PatternDef, PhaseDef, RectDef, TransitionDef, TransitionConditionDef, Vec2Def,
    };

    fn test_runtime() -> Runtime {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![BulletArchetypeDef {
                id: "shot".to_string(),
                sprite: 1,
                radius: 0.2,
                damage: 10.0,
                lifetime_frames: 10,
                speed: 6.0,
                accel: 0.0,
                turn_rate_deg: 90.0,
                delay_frames: 0,
                status_mask: STATUS_SLOW,
                status_duration_frames: 90,
                behavior: BulletBehavior::Default,
                render_layer: RenderLayer::EnemyBullets,
                color_rgba: [1.0, 0.2, 0.2, 1.0],
                die_on_wall: true,
                armor_piercing: false,
            }],
            patterns: vec![PatternDef {
                id: "pattern".to_string(),
                family: PatternFamily::Fire,
                nuke: false,
                duration_frames: 30,
                interruption_damage: Some(20.0),
                emitters: vec![EmitterDef {
                    source: EmitterSource::Boss,
                    cadence_frames: 1,
                    start_frame: 0,
                    end_frame: 1,
                    burst_count: 1,
                    spread_deg: 0.0,
                    base_angle_deg: 180.0,
                    angle_mode: AngleMode::Fixed,
                    spin_speed_deg: 0.0,
                    speed_mode: schema::SpeedMode::Constant,
                    speed_scale_step: 0.0,
                    bullet_id: "shot".to_string(),
                }],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 3,
                    generators: vec![],
                    phases: vec![PhaseDef {
                        id: "phase".to_string(),
                        invulnerable: false,
                        armored: false,
                        helper_gates_damage: false,
                        selector: AttackSelectorDef {
                            fire_patterns: vec!["pattern".to_string()],
                            ice_patterns: vec!["pattern".to_string()],
                            fire_nuke_patterns: vec![],
                            ice_nuke_patterns: vec![],
                            neutral_patterns: vec![],
                        },
                        enter_commands: vec![],
                        transitions: vec![TransitionDef {
                            condition: TransitionConditionDef::HpBelowRatio(0.0),
                            to_phase: "phase".to_string(),
                        }],
                    }],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        Runtime::new(&compiled, "encounter".to_string()).unwrap()
    }

    #[test]
    fn ttl_cleanup_removes_bullets() {
        let mut runtime = test_runtime();
        let mut rng = Rng64::new(7);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert!(runtime.boss.enemy_bullets.len() > 0);
        for _ in 0..20 {
            runtime.step_frame(InputSnapshot::default(), &mut rng);
        }
        assert_eq!(runtime.boss.enemy_bullets.len(), 0);
    }

    #[test]
    fn deterministic_checksum_matches() {
        let mut a = test_runtime();
        let mut b = test_runtime();
        let mut rng_a = Rng64::new(42);
        let mut rng_b = Rng64::new(42);
        for _ in 0..10 {
            a.step_frame(InputSnapshot::default(), &mut rng_a);
            b.step_frame(InputSnapshot::default(), &mut rng_b);
        }
        assert_eq!(a.checksum(), b.checksum());
    }

    #[test]
    fn tile_collision_keeps_player_out_of_wall() {
        let runtime = test_runtime();
        let mut x = 0.4;
        let mut y = 0.4;
        resolve_actor_vs_tiles(&runtime.arena.arena, &mut x, &mut y, PLAYER_RADIUS);
        assert!(x >= PLAYER_RADIUS);
        assert!(y >= PLAYER_RADIUS);
    }

    #[test]
    fn bullet_seam_query_does_not_hit_adjacent_wall() {
        let compiled = schema::compile_author_root(AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "#####".to_string(),
                    "#...#".to_string(),
                    "#...#".to_string(),
                    "#.###".to_string(),
                    "#####".to_string(),
                ],
                player_spawn: Vec2Def { x: 1.5, y: 1.5 },
                boss_spawn: Vec2Def { x: 2.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 5.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![PhaseDef {
                        id: "phase".to_string(),
                        invulnerable: false,
                        armored: false,
                        helper_gates_damage: false,
                        selector: AttackSelectorDef {
                            fire_patterns: vec![],
                            ice_patterns: vec![],
                            fire_nuke_patterns: vec![],
                            ice_nuke_patterns: vec![],
                            neutral_patterns: vec!["placeholder".to_string()],
                        },
                        enter_commands: vec![],
                        transitions: vec![],
                    }],
                },
            }],
        });
        let arena = &compiled.arenas[0];
        assert!(!bullet_hits_wall(2.0, 2.0, 0.18, arena));
    }

    #[test]
    fn phase_entry_keeps_helpers_without_explicit_despawn() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![
                        PhaseDef {
                            id: "phase_a".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SpawnHelper {
                                helper_id: "bird".to_string(),
                                sprite: 1,
                                hp: 50.0,
                                radius: 0.4,
                                motion: HelperMotion::Hover,
                                orbit_radius: 0.0,
                                orbit_speed_deg: 0.0,
                                bullet_pattern: None,
                                color_rgba: [1.0, 1.0, 1.0, 1.0],
                            }],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::TimerAtLeast(1),
                                to_phase: "phase_b".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "phase_b".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "phase_b");
        assert_eq!(runtime.boss.helpers.len(), 1);
        assert_eq!(runtime.boss.helpers.ids[0], "bird");
    }

    #[test]
    fn spawn_object_replaces_existing_id() {
        let mut runtime = test_runtime();
        runtime.execute_command(CommandDef::SpawnObject {
            object_id: "gate".to_string(),
            sprite: 1,
            hp: 10.0,
            radius: 0.4,
            motion: ObjectMotion::Fixed,
            anchor: Vec2Def { x: 5.0, y: 2.0 },
            orbit_radius: 0.0,
            orbit_speed_deg: 0.0,
            bullet_pattern: None,
            color_rgba: [1.0, 0.0, 0.0, 1.0],
        });
        runtime.execute_command(CommandDef::SpawnObject {
            object_id: "gate".to_string(),
            sprite: 2,
            hp: 20.0,
            radius: 0.5,
            motion: ObjectMotion::Fixed,
            anchor: Vec2Def { x: 6.0, y: 2.0 },
            orbit_radius: 0.0,
            orbit_speed_deg: 0.0,
            bullet_pattern: None,
            color_rgba: [0.0, 0.0, 1.0, 1.0],
        });
        assert_eq!(runtime.boss.objects.len(), 1);
        assert_eq!(runtime.boss.objects.ids[0], "gate");
        assert_eq!(runtime.boss.objects.sprite[0], 2);
        assert_eq!(runtime.boss.objects.max_hp[0], 20.0);
    }

    #[test]
    fn objects_dead_transition_advances_phase() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![
                        PhaseDef {
                            id: "gate".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: true,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SpawnObject {
                                object_id: "seal".to_string(),
                                sprite: 1,
                                hp: 10.0,
                                radius: 0.4,
                                motion: ObjectMotion::Fixed,
                                anchor: Vec2Def { x: 5.0, y: 2.0 },
                                orbit_radius: 0.0,
                                orbit_speed_deg: 0.0,
                                bullet_pattern: None,
                                color_rgba: [1.0, 0.0, 0.0, 1.0],
                            }],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::ObjectsDead,
                                to_phase: "next".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "next".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        runtime.boss.objects.hp[0] = 0.0;
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "next");
    }

    #[test]
    fn sealing_generator_advances_phase() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 1,
                    generators: vec![GeneratorDef {
                        id: "gen".to_string(),
                        anchor: Vec2Def { x: 4.0, y: 2.0 },
                        hp: 10.0,
                        radius: 0.4,
                    }],
                    phases: vec![
                        PhaseDef {
                            id: "seal".to_string(),
                            invulnerable: true,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SetGeneratorsVulnerable(true)],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::SealedGeneratorsAtLeast(1),
                                to_phase: "next".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "next".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SetGeneratorsVulnerable(false)],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        runtime.boss.generators.hp[0] = 0.0;
        runtime.seal_generator(0);
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "next");
        assert!(runtime.boss.generators.sealed[0]);
        assert!(!runtime.boss.generators.vulnerable[0]);
    }
}
