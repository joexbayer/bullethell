use schema::{GeneratorElement, STATUS_EXPOSED, STATUS_SICK, STATUS_SLOW};

use crate::constants::*;
use crate::runtime::Runtime;

impl Runtime {
    pub fn build_render_data(&mut self) {
        self.instances.clear();
        self.debug_lines.clear();
        let estimated = self.arena.arena.tiles.len()
            + self.boss.generators.len() * 4
            + self.boss.enemy_bullets.len()
            + self.boss.player_shots.len()
            + self.boss.helpers.len() * 4
            + self.boss.objects.len() * 4
            + 8;
        self.instances.reserve(estimated * INSTANCE_FLOATS);
        self.render_tiles();
        self.render_generators();
        self.render_enemy_bullets();
        self.render_player_shots();
        self.render_helpers();
        self.render_objects();
        self.render_boss();
        self.render_player();
        self.render_debug_hitboxes();
    }

    fn render_tiles(&mut self) {
        for (index, tile_kind) in self.arena.arena.tiles.iter().enumerate() {
            if *tile_kind == 0 {
                continue;
            }
            let tile_x = index as u32 % self.arena.arena.width;
            let tile_y = index as u32 / self.arena.arena.width;
            let x = tile_x as f32 * self.arena.arena.tile_size;
            let y = tile_y as f32 * self.arena.arena.tile_size;
            let is_edge_wall = *tile_kind == 1
                && (tile_x == 0
                    || tile_y == 0
                    || tile_x == self.arena.arena.width - 1
                    || tile_y == self.arena.arena.height - 1);
            let color = match tile_kind {
                1 if is_edge_wall => [0.92, 0.95, 1.0, 1.0],
                1 => [0.62, 0.70, 0.84, 1.0],
                2 => [0.80, 0.42, 0.56, 1.0],
                _ => [0.40, 0.46, 0.54, 1.0],
            };
            let sprite = match tile_kind {
                1 if is_edge_wall => SPRITE_EDGE_WALL,
                1 => SPRITE_TILE,
                2 => SPRITE_EDGE_WALL,
                _ => SPRITE_TILE,
            };
            push_instance(
                &mut self.instances,
                x + self.arena.arena.tile_size * 0.5,
                y + self.arena.arena.tile_size * 0.5,
                self.arena.arena.tile_size,
                self.arena.arena.tile_size,
                0.0,
                sprite,
                color,
                0.0,
                1.0,
                1.0,
                0.0,
                0.0,
            );
        }
    }

