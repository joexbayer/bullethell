use schema::{AngleMode, BulletBehavior, EmitterDef, EmitterSource};

use crate::constants::*;
use crate::pool::bullet::SpawnedBullet;
use crate::rng::Rng64;
use crate::runtime::status::projectile_color;
use crate::runtime::Runtime;

impl Runtime {
    pub fn fire_emitter(&mut self, emitter: &EmitterDef, frame: u16, rng: &mut Rng64) {
        let origin = match emitter.source {
            EmitterSource::Boss => (self.boss.pos_x, self.boss.pos_y),
            EmitterSource::ArenaTop => {
                (self.player.pos_x, self.arena.arena.camera_bounds.min_y + 1.0)
            }
            EmitterSource::ArenaBottom => {
                (self.player.pos_x, self.arena.arena.camera_bounds.max_y - 1.0)
            }
            EmitterSource::ArenaLeft => {
                (self.arena.arena.camera_bounds.min_x + 1.0, self.player.pos_y)
            }
            EmitterSource::ArenaRight => {
                (self.arena.arena.camera_bounds.max_x - 1.0, self.player.pos_y)
            }
            EmitterSource::Helper | EmitterSource::Object => return,
        };
        self.spawn_emitter_burst(origin.0, origin.1, emitter, frame, rng);
    }

    pub fn fire_emitter_from_helper(
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

    pub fn fire_emitter_from_object(
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

    pub fn spawn_emitter_burst(
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
                    flags |= BULLET_FLAG_ORBIT;
                }
                BulletBehavior::Boomerang => flags |= BULLET_FLAG_BOOMERANG,
                BulletBehavior::AccelerateAfterDelay | BulletBehavior::Default => {}
            }
            if archetype.armor_piercing {
                flags |= BULLET_FLAG_ARMOR_PIERCING;
            }
            if archetype.die_on_wall {
                flags |= BULLET_FLAG_DIE_ON_WALL;
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
                detonate_frames: archetype
                    .detonation
                    .as_ref()
                    .map(|detonation| detonation.after_frames)
                    .unwrap_or(0),
                damage: archetype.damage,
                render_layer: archetype.render_layer,
            });
        }
    }
}

pub fn compute_angle_deg(
    emitter: &EmitterDef,
    shot_index: usize,
    burst_count: usize,
    spread_total: f32,
    frame: u16,
    player_dx: f32,
    player_dy: f32,
) -> f32 {
    let center_angle = match emitter.angle_mode {
        AngleMode::AimAtPlayer => {
            player_dy.atan2(player_dx).to_degrees() + emitter.base_angle_deg
        }
        AngleMode::Fixed => emitter.base_angle_deg,
        AngleMode::Spin => emitter.base_angle_deg + frame as f32 * emitter.spin_speed_deg / 60.0,
        AngleMode::Radial => {
            emitter.base_angle_deg + frame as f32 * emitter.spin_speed_deg / 120.0
        }
    };
    if burst_count == 1 {
        center_angle
    } else {
        let offset = shot_index as f32 / (burst_count - 1) as f32 - 0.5;
        center_angle + offset * spread_total
    }
}
