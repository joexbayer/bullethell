use schema::{GeneratorElement, PatternFamily};

use crate::constants::*;
use crate::rng::Rng64;
use crate::runtime::Runtime;

impl Runtime {
    pub fn seal_generator(&mut self, index: usize) {
        self.boss.generators.sealed[index] = true;
        self.boss.generators.vulnerable[index] = false;
        self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
        let gc = match self.boss.generators.element[index] {
            GeneratorElement::Fire => [1.0_f32, 0.56, 0.20],
            GeneratorElement::Ice => [0.58_f32, 0.88, 1.0],
        };
        self.events.extend_from_slice(&[
            EVENT_GENERATOR_SEALED,
            self.boss.generators.pos_x[index],
            self.boss.generators.pos_y[index],
            gc[0], gc[1], gc[2], 0.0,
        ]);
        let label = match self.boss.generators.element[index] {
            GeneratorElement::Fire => "Fire",
            GeneratorElement::Ice => "Ice",
        };
        self.current_message = format!("Generator sealed: {label} locked");
    }

    pub fn apply_legacy_lock_counts_to_generators(&mut self) {
        let mut remaining_fire = self.boss.fire_locks as usize;
        let mut remaining_ice = self.boss.ice_locks as usize;
        for index in 0..self.boss.generators.len() {
            if remaining_fire > 0 {
                self.boss.generators.element[index] = GeneratorElement::Fire;
                self.boss.generators.sealed[index] = true;
                self.boss.generators.vulnerable[index] = false;
                remaining_fire -= 1;
            } else if remaining_ice > 0 {
                self.boss.generators.element[index] = GeneratorElement::Ice;
                self.boss.generators.sealed[index] = true;
                self.boss.generators.vulnerable[index] = false;
                remaining_ice -= 1;
            } else {
                self.boss.generators.sealed[index] = false;
                self.boss.generators.vulnerable[index] = false;
                self.boss.generators.hp[index] = self.boss.generators.max_hp[index];
            }
        }
    }

    pub fn select_generator_family(&mut self, rng: &mut Rng64) -> (PatternFamily, bool) {
        if self.boss.generators.len() == 0 {
            return select_family(
                self.encounter.boss.generator_count,
                self.boss.fire_locks,
                self.boss.ice_locks,
                rng,
            );
        }
        let mut fire = 0_u8;
        let mut ice = 0_u8;
        for index in 0..self.boss.generators.len() {
            if !self.boss.generators.sealed[index] {
                self.boss.generators.element[index] = if rng.next_f32() > 0.5 {
                    GeneratorElement::Fire
                } else {
                    GeneratorElement::Ice
                };
            }
            match self.boss.generators.element[index] {
                GeneratorElement::Fire => fire += 1,
                GeneratorElement::Ice => ice += 1,
            }
        }
        self.boss.fire_locks = self
            .boss
            .generators
            .element
            .iter()
            .zip(self.boss.generators.sealed.iter())
            .filter(|(element, sealed)| **sealed && **element == GeneratorElement::Fire)
            .count() as u8;
        self.boss.ice_locks = self
            .boss
            .generators
            .element
            .iter()
            .zip(self.boss.generators.sealed.iter())
            .filter(|(element, sealed)| **sealed && **element == GeneratorElement::Ice)
            .count() as u8;
        if fire == ice {
            (PatternFamily::Neutral, false)
        } else if fire > ice {
            (
                PatternFamily::Fire,
                fire as usize == self.boss.generators.len(),
            )
        } else {
            (
                PatternFamily::Ice,
                ice as usize == self.boss.generators.len(),
            )
        }
    }
}

pub fn select_family(
    generator_count: u8,
    fire_locks: u8,
    ice_locks: u8,
    rng: &mut Rng64,
) -> (PatternFamily, bool) {
    let unlocked = generator_count.saturating_sub(fire_locks + ice_locks);
    let mut fire = fire_locks;
    let mut ice = ice_locks;
    for _ in 0..unlocked {
        if rng.next_f32() > 0.5 {
            fire += 1;
        } else {
            ice += 1;
        }
    }
    if fire == ice {
        (PatternFamily::Neutral, false)
    } else if fire > ice {
        (PatternFamily::Fire, fire == generator_count)
    } else {
        (PatternFamily::Ice, ice == generator_count)
    }
}
