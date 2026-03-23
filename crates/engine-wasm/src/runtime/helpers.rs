use schema::{EmitterSource, HelperMotion, ObjectMotion};

use crate::constants::*;
use crate::rng::Rng64;
use crate::runtime::Runtime;

impl Runtime {
    pub fn update_helpers(&mut self, rng: &mut Rng64) {
        for index in 0..self.boss.helpers.len() {
            self.boss.helpers.angle_deg[index] +=
                self.boss.helpers.orbit_speed_deg[index] / 60.0;
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
                    self.boss.helpers.pos_x[index] = self.boss.pos_x
                        + radians.cos() * (self.boss.helpers.orbit_radius[index] + 4.5);
                    self.boss.helpers.pos_y[index] = self.boss.pos_y
                        + radians.sin() * (self.boss.helpers.orbit_radius[index] + 4.5);
                }
                HelperMotion::Hover => {}
            }
            if let Some(pattern_index) = self.boss.helpers.bullet_pattern[index] {
                let helper_frame =
                    (self.frame % self.patterns[pattern_index].duration_frames as u32) as u16;
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
                        let emitter =
                            self.patterns[pattern_index].emitters[emitter_index].clone();
                        self.fire_emitter_from_helper(index, &emitter, helper_frame, rng);
                    }
                }
            }
        }
        let mut index = 0;
        while index < self.boss.helpers.len() {
            if self.boss.helpers.hp[index] <= 0.0 {
                let hx = self.boss.helpers.pos_x[index];
                let hy = self.boss.helpers.pos_y[index];
                let hc = self.boss.helpers.color_rgba[index];
                self.events.extend_from_slice(&[
                    EVENT_HELPER_DEATH, hx, hy, hc[0], hc[1], hc[2], 0.0,
                ]);
                self.boss.helpers.swap_remove(index);
                if self.current_phase().helper_gates_damage && !self.has_phase_blockers() {
                    self.boss.stagger_frames = STAGGER_FRAMES_DEFAULT;
                    self.current_message = "Helper down: stagger window".to_string();
                }
            } else {
                index += 1;
            }
        }
    }

    pub fn update_objects(&mut self, rng: &mut Rng64) {
        for index in 0..self.boss.objects.len() {
            self.boss.objects.angle_deg[index] +=
                self.boss.objects.orbit_speed_deg[index] / 60.0;
            match self.boss.objects.motion[index] {
                ObjectMotion::Fixed => {
                    self.boss.objects.pos_x[index] = self.boss.objects.anchor_x[index];
                    self.boss.objects.pos_y[index] = self.boss.objects.anchor_y[index];
                }
                ObjectMotion::OrbitBoss => {
                    let radians = self.boss.objects.angle_deg[index].to_radians();
                    self.boss.objects.pos_x[index] = self.boss.pos_x
                        + radians.cos() * self.boss.objects.orbit_radius[index];
                    self.boss.objects.pos_y[index] = self.boss.pos_y
                        + radians.sin() * self.boss.objects.orbit_radius[index];
                }
                ObjectMotion::CircleArena => {
                    let radians = self.boss.objects.angle_deg[index].to_radians();
                    self.boss.objects.pos_x[index] = self.boss.objects.anchor_x[index]
                        + radians.cos() * self.boss.objects.orbit_radius[index];
                    self.boss.objects.pos_y[index] = self.boss.objects.anchor_y[index]
                        + radians.sin() * self.boss.objects.orbit_radius[index];
                }
            }
            if let Some(pattern_index) = self.boss.objects.bullet_pattern[index] {
                let object_frame =
                    (self.frame % self.patterns[pattern_index].duration_frames as u32) as u16;
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
                        let emitter =
                            self.patterns[pattern_index].emitters[emitter_index].clone();
                        self.fire_emitter_from_object(index, &emitter, object_frame, rng);
                    }
                }
            }
        }
        let mut index = 0;
        while index < self.boss.objects.len() {
            if self.boss.objects.hp[index] <= 0.0 {
                let ox = self.boss.objects.pos_x[index];
                let oy = self.boss.objects.pos_y[index];
                let oc = self.boss.objects.color_rgba[index];
                self.events.extend_from_slice(&[
                    EVENT_OBJECT_DEATH, ox, oy, oc[0], oc[1], oc[2], 0.0,
                ]);
                self.boss.objects.swap_remove(index);
                if self.current_phase().helper_gates_damage && !self.has_phase_blockers() {
                    self.boss.stagger_frames = STAGGER_FRAMES_DEFAULT;
                    self.current_message = "Barrier down: stagger window".to_string();
                }
            } else {
                index += 1;
            }
        }
    }
}
