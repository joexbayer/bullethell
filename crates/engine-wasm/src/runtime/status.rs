use schema::{STATUS_ARMOR_BROKEN, STATUS_ARMORED, STATUS_INVULNERABLE};

use crate::constants::MAX_STATUS_SLOTS;
use crate::types::StatusView;

use schema::{STATUS_EXPOSED, STATUS_SICK, STATUS_SILENCED, STATUS_SLOW};

#[derive(Clone, Copy, Default)]
pub struct StatusTimer {
    pub mask: u32,
    pub frames_left: u16,
}

pub fn tick_status_array(statuses: &mut [StatusTimer; MAX_STATUS_SLOTS], status_mask: &mut u32) {
    *status_mask = 0;
    for status in statuses.iter_mut() {
        if status.frames_left > 0 {
            status.frames_left -= 1;
            *status_mask |= status.mask;
        }
    }
}

pub fn apply_status(
    statuses: &mut [StatusTimer; MAX_STATUS_SLOTS],
    status_mask: &mut u32,
    incoming_mask: u32,
    duration_frames: u16,
) {
    if incoming_mask == 0 {
        return;
    }
    for bit in 0..32 {
        let mask = 1_u32 << bit;
        if incoming_mask & mask == 0 {
            continue;
        }
        if let Some(slot) = statuses
            .iter_mut()
            .find(|slot| slot.mask == mask || slot.frames_left == 0)
        {
            slot.mask = mask;
            slot.frames_left = duration_frames.max(slot.frames_left);
        }
        *status_mask |= mask;
    }
}

pub fn projectile_color(status_mask: u32, base: [f32; 4]) -> [f32; 4] {
    if status_mask & STATUS_SLOW != 0 {
        return [0.44, 0.90, 1.0, base[3]];
    }
    if status_mask & STATUS_SICK != 0 {
        return [0.58, 1.0, 0.36, base[3]];
    }
    if status_mask & STATUS_SILENCED != 0 {
        return [0.84, 0.60, 1.0, base[3]];
    }
    if status_mask & STATUS_EXPOSED != 0 {
        return [1.0, 0.88, 0.36, base[3]];
    }
    base
}

pub fn collect_status_views(statuses: &[StatusTimer; MAX_STATUS_SLOTS]) -> Vec<StatusView> {
    let mut views = Vec::new();
    for status in statuses.iter().filter(|status| status.frames_left > 0) {
        views.push(StatusView {
            id: status_id(status.mask).to_string(),
            label: status_label(status.mask).to_string(),
            frames_left: status.frames_left,
        });
    }
    views.sort_by(|a, b| a.label.cmp(&b.label));
    views
}

fn status_id(mask: u32) -> &'static str {
    match mask {
        STATUS_SLOW => "slow",
        STATUS_SICK => "sick",
        STATUS_SILENCED => "silenced",
        STATUS_ARMOR_BROKEN => "armor_broken",
        STATUS_EXPOSED => "exposed",
        STATUS_INVULNERABLE => "invulnerable",
        STATUS_ARMORED => "armored",
        _ => "unknown",
    }
}

fn status_label(mask: u32) -> &'static str {
    match mask {
        STATUS_SLOW => "Slow",
        STATUS_SICK => "Sick",
        STATUS_SILENCED => "Silenced",
        STATUS_ARMOR_BROKEN => "Armor Broken",
        STATUS_EXPOSED => "Exposed",
        STATUS_INVULNERABLE => "Invulnerable",
        STATUS_ARMORED => "Armored",
        _ => "Unknown",
    }
}
