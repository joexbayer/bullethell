use schema::HelperMotion;

soa_pool! {
    pool HelperPool, spawn HelperSpawn {
        ids: String,
        sprite: u32,
        pos_x: f32,
        pos_y: f32,
        hp: f32,
        max_hp: f32,
        radius: f32,
        motion: HelperMotion,
        orbit_radius: f32,
        orbit_speed_deg: f32,
        angle_deg: f32,
        bullet_pattern: Option<usize>,
        color_rgba: [f32; 4],
    }
}

impl HelperPool {
    pub fn find_index(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|existing| existing == id)
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.find_index(id).is_some()
    }

    pub fn remove_id(&mut self, id: &str) -> bool {
        let Some(index) = self.find_index(id) else {
            return false;
        };
        self.swap_remove(index);
        true
    }
}
