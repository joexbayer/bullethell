use schema::ObjectMotion;

soa_pool! {
    pool EncounterObjectPool, spawn EncounterObjectSpawn {
        ids: String,
        sprite: u32,
        pos_x: f32,
        pos_y: f32,
        hp: f32,
        max_hp: f32,
        radius: f32,
        motion: ObjectMotion,
        anchor_x: f32,
        anchor_y: f32,
        orbit_radius: f32,
        orbit_speed_deg: f32,
        angle_deg: f32,
        bullet_pattern: Option<usize>,
        color_rgba: [f32; 4],
        transition_frames: u16,
        transition_state: u8,
    }
}

impl EncounterObjectPool {
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
