use std::collections::HashMap;

use schema::{
    ArenaDef, BulletArchetypeDef, CompiledContent, EncounterDef, GeneratorElement, PatternDef,
    PhaseDef,
};
use wasm_bindgen::prelude::*;

use crate::constants::*;
use crate::pool::bullet::BulletPool;
use crate::pool::generator::{GeneratorPool, GeneratorSpawn};
use crate::pool::helper::HelperPool;
use crate::pool::object::EncounterObjectPool;
use crate::rng::Rng64;
use crate::types::{FrameMeta, InputSnapshot};

pub mod boss;
pub mod bullet_update;
pub mod collision;
pub mod emitter;
pub mod generators;
pub mod helpers;
pub mod player;
pub mod render;
pub mod status;

#[derive(Clone)]
pub struct Runtime {
    pub frame: u32,
    pub accumulator_frames: u32,
    pub arena: ArenaRuntime,
    pub patterns: Vec<PatternDef>,
    pub bullet_archetypes: Vec<BulletArchetypeDef>,
    pub player: player::PlayerState,
    pub boss: boss::BossRuntime,
    pub encounter_id: String,
    pub encounter: EncounterDef,
    pub phase_lookup: HashMap<String, usize>,
    pub pattern_lookup: HashMap<String, usize>,
    pub bullet_lookup: HashMap<String, usize>,
    pub instances: Vec<f32>,
    pub debug_lines: Vec<f32>,
    pub events: Vec<f32>,
    pub debug_enabled: bool,
    pub debug_hitboxes: bool,
    pub paused: bool,
    pub slow_mo: bool,
    pub fps_estimate: f32,
    pub current_message: String,
    pub shake_amplitude: f32,
    pub shake_frames: u16,
    pub world_rotation_deg: f32,
}

#[derive(Clone)]
pub struct ArenaRuntime {
    pub arena: ArenaDef,
}

impl Runtime {
    pub fn new(content: &CompiledContent, encounter_id: String) -> Result<Self, JsValue> {
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
            player: player::PlayerState {
                pos_x: 0.0,
                pos_y: 0.0,
                hp: PLAYER_MAX_HP,
                mp: PLAYER_MAX_MP,
                fire_cooldown: 0,
                status_mask: 0,
                statuses: [status::StatusTimer::default(); MAX_STATUS_SLOTS],
                in_combat_frames: 0,
            },
            boss: boss::BossRuntime {
                pos_x: 0.0,
                pos_y: 0.0,
                hp: encounter.boss.hp,
                max_hp: encounter.boss.hp,
                radius: encounter.boss.radius,
                status_mask: 0,
                statuses: [status::StatusTimer::default(); MAX_STATUS_SLOTS],
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
                enemy_bullets: BulletPool::with_capacity(8192),
                player_shots: BulletPool::with_capacity(512),
            },
            encounter_id,
            encounter,
            phase_lookup,
            pattern_lookup,
            bullet_lookup,
            instances: Vec::with_capacity(16 * 1024 * INSTANCE_FLOATS),
            debug_lines: Vec::with_capacity(4096),
            events: Vec::with_capacity(256),
            debug_enabled: false,
            debug_hitboxes: false,
            paused: false,
            slow_mo: false,
            fps_estimate: 60.0,
            current_message: String::new(),
            shake_amplitude: 0.0,
            shake_frames: 0,
            world_rotation_deg: 0.0,
        };
        runtime.player.pos_x = runtime.arena.arena.player_spawn.x;
        runtime.player.pos_y = runtime.arena.arena.player_spawn.y;
        runtime.boss.pos_x = runtime.arena.arena.boss_spawn.x;
        runtime.boss.pos_y = runtime.arena.arena.boss_spawn.y;
        for generator in runtime.encounter.boss.generators.iter() {
            runtime.boss.generators.push(GeneratorSpawn {
                ids: generator.id.clone(),
                pos_x: generator.anchor.x,
                pos_y: generator.anchor.y,
                hp: generator.hp,
                max_hp: generator.hp,
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

    pub fn current_phase(&self) -> &PhaseDef {
        &self.encounter.boss.phases[self.boss.phase_index]
    }

    pub fn step_frame(&mut self, input: InputSnapshot, rng: &mut Rng64) {
        self.advance_frame_inner(input, rng);
    }

    pub fn advance_one_frame(&mut self, rng: &mut Rng64) {
        self.advance_frame_inner(InputSnapshot::default(), rng);
    }

    fn advance_frame_inner(&mut self, input: InputSnapshot, rng: &mut Rng64) {
        self.frame += 1;
        self.fps_estimate = 60.0;
        self.world_rotation_deg = input.world_rotation_deg;
        self.events.clear();
        self.phase_tick();
        self.tick_statuses();
        self.update_player(input);
        self.update_pattern(rng);
        self.update_helpers(rng);
        self.update_objects(rng);
        self.update_bullets();
        self.resolve_collisions();
        self.apply_transitions();
        self.build_render_data();
    }

    pub fn frame_meta(&self) -> FrameMeta {
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
                .map(|active| self.patterns[active.pattern_index].id.clone())
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
            player_statuses: status::collect_status_views(&self.player.statuses),
            boss_x: self.boss.pos_x,
            boss_y: self.boss.pos_y,
            boss_max_hp: self.boss.max_hp,
            boss_hp: self.boss.hp,
            boss_status_mask: self.boss.status_mask,
            boss_statuses: status::collect_status_views(&self.boss.statuses),
            boss_invulnerable: self.boss_is_invulnerable(),
            boss_armored: self.boss_is_armored(),
            stagger_frames: self.boss.stagger_frames,
            shake_amplitude: self.shake_amplitude,
            shake_frames: self.shake_frames,
        }
    }

    pub fn boss_is_invulnerable(&self) -> bool {
        self.current_phase().invulnerable || self.boss.invulnerable_override
    }

    pub fn boss_is_armored(&self) -> bool {
        self.current_phase().armored || self.boss.armored_override
    }

    fn phase_tick(&mut self) {
        self.boss.phase_timer += 1;
        if self.boss.stagger_frames > 0 {
            self.boss.stagger_frames -= 1;
        }
        if self.shake_frames > 0 {
            self.shake_frames -= 1;
            if self.shake_frames == 0 {
                self.shake_amplitude = 0.0;
            }
        }
    }

    pub fn checksum(&self) -> u64 {
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
