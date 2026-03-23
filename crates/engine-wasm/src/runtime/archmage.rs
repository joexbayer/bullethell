use schema::{HelperMotion, PatternFamily};

use crate::constants::*;
use crate::pool::helper::HelperSpawn;
use crate::runtime::Runtime;

const SUPPORT_GRACE_FRAMES: u16 = 180;
const SUPPORT_DAMAGE_WINDOW_FRAMES: u16 = 1200;
const SUPPORT_RESPAWN_HP_RATIO: f32 = 0.65;
const DUAL_GUARD_HELPER_HP: f32 = 240.0;

impl Runtime {
    pub fn begin_support_damage_window(&mut self) {
        if !self.is_twilight_archmage_regular() {
            return;
        }
        if !matches!(self.current_phase().id.as_str(), "single_bird" | "dual_guard") {
            return;
        }
        self.boss.damage_window_frames = SUPPORT_DAMAGE_WINDOW_FRAMES;
        self.boss.stagger_frames = SUPPORT_DAMAGE_WINDOW_FRAMES;
        let pending = std::mem::take(&mut self.boss.pending_helper_respawns);
        for mut helper in pending {
            helper.invulnerable = true;
            helper.armored = false;
            helper.exposed = false;
            self.upsert_helper(helper, false);
        }
        self.current_message = "Archmage staggered".to_string();
    }

    pub fn trigger_support_window(&mut self) {
        if !self.is_twilight_archmage_regular() {
            self.boss.stagger_frames = crate::constants::STAGGER_FRAMES_DEFAULT;
            return;
        }
        if !matches!(self.current_phase().id.as_str(), "single_bird" | "dual_guard") {
            self.boss.stagger_frames = crate::constants::STAGGER_FRAMES_DEFAULT;
            return;
        }
        self.boss.active_pattern = None;
        self.boss.support_delay_frames = SUPPORT_GRACE_FRAMES;
        self.boss.damage_window_frames = 0;
        self.boss.stagger_frames = 0;
        self.current_message = "Bird down: clear bullets".to_string();
    }

    pub fn archmage_on_phase_enter(&mut self) {
        if !self.is_twilight_archmage_regular() {
            return;
        }
        match self.current_phase().id.as_str() {
            "seal_two" | "seal_three" => {
                if self.current_phase().id == "seal_two" {
                    self.configure_seal_two_support();
                } else {
                    self.configure_seal_three_support();
                }
            }
            "duel" => {
                self.refresh_generator_lock_counts();
                self.boss.hp = self.boss.max_hp * 0.20;
                self.boss.duel_majority = self.archmage_locked_majority();
                self.boss.duel_stage = 0;
                self.boss.pending_helper_respawns.clear();
                self.setup_archmage_duel_stage();
            }
            "finale" => {
                self.refresh_generator_lock_counts();
                self.boss.support_delay_frames = 0;
                self.boss.damage_window_frames = 0;
                self.boss.stagger_frames = 0;
                self.boss.pending_helper_respawns.clear();
                self.configure_finale_support();
            }
            "single_bird" => {
                self.current_message = "Kill the bird to stagger the Archmage".to_string();
            }
            "dual_guard" => {
                self.current_message = "Kill both birds to stagger the Archmage".to_string();
            }
            _ => {}
        }
    }

    pub fn archmage_on_pattern_selected(&mut self) {
        if !self.is_twilight_archmage_regular() {
            return;
        }
        match self.current_phase().id.as_str() {
            "single_bird" => self.configure_single_bird_pattern(),
            "dual_guard" => self.configure_dual_guard_pattern(),
            "seal_two" | "seal_three" => {
                for index in 0..self.boss.helpers.len() {
                    self.boss.helpers.invulnerable[index] = true;
                    self.boss.helpers.armored[index] = false;
                    self.boss.helpers.exposed[index] = false;
                }
            }
            _ => {}
        }
    }

    pub fn archmage_handle_helper_death(&mut self, helper_id: &str) -> bool {
        if !self.is_twilight_archmage_regular() || self.current_phase().id != "duel" {
            return false;
        }
        match (self.boss.duel_majority, self.boss.duel_stage, helper_id) {
            (PatternFamily::Fire, 0, "blizzard") => {
                self.boss.duel_stage = 1;
                self.setup_archmage_duel_stage();
                true
            }
            (PatternFamily::Ice, 0, "inferno") => {
                self.boss.duel_stage = 1;
                self.setup_archmage_duel_stage();
                true
            }
            (PatternFamily::Fire | PatternFamily::Ice, 1, _) => {
                self.enter_archmage_finale();
                true
            }
            _ => false,
        }
    }

