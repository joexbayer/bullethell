use schema::RenderLayer;

// --- Player ---
pub const PLAYER_SPEED: f32 = 8.2;
pub const PLAYER_SPEED_SLOWED: f32 = 4.0;
pub const PLAYER_FIRE_COOLDOWN_FRAMES: u16 = 9;
pub const PLAYER_SHOT_SPEED: f32 = 14.0;
pub const PLAYER_SHOT_RADIUS: f32 = 0.18;
pub const PLAYER_SHOT_TTL_FRAMES: u16 = 48;
pub const PLAYER_SHOT_DAMAGE: f32 = 16.0;
pub const PLAYER_SHOT_LAYER: RenderLayer = RenderLayer::PlayerShots;
pub const PLAYER_RADIUS: f32 = 0.24;
pub const PLAYER_RENDER_RADIUS: f32 = 0.18;
pub const PLAYER_MAX_HP: f32 = 500.0;
pub const PLAYER_MAX_MP: f32 = 180.0;
pub const PLAYER_DEF: f32 = 24.0;
pub const PLAYER_VIT_REGEN: f32 = 6.0;
pub const PLAYER_WIS_REGEN: f32 = 3.0;
pub const PLAYER_ABILITY_COST: f32 = 16.0;
pub const PLAYER_IN_COMBAT_DURATION: u16 = 180;
pub const PLAYER_ABILITY_EXPOSED_FRAMES: u16 = 12;

// --- Sprites ---
pub const SPRITE_TILE: u32 = 0;
pub const SPRITE_GENERATOR_CORE: u32 = 3;
pub const SPRITE_RING: u32 = 4;
pub const SPRITE_BOSS: u32 = 5;
pub const SPRITE_PLAYER: u32 = 6;
pub const SPRITE_PLAYER_SHOT: u32 = 7;
pub const SPRITE_EDGE_WALL: u32 = 9;
pub const SPRITE_GENERATOR_RING: u32 = 13;
pub const SPRITE_UI_RECT: u32 = 14;

// --- Rendering ---
pub const INSTANCE_FLOATS: usize = 16;
pub const MAX_STATUS_SLOTS: usize = 8;

// --- Combat ---
pub const ARMOR_REDUCTION: f32 = 0.65;
pub const EXPOSED_BONUS_DAMAGE: f32 = 6.0;
pub const STAGGER_FRAMES_DEFAULT: u16 = 180;
pub const REGEN_COMBAT_SCALE: f32 = 0.5;

// --- Events (type, x, y, r, g, b, extra) ---
pub const EVENT_FLOATS: usize = 7;
pub const EVENT_BULLET_HIT_PLAYER: f32 = 1.0;
pub const EVENT_SHOT_HIT_ENEMY: f32 = 2.0;
pub const EVENT_HELPER_DEATH: f32 = 3.0;
pub const EVENT_OBJECT_DEATH: f32 = 4.0;
pub const EVENT_GENERATOR_SEALED: f32 = 5.0;
pub const EVENT_BOSS_DEATH: f32 = 6.0;

// --- Bullet Flags ---
pub const BULLET_FLAG_DIE_ON_WALL: u32 = 1;
pub const BULLET_FLAG_ARMOR_PIERCING: u32 = 2;
pub const BULLET_FLAG_ORBIT: u32 = 4;
pub const BULLET_FLAG_BOOMERANG: u32 = 8;
pub const BULLET_FLAG_IS_PLAYER_SHOT: u32 = 1;
