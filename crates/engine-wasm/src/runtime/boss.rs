use schema::{CommandDef, PatternFamily};

use crate::constants::*;
use crate::pool::bullet::BulletPool;
use crate::pool::generator::GeneratorPool;
use crate::pool::helper::{HelperPool, HelperSpawn};
use crate::pool::object::{EncounterObjectPool, EncounterObjectSpawn};
use crate::rng::Rng64;
use crate::runtime::status::StatusTimer;
use crate::runtime::Runtime;

#[derive(Clone)]
pub struct BossRuntime {
    pub pos_x: f32,
    pub pos_y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub radius: f32,
    pub status_mask: u32,
    pub statuses: [StatusTimer; MAX_STATUS_SLOTS],
    pub phase_index: usize,
    pub phase_pattern_counter: u32,
    pub phase_timer: u32,
    pub fire_pattern_index: usize,
    pub ice_pattern_index: usize,
    pub fire_nuke_index: usize,
    pub ice_nuke_index: usize,
    pub neutral_index: usize,
    pub active_pattern: Option<ActivePattern>,
    pub stagger_frames: u16,
    pub support_delay_frames: u16,
    pub damage_window_frames: u16,
    pub invulnerable_override: bool,
    pub armored_override: bool,
    pub fire_locks: u8,
    pub ice_locks: u8,
    pub last_pattern_family: PatternFamily,
    pub last_pattern_nuke: bool,
    pub duel_majority: PatternFamily,
    pub duel_stage: u8,
    pub helper_gates_damage: bool,
    pub generators: GeneratorPool,
    pub helpers: HelperPool,
    pub objects: EncounterObjectPool,
    pub pending_helper_respawns: Vec<HelperSpawn>,
    pub enemy_bullets: BulletPool,
    pub player_shots: BulletPool,
}

#[derive(Clone)]
pub struct ActivePattern {
    pub pattern_index: usize,
    pub frame: u16,
    pub damage_taken: f32,
}