    fn render_generators(&mut self) {
        for index in 0..self.boss.generators.len() {
            let base_color = match self.boss.generators.element[index] {
                GeneratorElement::Fire => [1.0, 0.56, 0.20, 1.0],
                GeneratorElement::Ice => [0.58, 0.88, 1.0, 1.0],
            };
            let alpha = if self.boss.generators.sealed[index] {
                0.42
            } else {
                0.95
            };
            let ring_color = [base_color[0], base_color[1], base_color[2], alpha];
            let core_scale = if self.boss.generators.sealed[index] {
                0.70
            } else {
                0.92
            };
            let glow = if self.boss.generators.sealed[index] {
                0.0
            } else {
                0.4
            };
            push_instance(
                &mut self.instances,
                self.boss.generators.pos_x[index],
                self.boss.generators.pos_y[index],
                self.boss.generators.radius[index] * 2.4,
                self.boss.generators.radius[index] * 2.4,
                0.0,
                SPRITE_GENERATOR_RING,
                ring_color,
                0.8,
                1.0,
                0.0,
                glow,
                1.0,
            );
            push_instance(
                &mut self.instances,
                self.boss.generators.pos_x[index],
                self.boss.generators.pos_y[index],
                self.boss.generators.radius[index] * 2.0 * core_scale,
                self.boss.generators.radius[index] * 2.0 * core_scale,
                45.0,
                SPRITE_GENERATOR_CORE,
                if self.boss.generators.vulnerable[index]
                    && !self.boss.generators.sealed[index]
                {
                    [base_color[0], base_color[1], base_color[2], 1.0]
                } else {
                    [
                        base_color[0] * 0.72,
                        base_color[1] * 0.72,
                        base_color[2] * 0.72,
                        alpha,
                    ]
                },
                0.82,
                1.0,
                0.0,
                0.0,
                0.0,
            );
            if self.boss.generators.vulnerable[index] && !self.boss.generators.sealed[index] {
                let hp_ratio = (self.boss.generators.hp[index]
                    / self.boss.generators.max_hp[index])
                    .clamp(0.0, 1.0);
                let gx = self.boss.generators.pos_x[index];
                let gy = self.boss.generators.pos_y[index];
                let gr = self.boss.generators.radius[index];
                let bar_width = 1.3;
                let bar_height = 0.16;
                let bar_inner = bar_width - 0.08;
                let theta = self.world_rotation_deg.to_radians();
                let sin_t = theta.sin();
                let cos_t = theta.cos();
                let d = gr + 0.36;
                let bar_x = gx - sin_t * d;
                let bar_y = gy + cos_t * d;
                let bar_rot = -self.world_rotation_deg;
                push_instance(
                    &mut self.instances,
                    bar_x,
                    bar_y,
                    bar_width,
                    bar_height,
                    bar_rot,
                    SPRITE_UI_RECT,
                    [0.10, 0.12, 0.18, 0.96],
                    0.84,
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                );
                let fill_width = bar_inner * hp_ratio;
                if fill_width > 0.0 {
                    let shift = (bar_inner - fill_width) * 0.5;
                    push_instance(
                        &mut self.instances,
                        bar_x - cos_t * shift,
                        bar_y - sin_t * shift,
                        fill_width,
                        bar_height - 0.04,
                        bar_rot,
                        SPRITE_UI_RECT,
                        [0.94, 0.98, 0.88, 1.0],
                        0.86,
                        1.0,
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }
        }
    }

    fn render_enemy_bullets(&mut self) {
        for index in 0..self.boss.enemy_bullets.len() {
            if self.boss.enemy_bullets.delay_frames[index] > 0 {
                // Telegraph: pulsing ring that shrinks as delay decreases
                let max_delay = 60.0_f32;
                let ratio = (self.boss.enemy_bullets.delay_frames[index] as f32 / max_delay)
                    .clamp(0.0, 1.0);
                let pulse = 1.0 + (self.frame as f32 / 4.0).sin() * 0.15;
                let size = self.boss.enemy_bullets.radius[index] * (4.0 + ratio * 3.0) * pulse;
                let c = self.boss.enemy_bullets.color_rgba[index];
                push_instance(
                    &mut self.instances,
                    self.boss.enemy_bullets.pos_x[index],
                    self.boss.enemy_bullets.pos_y[index],
                    size,
                    size,
                    self.frame as f32 * 3.0,
                    SPRITE_RING,
                    [c[0], c[1], c[2], 0.35 + ratio * 0.25],
                    0.9,
                    1.0,
                    0.0,
                    0.3,
                    1.0,
                );
            } else {
                let render_diameter = self.boss.enemy_bullets.radius[index] * 3.4;
                push_instance(
                    &mut self.instances,
                    self.boss.enemy_bullets.pos_x[index],
                    self.boss.enemy_bullets.pos_y[index],
                    render_diameter,
                    render_diameter,
                    self.boss.enemy_bullets.angle_deg[index],
                    self.boss.enemy_bullets.sprite[index],
                    self.boss.enemy_bullets.color_rgba[index],
                    1.0,
                    1.0,
                    0.0,
                    0.6,
                    1.0,
                );
            }
        }
    }

    fn render_player_shots(&mut self) {
        for index in 0..self.boss.player_shots.len() {
            push_instance(
                &mut self.instances,
                self.boss.player_shots.pos_x[index],
                self.boss.player_shots.pos_y[index],
                self.boss.player_shots.radius[index] * 3.0,
                self.boss.player_shots.radius[index] * 3.0,
                self.boss.player_shots.angle_deg[index],
                self.boss.player_shots.sprite[index],
                self.boss.player_shots.color_rgba[index],
                2.0,
                1.0,
                0.0,
                0.45,
                1.0,
            );
        }
    }

    fn render_helpers(&mut self) {
        for index in 0..self.boss.helpers.len() {
            let r = self.boss.helpers.radius[index];
            let hx = self.boss.helpers.pos_x[index];
            let hy = self.boss.helpers.pos_y[index];
            let c = self.boss.helpers.color_rgba[index];
            let hp_ratio = (self.boss.helpers.hp[index] / self.boss.helpers.max_hp[index])
                .clamp(0.0, 1.0);

            // Enemy: solid background square so it reads as a unit, not a bullet
            push_instance(
                &mut self.instances,
                hx, hy,
                r * 3.6, r * 3.6,
                0.0,
                SPRITE_BOSS,
                [c[0] * 0.22, c[1] * 0.22, c[2] * 0.22, 0.90],
                2.98,
                1.0, 0.0,
                0.0, 0.0,
            );
            // Sprite on top
            push_instance(
                &mut self.instances,
                hx, hy,
                r * 2.8, r * 2.8,
                self.boss.helpers.angle_deg[index],
                self.boss.helpers.sprite[index],
                c,
                3.0,
                1.0, 0.0,
                0.0, 0.0,
            );

            // HP bar: rotates with camera, always below entity in screen space
            let bar_width = (r * 2.5).max(0.9);
            let bar_height = 0.14;
            let bar_inner = bar_width - 0.06;
            let theta = self.world_rotation_deg.to_radians();
            let sin_t = theta.sin();
            let cos_t = theta.cos();
            let d = r + 0.32;
            let bar_x = hx - sin_t * d;
            let bar_y = hy + cos_t * d;
            let bar_rot = -self.world_rotation_deg;
            push_instance(
                &mut self.instances,
                bar_x, bar_y,
                bar_width, bar_height,
                bar_rot,
                SPRITE_UI_RECT,
                [0.08, 0.09, 0.12, 0.96],
                3.06,
                1.0, 0.0,
                0.0, 0.0,
            );
            let fill_width = bar_inner * hp_ratio;
            if fill_width > 0.0 {
                let shift = (bar_inner - fill_width) * 0.5;
                let fill_x = bar_x - cos_t * shift;
                let fill_y = bar_y - sin_t * shift;
                push_instance(
                    &mut self.instances,
                    fill_x, fill_y,
                    fill_width, bar_height - 0.04,
                    bar_rot,
                    SPRITE_UI_RECT,
                    [0.18, 0.92, 0.40, 1.0],
                    3.07,
                    1.0, 0.0,
                    0.0, 0.0,
                );
            }
        }
    }

    fn render_objects(&mut self) {
        for index in 0..self.boss.objects.len() {
            let r = self.boss.objects.radius[index];
            let ox = self.boss.objects.pos_x[index];
            let oy = self.boss.objects.pos_y[index];
            let c = self.boss.objects.color_rgba[index];
            let hp_ratio = (self.boss.objects.hp[index] / self.boss.objects.max_hp[index])
                .clamp(0.0, 1.0);

            // Enemy: solid background
            push_instance(
                &mut self.instances,
                ox, oy,
                r * 3.6, r * 3.6,
                45.0,
                SPRITE_BOSS,
                [c[0] * 0.22, c[1] * 0.22, c[2] * 0.22, 0.90],
                3.28,
                1.0, 0.0,
                0.0, 0.0,
            );
            push_instance(
                &mut self.instances,
                ox, oy,
                r * 2.8, r * 2.8,
                self.boss.objects.angle_deg[index],
                self.boss.objects.sprite[index],
                c,
                3.3,
                1.0, 0.0,
                0.0, 0.0,
            );

            // HP bar: rotates with camera, always below entity in screen space
            let bar_width = (r * 2.5).max(1.1);
            let bar_height = 0.16;
            let bar_inner = bar_width - 0.06;
            let theta = self.world_rotation_deg.to_radians();
            let sin_t = theta.sin();
            let cos_t = theta.cos();
            let d = r + 0.38;
            let bar_x = ox - sin_t * d;
            let bar_y = oy + cos_t * d;
            let bar_rot = -self.world_rotation_deg;
            push_instance(
                &mut self.instances,
                bar_x, bar_y,
                bar_width, bar_height,
                bar_rot,
                SPRITE_UI_RECT,
                [0.06, 0.09, 0.07, 0.98],
                3.34,
                1.0, 0.0,
                0.0, 0.0,
            );
            let fill_width = bar_inner * hp_ratio;
            if fill_width > 0.0 {
                let shift = (bar_inner - fill_width) * 0.5;
                let fill_x = bar_x - cos_t * shift;
                let fill_y = bar_y - sin_t * shift;
                push_instance(
                    &mut self.instances,
                    fill_x, fill_y,
                    fill_width, bar_height - 0.05,
                    bar_rot,
                    SPRITE_UI_RECT,
                    [0.14, 0.96, 0.28, 1.0],
                    3.35,
                    1.0, 0.0,
                    0.0, 0.0,
                );
            }
        }
    }

    fn render_boss(&mut self) {
        // Boss aura: 2 concentric translucent rings, slowly rotating
        let aura_alpha = 0.12;
        push_instance(
            &mut self.instances,
            self.boss.pos_x,
            self.boss.pos_y,
            self.boss.radius * 4.2,
            self.boss.radius * 4.2,
            self.frame as f32 * 0.5,
            SPRITE_RING,
            [0.42, 0.24, 0.60, aura_alpha],
            3.9,
            1.0,
            0.0,
            0.2,
            1.0,
        );
        push_instance(
            &mut self.instances,
            self.boss.pos_x,
            self.boss.pos_y,
            self.boss.radius * 3.4,
            self.boss.radius * 3.4,
            -(self.frame as f32) * 0.8,
            SPRITE_RING,
            [0.58, 0.36, 0.80, aura_alpha * 1.5],
            3.91,
            1.0,
            0.0,
            0.15,
            1.0,
        );

        if self.boss_is_invulnerable() || self.boss_is_armored() {
            let pulse = 1.0 + (self.frame as f32 / 8.0).sin() * 0.05;
            let ring_color = if self.boss_is_invulnerable() {
                [1.0, 0.94, 0.62, 0.92]
            } else {
                [0.74, 0.88, 1.0, 0.88]
            };
            push_instance(
                &mut self.instances,
                self.boss.pos_x,
                self.boss.pos_y,
                self.boss.radius * 2.9 * pulse,
                self.boss.radius * 2.9 * pulse,
                self.frame as f32 * 2.5,
                SPRITE_GENERATOR_RING,
                ring_color,
                3.95,
                1.0,
                0.0,
                0.6,
                1.0,
            );
        }
        push_instance(
            &mut self.instances,
            self.boss.pos_x,
            self.boss.pos_y,
            self.boss.radius * 2.6,
            self.boss.radius * 2.6,
            0.0,
            SPRITE_BOSS,
            [0.42, 0.24, 0.60, 1.0],
            4.0,
            1.0,
            0.0,
            0.0,
            0.0,
        );
    }

    fn render_player(&mut self) {
        // Status effect rings
        if self.player.status_mask & STATUS_SLOW != 0 {
            let pulse = 1.0 + (self.frame as f32 / 6.0).sin() * 0.1;
            push_instance(
                &mut self.instances,
                self.player.pos_x,
                self.player.pos_y,
                PLAYER_RENDER_RADIUS * 3.6 * pulse,
                PLAYER_RENDER_RADIUS * 3.6 * pulse,
                self.frame as f32 * 2.0,
                SPRITE_RING,
                [0.4, 0.6, 1.0, 0.55],
                4.9,
                1.0,
                0.0,
                0.4,
                1.0,
            );
        }
        if self.player.status_mask & STATUS_SICK != 0 {
            push_instance(
                &mut self.instances,
                self.player.pos_x,
                self.player.pos_y,
                PLAYER_RENDER_RADIUS * 3.2,
                PLAYER_RENDER_RADIUS * 3.2,
                -(self.frame as f32) * 1.5,
                SPRITE_RING,
                [0.3, 0.9, 0.3, 0.50],
                4.91,
                1.0,
                0.0,
                0.35,
                1.0,
            );
        }
        if self.player.status_mask & STATUS_EXPOSED != 0 {
            push_instance(
                &mut self.instances,
                self.player.pos_x,
                self.player.pos_y,
                PLAYER_RENDER_RADIUS * 3.0,
                PLAYER_RENDER_RADIUS * 3.0,
                self.frame as f32 * 3.0,
                SPRITE_RING,
                [1.0, 0.85, 0.2, 0.50],
                4.92,
                1.0,
                0.0,
                0.35,
                1.0,
            );
        }

        push_instance(
            &mut self.instances,
            self.player.pos_x,
            self.player.pos_y,
            PLAYER_RENDER_RADIUS * 3.2,
            PLAYER_RENDER_RADIUS * 3.2,
            0.0,
            SPRITE_PLAYER,
            [0.90, 0.96, 1.0, 1.0],
            5.0,
            1.0,
            0.0,
            0.0,
            0.0,
        );
    }

    fn render_debug_hitboxes(&mut self) {
        if !(self.debug_enabled && self.debug_hitboxes) {
            return;
        }
        push_circle_debug(
            &mut self.debug_lines,
            self.player.pos_x,
            self.player.pos_y,
            PLAYER_RADIUS,
        );
        push_circle_debug(
            &mut self.debug_lines,
            self.boss.pos_x,
            self.boss.pos_y,
            self.boss.radius,
        );
        for index in 0..self.boss.helpers.len() {
            push_circle_debug(
                &mut self.debug_lines,
                self.boss.helpers.pos_x[index],
                self.boss.helpers.pos_y[index],
                self.boss.helpers.radius[index],
            );
        }
        for index in 0..self.boss.objects.len() {
            push_circle_debug(
                &mut self.debug_lines,
                self.boss.objects.pos_x[index],
                self.boss.objects.pos_y[index],
                self.boss.objects.radius[index],
            );
        }
        for index in 0..self.boss.generators.len() {
            if self.boss.generators.vulnerable[index] && !self.boss.generators.sealed[index] {
                push_circle_debug(
                    &mut self.debug_lines,
                    self.boss.generators.pos_x[index],
                    self.boss.generators.pos_y[index],
                    self.boss.generators.radius[index],
                );
            }
        }
    }
}

pub fn push_instance(
    instances: &mut Vec<f32>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation_deg: f32,
    sprite: u32,
    color: [f32; 4],
    layer: f32,
    world_rotate: f32,
    world_spin: f32,
    glow: f32,
    blend_mode: f32,
) {
    let screen_lock = if sprite == SPRITE_PLAYER { 1.0 } else { 0.0 };
    instances.extend_from_slice(&[
        x,
        y,
        width,
        height,
        rotation_deg,
        sprite as f32,
        color[0],
        color[1],
        color[2],
        color[3],
        layer,
        world_rotate,
        world_spin,
        screen_lock,
        glow,
        blend_mode,
    ]);
}

fn push_circle_debug(lines: &mut Vec<f32>, center_x: f32, center_y: f32, radius: f32) {
    let segments = 24;
    for index in 0..segments {
        let a0 = index as f32 / segments as f32 * std::f32::consts::TAU;
        let a1 = (index + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        lines.extend_from_slice(&[
            center_x + a0.cos() * radius,
            center_y + a0.sin() * radius,
            center_x + a1.cos() * radius,
            center_y + a1.sin() * radius,
        ]);
    }
}
