use schema::GeneratorElement;

soa_pool! {
    pool GeneratorPool, spawn GeneratorSpawn {
        ids: String,
        pos_x: f32,
        pos_y: f32,
        hp: f32,
        max_hp: f32,
        radius: f32,
        element: GeneratorElement,
        sealed: bool,
        vulnerable: bool,
    }
}

impl GeneratorPool {
    pub fn find_index(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|existing| existing == id)
    }

    pub fn sealed_count(&self) -> usize {
        self.sealed.iter().filter(|sealed| **sealed).count()
    }
}