    pub fn archmage_locked_majority(&self) -> PatternFamily {
        if self.boss.fire_locks > self.boss.ice_locks {
            PatternFamily::Fire
        } else if self.boss.ice_locks > self.boss.fire_locks {
            PatternFamily::Ice
        } else {
            PatternFamily::Neutral
        }
    }

    fn configure_single_bird_pattern(&mut self) {
        match self.boss.last_pattern_family {
            PatternFamily::Fire => {
                self.upsert_helper(self.inferno_spawn(false, self.boss.last_pattern_nuke, false), true);
                let mut blizzard = self.blizzard_spawn(true, false, false);
                blizzard.bullet_pattern = None;
                self.upsert_helper(blizzard, true);
                self.current_message = if self.boss.last_pattern_nuke {
                    "Inferno defends the Archmage (Armored)".to_string()
                } else {
                    "Inferno defends the Archmage".to_string()
                };
            }
            PatternFamily::Ice => {
                self.upsert_helper(self.blizzard_spawn(false, self.boss.last_pattern_nuke, false), true);
                let mut inferno = self.inferno_spawn(true, false, false);
                inferno.bullet_pattern = None;
                self.upsert_helper(inferno, true);
                self.current_message = if self.boss.last_pattern_nuke {
                    "Blizzard defends the Archmage (Armored)".to_string()
                } else {
                    "Blizzard defends the Archmage".to_string()
                };
            }
            PatternFamily::Neutral => {}
        }
    }

    fn configure_dual_guard_pattern(&mut self) {
        let (inferno_armored, inferno_exposed, blizzard_armored, blizzard_exposed) =
            match (self.boss.last_pattern_family, self.boss.last_pattern_nuke) {
                (PatternFamily::Fire, true) => (true, false, false, true),
                (PatternFamily::Fire, false) => (false, false, false, true),
                (PatternFamily::Ice, true) => (false, true, true, false),
                (PatternFamily::Ice, false) => (false, true, false, false),
                _ => (false, false, false, false),
            };
        let mut inferno = self.inferno_spawn(false, inferno_armored, inferno_exposed);
        inferno.hp = DUAL_GUARD_HELPER_HP;
        inferno.max_hp = DUAL_GUARD_HELPER_HP;
        let mut blizzard = self.blizzard_spawn(false, blizzard_armored, blizzard_exposed);
        blizzard.hp = DUAL_GUARD_HELPER_HP;
        blizzard.max_hp = DUAL_GUARD_HELPER_HP;
        self.upsert_helper(inferno, true);
        self.upsert_helper(blizzard, true);
    }

    fn configure_seal_two_support(&mut self) {
        match self.boss.last_pattern_family {
            PatternFamily::Fire => {
                let mut inferno =
                    self.inferno_spawn(true, self.boss.last_pattern_nuke, false);
                inferno.bullet_pattern =
                    self.pattern_lookup.get("inferno_seal_support_pattern").copied();
                let mut blizzard = self.blizzard_spawn(true, false, false);
                blizzard.bullet_pattern = None;
                self.upsert_helper(inferno, true);
                self.upsert_helper(blizzard, true);
            }
            PatternFamily::Ice => {
                let mut blizzard =
                    self.blizzard_spawn(true, self.boss.last_pattern_nuke, false);
                blizzard.bullet_pattern =
                    self.pattern_lookup.get("blizzard_seal_support_pattern").copied();
                let mut inferno = self.inferno_spawn(true, false, false);
                inferno.bullet_pattern = None;
                self.upsert_helper(blizzard, true);
                self.upsert_helper(inferno, true);
            }
            PatternFamily::Neutral => {
                let mut inferno = self.inferno_spawn(true, false, false);
                let mut blizzard = self.blizzard_spawn(true, false, false);
                inferno.bullet_pattern = None;
                blizzard.bullet_pattern = None;
                self.upsert_helper(inferno, true);
                self.upsert_helper(blizzard, true);
            }
        }
    }

    fn configure_seal_three_support(&mut self) {
        self.upsert_helper(self.inferno_spawn(true, false, false), true);
        self.upsert_helper(self.blizzard_spawn(true, false, false), true);
    }

    fn configure_finale_support(&mut self) {
        let mut inferno = self.inferno_spawn(true, false, false);
        let mut blizzard = self.blizzard_spawn(true, false, false);
        inferno.bullet_pattern = None;
        blizzard.bullet_pattern = None;
        self.upsert_helper(inferno, true);
        self.upsert_helper(blizzard, true);
    }

