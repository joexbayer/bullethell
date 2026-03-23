use schema::{ArenaDef, STATUS_EXPOSED};

use crate::constants::*;
use crate::runtime::status::apply_status;
use crate::runtime::Runtime;

impl Runtime {
    pub fn resolve_collisions(&mut self) {
        self.resolve_enemy_bullets_vs_player();
        self.resolve_player_shots_vs_targets();
    }

    fn resolve_enemy_bullets_vs_player(&mut self) {
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
                    self.boss.enemy_bullets.flags[index] & BULLET_FLAG_ARMOR_PIERCING != 0,
                );
                let bx = self.boss.enemy_bullets.pos_x[index];
                let by = self.boss.enemy_bullets.pos_y[index];
                let bc = self.boss.enemy_bullets.color_rgba[index];
                self.player.hp = (self.player.hp - damage).max(0.0);
                self.player.in_combat_frames = PLAYER_IN_COMBAT_DURATION;
                apply_status(
                    &mut self.player.statuses,
                    &mut self.player.status_mask,
                    self.boss.enemy_bullets.status_mask[index],
                    self.boss.enemy_bullets.status_duration_frames[index],
                );
                self.events.extend_from_slice(&[
                    EVENT_BULLET_HIT_PLAYER, bx, by, bc[0], bc[1], bc[2], damage,
                ]);
                self.boss.enemy_bullets.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn resolve_player_shots_vs_targets(&mut self) {
        let boss_is_damageable = self.boss.damage_window_frames > 0
            || self.boss.stagger_frames > 0
            || (self.boss.support_delay_frames == 0
                && !self.current_phase().invulnerable
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
                    self.boss.generators.hp[generator_index] -=
                        self.boss.player_shots.damage[shot_index];
                    if self.boss.generators.hp[generator_index] <= 0.0 {
                        self.seal_generator(generator_index);
                    }
                    hit = true;
                }
                generator_index += 1;
            }
            let mut helper_index = 0;
            while helper_index < self.boss.helpers.len() {
                if self.helper_is_active(helper_index)
                    && circles_overlap(
                    self.boss.player_shots.pos_x[shot_index],
                    self.boss.player_shots.pos_y[shot_index],
                    self.boss.player_shots.radius[shot_index],
                    self.boss.helpers.pos_x[helper_index],
                    self.boss.helpers.pos_y[helper_index],
                    self.boss.helpers.radius[helper_index],
                ) {
                    if !self.boss.helpers.invulnerable[helper_index] {
                        let mut damage = self.boss.player_shots.damage[shot_index];
                        if self.boss.helpers.armored[helper_index] {
                            damage *= ARMOR_REDUCTION;
                        }
                        if self.boss.helpers.exposed[helper_index] {
                            damage += EXPOSED_BONUS_DAMAGE;
                        }
                        self.boss.helpers.hp[helper_index] -= damage;
                    }
                    hit = true;
                    break;
                }
                helper_index += 1;
            }
            let mut object_index = 0;
            while !hit && object_index < self.boss.objects.len() {
                if self.object_is_active(object_index)
                    && circles_overlap(
                    self.boss.player_shots.pos_x[shot_index],
                    self.boss.player_shots.pos_y[shot_index],
                    self.boss.player_shots.radius[shot_index],
                    self.boss.objects.pos_x[object_index],
                    self.boss.objects.pos_y[object_index],
                    self.boss.objects.radius[object_index],
                ) {
                    self.boss.objects.hp[object_index] -=
                        self.boss.player_shots.damage[shot_index];
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
                    damage *= ARMOR_REDUCTION;
                }
                if exposed {
                    damage += EXPOSED_BONUS_DAMAGE;
                }
                self.boss.hp = (self.boss.hp - damage).max(0.0);
                if let Some(active_pattern) = self.boss.active_pattern.as_mut() {
                    active_pattern.damage_taken += damage;
                }
                self.events.extend_from_slice(&[
                    EVENT_SHOT_HIT_ENEMY,
                    self.boss.pos_x,
                    self.boss.pos_y,
                    1.0, 1.0, 0.8,
                    damage,
                ]);
                hit = true;
            }
            if hit {
                self.boss.player_shots.swap_remove(shot_index);
            } else {
                shot_index += 1;
            }
        }
    }
}

pub fn circles_overlap(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    let radius = ar + br;
    dx * dx + dy * dy <= radius * radius
}

pub fn resolve_actor_vs_tiles(arena: &ArenaDef, pos_x: &mut f32, pos_y: &mut f32, radius: f32) {
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

pub fn bullet_hits_wall(x: f32, y: f32, radius: f32, arena: &ArenaDef) -> bool {
    let tile_size = arena.tile_size;
    let min_x = ((x - radius) / tile_size).floor().max(0.0) as i32;
    let max_x = ((x + radius) / tile_size)
        .floor()
        .min(arena.width as f32 - 1.0) as i32;
    let min_y = ((y - radius) / tile_size).floor().max(0.0) as i32;
    let max_y = ((y + radius) / tile_size)
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

pub fn apply_defense(damage: f32, defense: f32, armor_piercing: bool) -> f32 {
    if armor_piercing {
        return damage;
    }
    (damage - defense).max(damage * 0.1)
}

fn tile_bit_is_set(words: &[u64], index: usize) -> bool {
    words
        .get(index / 64)
        .map(|word| (word >> (index % 64)) & 1 == 1)
        .unwrap_or(false)
}
