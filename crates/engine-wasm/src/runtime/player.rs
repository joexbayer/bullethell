use schema::{STATUS_EXPOSED, STATUS_SILENCED, STATUS_SLOW};

use crate::constants::*;
use crate::pool::bullet::SpawnedBullet;
use crate::runtime::collision::resolve_actor_vs_tiles;
use crate::runtime::status::{apply_status, StatusTimer};
use crate::runtime::Runtime;
use crate::types::InputSnapshot;

#[derive(Clone)]
pub struct PlayerState {
    pub pos_x: f32,
    pub pos_y: f32,
    pub hp: f32,
    pub mp: f32,
    pub fire_cooldown: u16,
    pub status_mask: u32,
    pub statuses: [StatusTimer; MAX_STATUS_SLOTS],
    pub in_combat_frames: u16,
}

impl Runtime {
    pub fn update_player(&mut self, input: InputSnapshot) {
        let mut move_x = input.move_x;
        let mut move_y = input.move_y;
        let length = (move_x * move_x + move_y * move_y).sqrt();
        if length > 1.0 {
            move_x /= length;
            move_y /= length;
        }
        let mut speed = PLAYER_SPEED;
        if self.player.status_mask & STATUS_SLOW != 0 {
            speed = PLAYER_SPEED_SLOWED;
        }
        if self.player.status_mask & STATUS_SILENCED == 0
            && input.ability_pressed
            && self.player.mp >= PLAYER_ABILITY_COST
        {
            self.player.mp -= PLAYER_ABILITY_COST;
            apply_status(
                &mut self.player.statuses,
                &mut self.player.status_mask,
                STATUS_SLOW,
                0,
            );
            apply_status(
                &mut self.player.statuses,
                &mut self.player.status_mask,
                STATUS_EXPOSED,
                PLAYER_ABILITY_EXPOSED_FRAMES,
            );
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
                    sprite: SPRITE_PLAYER_SHOT,
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
                    flags: BULLET_FLAG_IS_PLAYER_SHOT,
                    delay_frames: 0,
                    damage: PLAYER_SHOT_DAMAGE,
                    render_layer: PLAYER_SHOT_LAYER,
                });
                self.player.fire_cooldown = PLAYER_FIRE_COOLDOWN_FRAMES;
            }
        }
        if self.player.fire_cooldown > 0 {
            self.player.fire_cooldown -= 1;
        }
    }

    pub fn tick_statuses(&mut self) {
        super::status::tick_status_array(&mut self.player.statuses, &mut self.player.status_mask);
        super::status::tick_status_array(&mut self.boss.statuses, &mut self.boss.status_mask);
        if self.player.in_combat_frames > 0 {
            self.player.in_combat_frames -= 1;
        }
        let vit_scale = if self.player.in_combat_frames > 0 {
            REGEN_COMBAT_SCALE
        } else {
            1.0
        };
        let can_heal = self.player.status_mask & schema::STATUS_SICK == 0;
        if can_heal {
            self.player.hp =
                (self.player.hp + PLAYER_VIT_REGEN * vit_scale / 60.0).min(PLAYER_MAX_HP);
        }
        if self.player.status_mask & STATUS_SILENCED == 0 {
            self.player.mp =
                (self.player.mp + PLAYER_WIS_REGEN * vit_scale / 60.0).min(PLAYER_MAX_MP);
        }
    }
}