    fn setup_archmage_duel_stage(&mut self) {
        self.boss.helpers.clear();
        match (self.boss.duel_majority, self.boss.duel_stage) {
            (PatternFamily::Fire, 0) => {
                self.upsert_helper(self.inferno_duel_guard_spawn(true), false);
                self.upsert_helper(self.blizzard_duel_hunter_spawn(false, false), false);
                self.current_message = "Fire-locked duel: Blizzard first".to_string();
            }
            (PatternFamily::Fire, 1) => {
                self.upsert_helper(self.inferno_duel_guard_spawn(false), false);
                self.current_message = "Inferno enrages".to_string();
            }
            (PatternFamily::Ice, 0) => {
                self.upsert_helper(self.blizzard_duel_guard_spawn(true), false);
                self.upsert_helper(self.inferno_duel_hunter_spawn(false, false), false);
                self.current_message = "Ice-locked duel: Inferno first".to_string();
            }
            (PatternFamily::Ice, 1) => {
                self.upsert_helper(self.blizzard_duel_guard_spawn(false), false);
                self.current_message = "Blizzard enrages".to_string();
            }
            _ => {}
        }
    }

    fn enter_archmage_finale(&mut self) {
        let Some(finale_index) = self.phase_lookup.get("finale").copied() else {
            return;
        };
        self.boss.phase_index = finale_index;
        self.boss.phase_pattern_counter = 0;
        self.boss.phase_timer = 0;
        self.boss.active_pattern = None;
        self.boss.support_delay_frames = 0;
        self.boss.damage_window_frames = 0;
        self.boss.stagger_frames = 0;
        self.apply_phase_enter_commands();
    }

