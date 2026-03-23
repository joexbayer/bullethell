use schema::RenderLayer;

soa_pool! {
    pool BulletPool, spawn SpawnedBullet {
        sprite: u32,
        pos_x: f32,
        pos_y: f32,
        vel_x: f32,
        vel_y: f32,
        accel_x: f32,
        accel_y: f32,
        radius: f32,
        ttl_frames: u16,
        angle_deg: f32,
        angular_vel_deg: f32,
        archetype_id: usize,
        status_mask: u32,
        status_duration_frames: u16,
        color_rgba: [f32; 4],
        flags: u32,
        delay_frames: u16,
        detonate_frames: u16,
        damage: f32,
        render_layer: RenderLayer,
    }
}