impl Runtime {
    pub fn update_pattern(&mut self, rng: &mut Rng64) {
        if self.boss.support_delay_frames > 0 || self.boss.stagger_frames > 0 {
            return;
        }
        if self.boss.active_pattern.is_none() {
            let pattern_index = self.select_pattern_index(rng);
            self.boss.active_pattern = Some(ActivePattern {
                pattern_index,
                frame: 0,
                damage_taken: 0.0,
            });
            self.boss.phase_pattern_counter += 1;
            self.archmage_on_pattern_selected();
        }
        let Some(mut active) = self.boss.active_pattern.take() else {
            return;
        };
        let pattern_index = active.pattern_index;
        let mut pending_commands = Vec::new();
        {
            let pattern = &self.patterns[pattern_index];
            for command in pattern
                .commands
                .iter()
                .filter(|command| command.frame == active.frame)
            {
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

    pub fn select_pattern_index(&mut self, rng: &mut Rng64) -> usize {
        if self.encounter_id == "twilight_archmage_v1" {
            match self.current_phase().id.as_str() {
                "duel" => {
                    let pattern_id = match self.boss.duel_majority {
                        PatternFamily::Fire => "fire_axis_sweep",
                        PatternFamily::Ice => "ice_spinner_trap",
                        PatternFamily::Neutral => "fire_axis_sweep",
                    };
                    self.boss.last_pattern_family = self.boss.duel_majority;
                    self.boss.last_pattern_nuke = true;
                    return self.pattern_lookup[pattern_id];
                }
                "finale" => {
                    let pattern_id = match self.archmage_locked_majority() {
                        PatternFamily::Fire => "finale_fire",
                        PatternFamily::Ice => "finale_ice",
                        PatternFamily::Neutral => "finale_fire",
                    };
                    self.boss.last_pattern_family = self.archmage_locked_majority();
                    self.boss.last_pattern_nuke = true;
                    return self.pattern_lookup[pattern_id];
                }
                _ => {}
            }
        }
        let selector = self.current_phase().selector.clone();
        let (family, nuke) = self.select_generator_family(rng);
        self.boss.last_pattern_family = family;
        self.boss.last_pattern_nuke = nuke;
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

    pub fn execute_command(&mut self, command: CommandDef) {
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
                let bullet_pattern =
                    bullet_pattern.and_then(|id| self.pattern_lookup.get(&id).copied());
                self.boss.helpers.remove_id(&helper_id);
                let spawn = HelperSpawn {
                    ids: helper_id,
                    sprite,
                    pos_x: self.boss.pos_x + orbit_radius,
                    pos_y: self.boss.pos_y,
                    hp,
                    max_hp: hp,
                    radius,
                    motion,
                    orbit_radius,
                    orbit_speed_deg,
                    angle_deg: 0.0,
                    bullet_pattern,
                    color_rgba,
                    invulnerable: false,
                    armored: false,
                    exposed: false,
                    transition_frames: HELPER_SPAWN_FRAMES,
                    transition_state: ENTITY_STATE_SPAWNING,
                };
                self.upsert_helper(spawn, false);
            }
            CommandDef::DespawnHelper { helper_id } => {
                self.despawn_helper_id(&helper_id);
            }
            CommandDef::DespawnHelpers => self.despawn_all_helpers(),
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
                let bullet_pattern =
                    bullet_pattern.and_then(|id| self.pattern_lookup.get(&id).copied());
                self.boss.objects.remove_id(&object_id);
                let spawn = EncounterObjectSpawn {
                    ids: object_id,
                    sprite,
                    pos_x: anchor.x,
                    pos_y: anchor.y,
                    hp,
                    max_hp: hp,
                    radius,
                    motion,
                    anchor_x: anchor.x,
                    anchor_y: anchor.y,
                    orbit_radius,
                    orbit_speed_deg,
                    angle_deg: 0.0,
                    bullet_pattern,
                    color_rgba,
                    transition_frames: OBJECT_SPAWN_FRAMES,
                    transition_state: ENTITY_STATE_SPAWNING,
                };
                if let Some(index) = self.boss.objects.find_index(&spawn.ids) {
                    self.boss.objects.sprite[index] = spawn.sprite;
                    self.boss.objects.pos_x[index] = spawn.pos_x;
                    self.boss.objects.pos_y[index] = spawn.pos_y;
                    self.boss.objects.hp[index] = spawn.hp;
                    self.boss.objects.max_hp[index] = spawn.max_hp;
                    self.boss.objects.radius[index] = spawn.radius;
                    self.boss.objects.motion[index] = spawn.motion;
                    self.boss.objects.anchor_x[index] = spawn.anchor_x;
                    self.boss.objects.anchor_y[index] = spawn.anchor_y;
                    self.boss.objects.orbit_radius[index] = spawn.orbit_radius;
                    self.boss.objects.orbit_speed_deg[index] = spawn.orbit_speed_deg;
                    self.boss.objects.angle_deg[index] = spawn.angle_deg;
                    self.boss.objects.bullet_pattern[index] = spawn.bullet_pattern;
                    self.boss.objects.color_rgba[index] = spawn.color_rgba;
                    self.boss.objects.transition_frames[index] = OBJECT_SPAWN_FRAMES;
                    self.boss.objects.transition_state[index] = ENTITY_STATE_SPAWNING;
                } else {
                    self.boss.objects.push(spawn);
                }
            }
            CommandDef::DespawnObject { object_id } => {
                self.despawn_object_id(&object_id);
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
            CommandDef::DespawnObjects => self.despawn_all_objects(),
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
            CommandDef::SetArenaShake {
                amplitude,
                frames,
            } => {
                self.shake_amplitude = amplitude;
                self.shake_frames = frames;
            }
            CommandDef::ClearTilesRect {
                col,
                row,
                width,
                height,
            } => {
                let arena_width = self.arena.arena.width;
                for r in row..row + height {
                    for c in col..col + width {
                        let index = (r * arena_width + c) as usize;
                        if let Some(tile) = self.arena.arena.tiles.get_mut(index) {
                            *tile = 0;
                        }
                        // Clear the collision bitset so the hitbox is removed
                        if let Some(word) = self.arena.arena.collision_words.get_mut(index / 64) {
                            *word &= !(1_u64 << (index % 64));
                        }
                    }
                }
            }
        }
    }

    pub fn apply_phase_enter_commands(&mut self) {
        self.current_message = format!("Phase: {}", self.current_phase().id);
        self.boss.invulnerable_override = self.current_phase().invulnerable;
        self.boss.armored_override = self.current_phase().armored;
        self.boss.helper_gates_damage = self.current_phase().helper_gates_damage;
        self.boss.support_delay_frames = 0;
        self.boss.damage_window_frames = 0;
        self.boss.stagger_frames = 0;
        self.boss.pending_helper_respawns.clear();
        let commands = self.current_phase().enter_commands.clone();
        for command in commands {
            self.execute_command(command);
        }
        self.archmage_on_phase_enter();
    }

    pub fn apply_transitions(&mut self) {
        if self.boss.hp <= 0.0 {
            self.events.extend_from_slice(&[
                EVENT_BOSS_DEATH, self.boss.pos_x, self.boss.pos_y,
                0.42, 0.24, 0.60, 0.0,
            ]);
            self.current_message = "Boss defeated".to_string();
            self.boss.active_pattern = None;
            self.boss.support_delay_frames = 0;
            self.boss.damage_window_frames = 0;
            self.boss.stagger_frames = 0;
            self.boss.pending_helper_respawns.clear();
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
                schema::TransitionConditionDef::TimerAtLeast(frames) => {
                    self.boss.phase_timer >= frames
                }
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

    pub fn has_phase_blockers(&self) -> bool {
        self.boss
            .helpers
            .transition_state
            .iter()
            .zip(self.boss.helpers.invulnerable.iter())
            .any(|(state, invulnerable)| *state == ENTITY_STATE_ACTIVE && !*invulnerable)
            || self
                .boss
                .objects
                .transition_state
                .iter()
                .any(|state| *state == ENTITY_STATE_ACTIVE)
    }
}
