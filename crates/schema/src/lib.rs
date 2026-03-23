use serde::{Deserialize, Serialize};

pub const CONTENT_VERSION: u32 = 1;
pub const STATUS_SLOW: u32 = 1 << 0;
pub const STATUS_SICK: u32 = 1 << 1;
pub const STATUS_SILENCED: u32 = 1 << 2;
pub const STATUS_ARMOR_BROKEN: u32 = 1 << 3;
pub const STATUS_EXPOSED: u32 = 1 << 4;
pub const STATUS_INVULNERABLE: u32 = 1 << 5;
pub const STATUS_ARMORED: u32 = 1 << 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorRoot {
    pub arenas: Vec<AuthorArenaDef>,
    pub bullet_archetypes: Vec<BulletArchetypeDef>,
    pub patterns: Vec<PatternDef>,
    pub encounters: Vec<EncounterDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorArenaDef {
    pub id: String,
    pub tile_size: f32,
    pub rows: Vec<String>,
    pub player_spawn: Vec2Def,
    pub boss_spawn: Vec2Def,
    pub camera_bounds: RectDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectDef {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaDef {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub tiles: Vec<u8>,
    pub collision_words: Vec<u64>,
    pub hazard_words: Vec<u64>,
    pub player_spawn: Vec2Def,
    pub boss_spawn: Vec2Def,
    pub camera_bounds: RectDef,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternFamily {
    Fire,
    Ice,
    Neutral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderLayer {
    FloorFx,
    EnemyBullets,
    PlayerShots,
    Helpers,
    Boss,
    Player,
    OverlayFx,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmitterSource {
    Boss,
    Helper,
    Object,
    ArenaTop,
    ArenaBottom,
    ArenaLeft,
    ArenaRight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AngleMode {
    AimAtPlayer,
    Fixed,
    Spin,
    Radial,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpeedMode {
    Constant,
    RampByBurstIndex,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BulletBehavior {
    Default,
    TurnAfterDelay,
    CircleAfterDelay,
    AccelerateAfterDelay,
    Orbit,
    Boomerang,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HelperMotion {
    OrbitBoss,
    CircleArena,
    Hover,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObjectMotion {
    Fixed,
    OrbitBoss,
    CircleArena,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeneratorElement {
    Fire,
    Ice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorDef {
    pub id: String,
    pub anchor: Vec2Def,
    pub hp: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletArchetypeDef {
    pub id: String,
    pub sprite: u32,
    pub radius: f32,
    pub damage: f32,
    pub lifetime_frames: u16,
    pub speed: f32,
    pub accel: f32,
    pub turn_rate_deg: f32,
    pub delay_frames: u16,
    pub status_mask: u32,
    pub status_duration_frames: u16,
    pub behavior: BulletBehavior,
    pub render_layer: RenderLayer,
    pub color_rgba: [f32; 4],
    pub die_on_wall: bool,
    pub armor_piercing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterDef {
    pub source: EmitterSource,
    pub cadence_frames: u16,
    pub start_frame: u16,
    pub end_frame: u16,
    pub burst_count: u16,
    pub spread_deg: f32,
    pub base_angle_deg: f32,
    pub angle_mode: AngleMode,
    pub spin_speed_deg: f32,
    pub speed_mode: SpeedMode,
    pub speed_scale_step: f32,
    pub bullet_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandDef {
    SpawnHelper {
        helper_id: String,
        sprite: u32,
        hp: f32,
        radius: f32,
        motion: HelperMotion,
        orbit_radius: f32,
        orbit_speed_deg: f32,
        bullet_pattern: Option<String>,
        color_rgba: [f32; 4],
    },
    DespawnHelper {
        helper_id: String,
    },
    SpawnObject {
        object_id: String,
        sprite: u32,
        hp: f32,
        radius: f32,
        motion: ObjectMotion,
        anchor: Vec2Def,
        orbit_radius: f32,
        orbit_speed_deg: f32,
        bullet_pattern: Option<String>,
        color_rgba: [f32; 4],
    },
    SetGeneratorsVulnerable(bool),
    SetGeneratorElement {
        generator_id: String,
        element: GeneratorElement,
    },
    DespawnObject {
        object_id: String,
    },
    DespawnHelpers,
    DespawnObjects,
    SetBossInvulnerable(bool),
    SetBossArmored(bool),
    SetElementLocks {
        fire_locks: u8,
        ice_locks: u8,
    },
    SetMessage(String),
    StartStagger {
        frames: u16,
    },
    SetArenaShake {
        amplitude: f32,
        frames: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedCommandDef {
    pub frame: u16,
    pub command: CommandDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDef {
    pub id: String,
    pub family: PatternFamily,
    pub nuke: bool,
    pub duration_frames: u16,
    pub interruption_damage: Option<f32>,
    pub emitters: Vec<EmitterDef>,
    pub commands: Vec<TimedCommandDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionConditionDef {
    HpBelowRatio(f32),
    PatternCountAtLeast(u32),
    TimerAtLeast(u32),
    SealedGeneratorsAtLeast(u8),
    HelpersDead,
    ObjectsDead,
    HelperDead(String),
    ObjectDead(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSelectorDef {
    pub fire_patterns: Vec<String>,
    pub ice_patterns: Vec<String>,
    pub fire_nuke_patterns: Vec<String>,
    pub ice_nuke_patterns: Vec<String>,
    pub neutral_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDef {
    pub id: String,
    pub invulnerable: bool,
    pub armored: bool,
    pub helper_gates_damage: bool,
    pub selector: AttackSelectorDef,
    pub enter_commands: Vec<CommandDef>,
    pub transitions: Vec<TransitionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    pub condition: TransitionConditionDef,
    pub to_phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossDef {
    pub hp: f32,
    pub radius: f32,
    pub generator_count: u8,
    pub generators: Vec<GeneratorDef>,
    pub phases: Vec<PhaseDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterDef {
    pub id: String,
    pub arena_id: String,
    pub boss: BossDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContent {
    pub version: u32,
    pub arenas: Vec<ArenaDef>,
    pub bullet_archetypes: Vec<BulletArchetypeDef>,
    pub patterns: Vec<PatternDef>,
    pub encounters: Vec<EncounterDef>,
}

impl CompiledContent {
    pub fn encode(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::error::DecodeError> {
        let (decoded, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(decoded)
    }
}

pub fn parse_author_root(text: &str) -> Result<AuthorRoot, ron::error::SpannedError> {
    ron::from_str(text)
}

pub fn compile_author_root(root: AuthorRoot) -> CompiledContent {
    let arenas = root
        .arenas
        .into_iter()
        .map(|arena| {
            let height = arena.rows.len() as u32;
            let width = arena.rows.first().map(|row| row.len()).unwrap_or(0) as u32;
            let mut tiles = Vec::with_capacity((width * height) as usize);
            let mut collision_words = vec![0_u64; ((width * height) as usize).div_ceil(64)];
            let hazard_words = vec![0_u64; ((width * height) as usize).div_ceil(64)];
            for (y, row) in arena.rows.iter().enumerate() {
                for (x, ch) in row.bytes().enumerate() {
                    let index = y * width as usize + x;
                    match ch {
                        b'#' => {
                            tiles.push(1);
                            collision_words[index / 64] |= 1_u64 << (index % 64);
                        }
                        b'~' => tiles.push(2),
                        _ => tiles.push(0),
                    }
                }
            }
            ArenaDef {
                id: arena.id,
                width,
                height,
                tile_size: arena.tile_size,
                tiles,
                collision_words,
                hazard_words,
                player_spawn: arena.player_spawn,
                boss_spawn: arena.boss_spawn,
                camera_bounds: arena.camera_bounds,
            }
        })
        .collect();
    CompiledContent {
        version: CONTENT_VERSION,
        arenas,
        bullet_archetypes: root.bullet_archetypes,
        patterns: root.patterns,
        encounters: root.encounters,
    }
}
