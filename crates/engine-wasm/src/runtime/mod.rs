use std::collections::HashMap;

use schema::{
    ArenaDef, BulletArchetypeDef, CompiledContent, EncounterDef, GeneratorElement, PatternDef,
    PatternFamily, PhaseDef,
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
pub mod archmage;
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
                support_delay_frames: 0,
                damage_window_frames: 0,
                invulnerable_override: false,
                armored_override: false,
                fire_locks: 0,
                ice_locks: 0,
                last_pattern_family: schema::PatternFamily::Neutral,
                last_pattern_nuke: false,
                duel_majority: schema::PatternFamily::Neutral,
                duel_stage: 0,
                helper_gates_damage: false,
                generators: GeneratorPool::new(),
                helpers: HelperPool::new(),
                objects: EncounterObjectPool::new(),
                pending_helper_respawns: Vec::new(),
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
            active_helpers: self
                .boss
                .helpers
                .transition_state
                .iter()
                .zip(self.boss.helpers.invulnerable.iter())
                .filter(|(state, invulnerable)| **state == ENTITY_STATE_ACTIVE && !**invulnerable)
                .count(),
            active_objects: self
                .boss
                .objects
                .transition_state
                .iter()
                .filter(|state| **state == ENTITY_STATE_ACTIVE)
                .count(),
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
            support_delay_frames: self.boss.support_delay_frames,
            damage_window_frames: self.boss.damage_window_frames,
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
        if self.boss.support_delay_frames > 0 {
            self.boss.support_delay_frames -= 1;
            if self.boss.support_delay_frames == 0 {
                self.begin_support_damage_window();
            }
        }
        if self.boss.damage_window_frames > 0 {
            self.boss.damage_window_frames -= 1;
            self.boss.stagger_frames = self.boss.damage_window_frames;
        } else if self.boss.stagger_frames > 0 {
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

    pub fn debug_jump_phase(&mut self, target: &str) -> Result<(), JsValue> {
        if self.encounter_id == "twilight_archmage_v1" {
            self.debug_jump_archmage_phase(target)?;
        } else {
            let phase_index = self
                .phase_lookup
                .get(target)
                .copied()
                .ok_or_else(|| JsValue::from_str("unknown phase"))?;
            self.boss.phase_index = phase_index;
            self.boss.phase_pattern_counter = 0;
            self.boss.phase_timer = 0;
            self.boss.active_pattern = None;
            self.apply_phase_enter_commands();
        }
        self.build_render_data();
        Ok(())
    }

    fn debug_jump_archmage_phase(&mut self, target: &str) -> Result<(), JsValue> {
        let (phase_id, hp_ratio, locks) = match target {
            "opening" => ("opening", 1.0, &[][..]),
            "single_bird" => ("single_bird", 0.65, &[GeneratorElement::Fire][..]),
            "dual_guard" => (
                "dual_guard",
                0.35,
                &[GeneratorElement::Fire, GeneratorElement::Ice][..],
            ),
            "duel" | "duel_fire" => (
                "duel",
                0.20,
                &[
                    GeneratorElement::Fire,
                    GeneratorElement::Fire,
                    GeneratorElement::Ice,
                ][..],
            ),
            "duel_ice" => (
                "duel",
                0.20,
                &[
                    GeneratorElement::Ice,
                    GeneratorElement::Ice,
                    GeneratorElement::Fire,
                ][..],
            ),
            "finale" | "finale_fire" => (
                "finale",
                0.12,
                &[
                    GeneratorElement::Fire,
                    GeneratorElement::Fire,
                    GeneratorElement::Ice,
                ][..],
            ),
            "finale_ice" => (
                "finale",
                0.12,
                &[
                    GeneratorElement::Ice,
                    GeneratorElement::Ice,
                    GeneratorElement::Fire,
                ][..],
            ),
            _ => return Err(JsValue::from_str("unknown archmage phase preset")),
        };

        let phase_index = self
            .phase_lookup
            .get(phase_id)
            .copied()
            .ok_or_else(|| JsValue::from_str("unknown phase"))?;

        self.player.pos_x = self.arena.arena.player_spawn.x;
        self.player.pos_y = self.arena.arena.player_spawn.y;
        self.player.hp = PLAYER_MAX_HP;
        self.player.mp = PLAYER_MAX_MP;
        self.player.fire_cooldown = 0;
        self.player.status_mask = 0;
        self.player.statuses = [status::StatusTimer::default(); MAX_STATUS_SLOTS];
        self.player.in_combat_frames = 0;

        self.boss.pos_x = self.arena.arena.boss_spawn.x;
        self.boss.pos_y = self.arena.arena.boss_spawn.y;
        self.boss.hp = self.boss.max_hp * hp_ratio;
        self.boss.status_mask = 0;
        self.boss.statuses = [status::StatusTimer::default(); MAX_STATUS_SLOTS];
        self.boss.phase_index = phase_index;
        self.boss.phase_pattern_counter = 0;
        self.boss.phase_timer = 0;
        self.boss.fire_pattern_index = 0;
        self.boss.ice_pattern_index = 0;
        self.boss.fire_nuke_index = 0;
        self.boss.ice_nuke_index = 0;
        self.boss.neutral_index = 0;
        self.boss.active_pattern = None;
        self.boss.stagger_frames = 0;
        self.boss.support_delay_frames = 0;
        self.boss.damage_window_frames = 0;
        self.boss.invulnerable_override = false;
        self.boss.armored_override = false;
        self.boss.last_pattern_family = PatternFamily::Neutral;
        self.boss.last_pattern_nuke = false;
        self.boss.duel_majority = PatternFamily::Neutral;
        self.boss.duel_stage = 0;
        self.boss.helper_gates_damage = false;
        self.boss.helpers.clear();
        self.boss.objects.clear();
        self.boss.pending_helper_respawns.clear();
        self.boss.enemy_bullets.clear();
        self.boss.player_shots.clear();
        self.shake_amplitude = 0.0;
        self.shake_frames = 0;
        self.current_message.clear();

        self.debug_set_archmage_locks(locks);
        self.apply_phase_enter_commands();
        Ok(())
    }

    fn debug_set_archmage_locks(&mut self, locks: &[GeneratorElement]) {
        for index in 0..self.boss.generators.len() {
            let element = locks.get(index).copied().unwrap_or(GeneratorElement::Fire);
            self.boss.generators.element[index] = element;
            self.boss.generators.sealed[index] = index < locks.len();
            self.boss.generators.vulnerable[index] = false;
            self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
        }
        self.refresh_generator_lock_counts();
    }
}