    fn inferno_spawn(&self, invulnerable: bool, armored: bool, exposed: bool) -> HelperSpawn {
        HelperSpawn {
            ids: "inferno".to_string(),
            sprite: 10,
            pos_x: self.boss.pos_x + 5.0,
            pos_y: self.boss.pos_y,
            hp: 440.0,
            max_hp: 440.0,
            radius: 0.44,
            motion: HelperMotion::OrbitBoss,
            orbit_radius: 5.0,
            orbit_speed_deg: 58.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("inferno_guard_pattern").copied(),
            color_rgba: [1.0, 0.56, 0.18, 1.0],
            invulnerable,
            armored,
            exposed,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    fn blizzard_spawn(&self, invulnerable: bool, armored: bool, exposed: bool) -> HelperSpawn {
        HelperSpawn {
            ids: "blizzard".to_string(),
            sprite: 11,
            pos_x: self.boss.pos_x + 6.2,
            pos_y: self.boss.pos_y,
            hp: 440.0,
            max_hp: 440.0,
            radius: 0.44,
            motion: HelperMotion::CircleArena,
            orbit_radius: 1.8,
            orbit_speed_deg: -50.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("blizzard_guard_pattern").copied(),
            color_rgba: [0.72, 0.92, 1.0, 1.0],
            invulnerable,
            armored,
            exposed,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    fn inferno_duel_guard_spawn(&self, invulnerable: bool) -> HelperSpawn {
        HelperSpawn {
            ids: "inferno".to_string(),
            sprite: 10,
            pos_x: self.boss.pos_x + 3.8,
            pos_y: self.boss.pos_y,
            hp: 380.0,
            max_hp: 380.0,
            radius: 0.42,
            motion: HelperMotion::OrbitBoss,
            orbit_radius: 3.8,
            orbit_speed_deg: 74.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("inferno_duel_pattern").copied(),
            color_rgba: [1.0, 0.56, 0.18, 1.0],
            invulnerable,
            armored: true,
            exposed: false,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    fn blizzard_duel_guard_spawn(&self, invulnerable: bool) -> HelperSpawn {
        HelperSpawn {
            ids: "blizzard".to_string(),
            sprite: 11,
            pos_x: self.boss.pos_x + 2.2,
            pos_y: self.boss.pos_y,
            hp: 380.0,
            max_hp: 380.0,
            radius: 0.42,
            motion: HelperMotion::CircleArena,
            orbit_radius: 2.2,
            orbit_speed_deg: -68.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("blizzard_duel_pattern").copied(),
            color_rgba: [0.72, 0.92, 1.0, 1.0],
            invulnerable,
            armored: true,
            exposed: false,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    fn inferno_duel_hunter_spawn(
        &self,
        invulnerable: bool,
        armored: bool,
    ) -> HelperSpawn {
        HelperSpawn {
            ids: "inferno".to_string(),
            sprite: 10,
            pos_x: self.boss.pos_x + 5.8,
            pos_y: self.boss.pos_y,
            hp: 380.0,
            max_hp: 380.0,
            radius: 0.42,
            motion: HelperMotion::OrbitBoss,
            orbit_radius: 5.8,
            orbit_speed_deg: 40.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("inferno_duel_finish_pattern").copied(),
            color_rgba: [1.0, 0.56, 0.18, 1.0],
            invulnerable,
            armored,
            exposed: false,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    fn blizzard_duel_hunter_spawn(
        &self,
        invulnerable: bool,
        armored: bool,
    ) -> HelperSpawn {
        HelperSpawn {
            ids: "blizzard".to_string(),
            sprite: 11,
            pos_x: self.boss.pos_x + 5.6,
            pos_y: self.boss.pos_y,
            hp: 380.0,
            max_hp: 380.0,
            radius: 0.42,
            motion: HelperMotion::CircleArena,
            orbit_radius: 2.8,
            orbit_speed_deg: -88.0,
            angle_deg: 0.0,
            bullet_pattern: self.pattern_lookup.get("blizzard_duel_finish_pattern").copied(),
            color_rgba: [0.72, 0.92, 1.0, 1.0],
            invulnerable,
            armored,
            exposed: false,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    pub fn helper_spawn_from_index(&self, index: usize) -> HelperSpawn {
        let respawn_hp = if self.is_twilight_archmage_regular() {
            self.boss.helpers.max_hp[index] * SUPPORT_RESPAWN_HP_RATIO
        } else {
            self.boss.helpers.max_hp[index]
        };
        HelperSpawn {
            ids: self.boss.helpers.ids[index].clone(),
            sprite: self.boss.helpers.sprite[index],
            pos_x: self.boss.helpers.pos_x[index],
            pos_y: self.boss.helpers.pos_y[index],
            hp: respawn_hp,
            max_hp: self.boss.helpers.max_hp[index],
            radius: self.boss.helpers.radius[index],
            motion: self.boss.helpers.motion[index],
            orbit_radius: self.boss.helpers.orbit_radius[index],
            orbit_speed_deg: self.boss.helpers.orbit_speed_deg[index],
            angle_deg: self.boss.helpers.angle_deg[index],
            bullet_pattern: self.boss.helpers.bullet_pattern[index],
            color_rgba: self.boss.helpers.color_rgba[index],
            invulnerable: true,
            armored: false,
            exposed: false,
            transition_frames: HELPER_SPAWN_FRAMES,
            transition_state: ENTITY_STATE_SPAWNING,
        }
    }

    pub fn upsert_helper(&mut self, spawn: HelperSpawn, preserve_hp: bool) {
        if let Some(index) = self.boss.helpers.find_index(&spawn.ids) {
            let hp = if preserve_hp {
                self.boss.helpers.hp[index]
            } else {
                spawn.hp
            };
            self.boss.helpers.sprite[index] = spawn.sprite;
            if !preserve_hp {
                self.boss.helpers.pos_x[index] = spawn.pos_x;
                self.boss.helpers.pos_y[index] = spawn.pos_y;
            }
            self.boss.helpers.hp[index] = hp.min(spawn.max_hp);
            self.boss.helpers.max_hp[index] = spawn.max_hp;
            self.boss.helpers.radius[index] = spawn.radius;
            self.boss.helpers.motion[index] = spawn.motion;
            self.boss.helpers.orbit_radius[index] = spawn.orbit_radius;
            self.boss.helpers.orbit_speed_deg[index] = spawn.orbit_speed_deg;
            if !preserve_hp {
                self.boss.helpers.angle_deg[index] = spawn.angle_deg;
            }
            self.boss.helpers.bullet_pattern[index] = spawn.bullet_pattern;
            self.boss.helpers.color_rgba[index] = spawn.color_rgba;
            self.boss.helpers.invulnerable[index] = spawn.invulnerable;
            self.boss.helpers.armored[index] = spawn.armored;
            self.boss.helpers.exposed[index] = spawn.exposed;
            if preserve_hp {
                self.boss.helpers.transition_frames[index] = 0;
                self.boss.helpers.transition_state[index] = ENTITY_STATE_ACTIVE;
            } else {
                self.boss.helpers.transition_frames[index] = spawn.transition_frames;
                self.boss.helpers.transition_state[index] = spawn.transition_state;
            }
        } else {
            self.boss.helpers.push(spawn);
        }
    }

    fn is_twilight_archmage_regular(&self) -> bool {
        self.encounter_id == "twilight_archmage_v1"
    }
}
