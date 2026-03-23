use schema::ArenaDef;

use crate::constants::BULLET_FLAG_DIE_ON_WALL;
use crate::pool::bullet::BulletPool;
use crate::runtime::collision::bullet_hits_wall;
use crate::runtime::Runtime;

impl Runtime {
    pub fn update_bullets(&mut self) {
        update_bullet_pool(&mut self.boss.enemy_bullets, &self.arena.arena);
        update_bullet_pool(&mut self.boss.player_shots, &self.arena.arena);
    }
}

pub fn update_bullet_pool(pool: &mut BulletPool, arena: &ArenaDef) {
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
