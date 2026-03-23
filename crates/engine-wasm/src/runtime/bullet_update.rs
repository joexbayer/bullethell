use crate::constants::BULLET_FLAG_DIE_ON_WALL;
use crate::pool::bullet::BulletPool;
use crate::runtime::emitter::compute_angle_deg;
use crate::runtime::collision::bullet_hits_wall;
use crate::runtime::Runtime;

impl Runtime {
    pub fn update_bullets(&mut self) {
        self.update_enemy_bullets();
        update_bullet_pool(&mut self.boss.player_shots, &self.arena.arena);
    }

    fn update_enemy_bullets(&mut self) {
        let len = self.boss.enemy_bullets.len();

        for i in 0..len {
            if self.boss.enemy_bullets.ttl_frames[i] > 0 {
                self.boss.enemy_bullets.ttl_frames[i] -= 1;
            }
        }

        for i in 0..len {
            if self.boss.enemy_bullets.delay_frames[i] > 0 {
                self.boss.enemy_bullets.delay_frames[i] -= 1;
            }
            if self.boss.enemy_bullets.detonate_frames[i] > 0 {
                self.boss.enemy_bullets.detonate_frames[i] -= 1;
            }
        }

        for i in 0..len {
            if self.boss.enemy_bullets.delay_frames[i] == 0
                && self.boss.enemy_bullets.angular_vel_deg[i] != 0.0
            {
                self.boss.enemy_bullets.angle_deg[i] +=
                    self.boss.enemy_bullets.angular_vel_deg[i] / 60.0;
                let speed = (self.boss.enemy_bullets.vel_x[i] * self.boss.enemy_bullets.vel_x[i]
                    + self.boss.enemy_bullets.vel_y[i] * self.boss.enemy_bullets.vel_y[i])
                    .sqrt();
                let angle_rad = self.boss.enemy_bullets.angle_deg[i].to_radians();
                self.boss.enemy_bullets.vel_x[i] = angle_rad.cos() * speed;
                self.boss.enemy_bullets.vel_y[i] = angle_rad.sin() * speed;
            }
        }

        for i in 0..len {
            if self.boss.enemy_bullets.delay_frames[i] == 0 {
                self.boss.enemy_bullets.vel_x[i] += self.boss.enemy_bullets.accel_x[i] / 60.0;
                self.boss.enemy_bullets.vel_y[i] += self.boss.enemy_bullets.accel_y[i] / 60.0;
            }
        }

        for i in 0..len {
            self.boss.enemy_bullets.pos_x[i] += self.boss.enemy_bullets.vel_x[i] / 60.0;
            self.boss.enemy_bullets.pos_y[i] += self.boss.enemy_bullets.vel_y[i] / 60.0;
        }

        let mut index = 0;
        while index < self.boss.enemy_bullets.len() {
            let archetype_index = self.boss.enemy_bullets.archetype_id[index];
            let has_detonation = archetype_index != usize::MAX
                && self.bullet_archetypes[archetype_index].detonation.is_some();
            let out_of_bounds = self.boss.enemy_bullets.pos_x[index]
                < self.arena.arena.camera_bounds.min_x - 1.0
                || self.boss.enemy_bullets.pos_x[index] > self.arena.arena.camera_bounds.max_x + 1.0
                || self.boss.enemy_bullets.pos_y[index] < self.arena.arena.camera_bounds.min_y - 1.0
                || self.boss.enemy_bullets.pos_y[index] > self.arena.arena.camera_bounds.max_y + 1.0;
            let wall_hit = bullet_hits_wall(
                self.boss.enemy_bullets.pos_x[index],
                self.boss.enemy_bullets.pos_y[index],
                self.boss.enemy_bullets.radius[index],
                &self.arena.arena,
            );
            let die_on_wall = self.boss.enemy_bullets.flags[index] & BULLET_FLAG_DIE_ON_WALL != 0;
            let detonate_ready = has_detonation && self.boss.enemy_bullets.detonate_frames[index] == 0;
            let should_remove = self.boss.enemy_bullets.ttl_frames[index] == 0
                || out_of_bounds
                || (wall_hit && die_on_wall)
                || detonate_ready;
            if should_remove {
                if has_detonation
                    && (detonate_ready
                        || (wall_hit && die_on_wall)
                        || self.boss.enemy_bullets.ttl_frames[index] == 0)
                {
                    self.detonate_enemy_bullet(index);
                }
                self.boss.enemy_bullets.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn detonate_enemy_bullet(&mut self, bullet_index: usize) {
        let archetype_index = self.boss.enemy_bullets.archetype_id[bullet_index];
        let Some(detonation) = self.bullet_archetypes[archetype_index].detonation.clone() else {
            return;
        };
        let child_archetype_index = self.bullet_lookup[&detonation.bullet_id];
        let child_archetype = self.bullet_archetypes[child_archetype_index].clone();
        let origin_x = self.boss.enemy_bullets.pos_x[bullet_index];
        let origin_y = self.boss.enemy_bullets.pos_y[bullet_index];
        let burst_count = detonation.burst_count.max(1) as usize;
        for shot_index in 0..burst_count {
            let angle_deg = compute_angle_deg(
                &schema::EmitterDef {
                    source: schema::EmitterSource::Boss,
                    cadence_frames: 1,
                    start_frame: 0,
                    end_frame: 0,
                    burst_count: detonation.burst_count.max(1),
                    spread_deg: detonation.spread_deg,
                    base_angle_deg: match detonation.angle_mode {
                        schema::AngleMode::Fixed => {
                            self.boss.enemy_bullets.angle_deg[bullet_index] + detonation.base_angle_deg
                        }
                        _ => detonation.base_angle_deg,
                    },
                    angle_mode: detonation.angle_mode,
                    spin_speed_deg: 0.0,
                    speed_mode: schema::SpeedMode::Constant,
                    speed_scale_step: 0.0,
                    bullet_id: detonation.bullet_id.clone(),
                },
                shot_index,
                burst_count,
                detonation.spread_deg,
                0,
                self.player.pos_x - origin_x,
                self.player.pos_y - origin_y,
            );
            let angle_rad = angle_deg.to_radians();
            let mut angular_vel_deg = 0.0;
            let mut flags = 0_u32;
            match child_archetype.behavior {
                schema::BulletBehavior::TurnAfterDelay => angular_vel_deg = child_archetype.turn_rate_deg,
                schema::BulletBehavior::CircleAfterDelay => angular_vel_deg = child_archetype.turn_rate_deg,
                schema::BulletBehavior::Orbit => {
                    angular_vel_deg = child_archetype.turn_rate_deg;
                    flags |= crate::constants::BULLET_FLAG_ORBIT;
                }
                schema::BulletBehavior::Boomerang => flags |= crate::constants::BULLET_FLAG_BOOMERANG,
                schema::BulletBehavior::AccelerateAfterDelay | schema::BulletBehavior::Default => {}
            }
            if child_archetype.armor_piercing {
                flags |= crate::constants::BULLET_FLAG_ARMOR_PIERCING;
            }
            if child_archetype.die_on_wall {
                flags |= crate::constants::BULLET_FLAG_DIE_ON_WALL;
            }
            let color_rgba = crate::runtime::status::projectile_color(
                child_archetype.status_mask,
                child_archetype.color_rgba,
            );
            self.boss.enemy_bullets.push(crate::pool::bullet::SpawnedBullet {
                sprite: child_archetype.sprite,
                pos_x: origin_x,
                pos_y: origin_y,
                vel_x: angle_rad.cos() * child_archetype.speed,
                vel_y: angle_rad.sin() * child_archetype.speed,
                accel_x: angle_rad.cos() * child_archetype.accel,
                accel_y: angle_rad.sin() * child_archetype.accel,
                radius: child_archetype.radius,
                ttl_frames: child_archetype.lifetime_frames,
                angle_deg,
                angular_vel_deg,
                archetype_id: child_archetype_index,
                status_mask: child_archetype.status_mask,
                status_duration_frames: child_archetype.status_duration_frames,
                color_rgba,
                flags,
                delay_frames: child_archetype.delay_frames,
                detonate_frames: child_archetype
                    .detonation
                    .as_ref()
                    .map(|next| next.after_frames)
                    .unwrap_or(0),
                damage: child_archetype.damage,
                render_layer: child_archetype.render_layer,
            });
        }
    }
}

pub fn update_bullet_pool(pool: &mut BulletPool, arena: &schema::ArenaDef) {
    let len = pool.len();

    // Pass 1: Tick TTL (tight loop, auto-vectorizable)
    for i in 0..len {
        if pool.ttl_frames[i] > 0 {
            pool.ttl_frames[i] -= 1;
        }
    }

    // Pass 2: Tick delay (tight loop)
    for i in 0..len {
        if pool.delay_frames[i] > 0 {
            pool.delay_frames[i] -= 1;
        }
    }

    // Pass 3: Angular velocity (sparse - only bullets that turn)
    for i in 0..len {
        if pool.delay_frames[i] == 0 && pool.angular_vel_deg[i] != 0.0 {
            pool.angle_deg[i] += pool.angular_vel_deg[i] / 60.0;
            let speed = (pool.vel_x[i] * pool.vel_x[i] + pool.vel_y[i] * pool.vel_y[i]).sqrt();
            let angle_rad = pool.angle_deg[i].to_radians();
            pool.vel_x[i] = angle_rad.cos() * speed;
            pool.vel_y[i] = angle_rad.sin() * speed;
        }
    }

    // Pass 4: Apply acceleration to velocity (tight loop, auto-vectorizable)
    for i in 0..len {
        if pool.delay_frames[i] == 0 {
            pool.vel_x[i] += pool.accel_x[i] / 60.0;
            pool.vel_y[i] += pool.accel_y[i] / 60.0;
        }
    }

    // Pass 5: Apply velocity to position (tight loop, auto-vectorizable)
    for i in 0..len {
        pool.pos_x[i] += pool.vel_x[i] / 60.0;
        pool.pos_y[i] += pool.vel_y[i] / 60.0;
    }

    // Pass 6: Cull dead bullets (swap_remove, not vectorizable but runs once)
    let mut index = 0;
    while index < pool.len() {
        let out_of_bounds = pool.pos_x[index] < arena.camera_bounds.min_x - 1.0
            || pool.pos_x[index] > arena.camera_bounds.max_x + 1.0
            || pool.pos_y[index] < arena.camera_bounds.min_y - 1.0
            || pool.pos_y[index] > arena.camera_bounds.max_y + 1.0;
        let wall_hit = bullet_hits_wall(
            pool.pos_x[index],
            pool.pos_y[index],
            pool.radius[index],
            arena,
        );
        let die_on_wall = pool.flags[index] & BULLET_FLAG_DIE_ON_WALL != 0;
        if pool.ttl_frames[index] == 0 || out_of_bounds || (wall_hit && die_on_wall) {
            pool.swap_remove(index);
        } else {
            index += 1;
        }
    }
}
