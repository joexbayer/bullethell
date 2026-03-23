use schema::CONTENT_VERSION;
use wasm_bindgen::prelude::*;

pub mod constants;
pub mod game;
pub mod pool;
pub mod rng;
pub mod runtime;
pub mod types;

use constants::INSTANCE_FLOATS;
use game::{Game, ReplayState};
use rng::Rng64;
use runtime::Runtime;
use types::*;

#[derive(Default)]
struct GlobalState {
    game: Option<Game>,
}

thread_local! {
    static STATE: std::cell::RefCell<GlobalState> = std::cell::RefCell::new(GlobalState::default());
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn init_game(
    content_blob: &[u8],
    atlas_meta: JsValue,
    config: JsValue,
) -> Result<(), JsValue> {
    let content = schema::CompiledContent::decode(content_blob)
        .map_err(|error| JsValue::from_str(&format!("failed to decode content blob: {error}")))?;
    if content.version != CONTENT_VERSION {
        return Err(JsValue::from_str("content version mismatch"));
    }
    let atlas_meta: AtlasMeta = serde_wasm_bindgen::from_value(atlas_meta)
        .map_err(|error| JsValue::from_str(&format!("invalid atlas meta: {error}")))?;
    let config: GameConfig = serde_wasm_bindgen::from_value(config)
        .map_err(|error| JsValue::from_str(&format!("invalid config: {error}")))?;
    let seed = 0xA5A5_4D3C_2B1A_9087;
    let runtime = Runtime::new(&content, content.encounters[0].id.clone())?;
    let replay = ReplayState::new(seed);
    STATE.with(|state| {
        state.borrow_mut().game = Some(Game {
            content,
            _atlas_meta: atlas_meta,
            _config: config,
            rng: Rng64::new(seed),
            runtime,
            replay,
        });
    });
    Ok(())
}

#[wasm_bindgen]
pub fn load_encounter(encounter_id: &str) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        game.runtime = Runtime::new(&game.content, encounter_id.to_string())?;
        game.replay.recorded_inputs.clear();
        game.replay.playback = None;
        game.replay.playback_cursor = 0;
        Ok(())
    })
}

#[wasm_bindgen]
pub fn step(input_snapshot: JsValue) -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let mut input: InputSnapshot = serde_wasm_bindgen::from_value(input_snapshot)
            .map_err(|error| JsValue::from_str(&format!("invalid input snapshot: {error}")))?;
        if let Some(playback) = game.replay.playback.as_ref() {
            if game.replay.playback_cursor < playback.inputs.len() {
                input = playback.inputs[game.replay.playback_cursor].clone();
                game.replay.playback_cursor += 1;
            }
        } else {
            game.replay.recorded_inputs.push(input.clone());
        }
        let meta = game.step(input);
        serde_wasm_bindgen::to_value(&meta)
            .map_err(|error| JsValue::from_str(&format!("failed to serialize frame meta: {error}")))
    })
}

#[wasm_bindgen]
pub fn get_render_views() -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        let game = state
            .game
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let views = RenderViews {
            instance_ptr: game.runtime.instances.as_ptr() as u32,
            instance_len: game.runtime.instances.len() as u32,
            tile_ptr: game.runtime.arena.arena.tiles.as_ptr() as u32,
            tile_len: game.runtime.arena.arena.tiles.len() as u32,
            tile_width: game.runtime.arena.arena.width,
            tile_height: game.runtime.arena.arena.height,
            debug_ptr: game.runtime.debug_lines.as_ptr() as u32,
            debug_len: game.runtime.debug_lines.len() as u32,
            event_ptr: game.runtime.events.as_ptr() as u32,
            event_len: game.runtime.events.len() as u32,
            floats_per_instance: INSTANCE_FLOATS as u32,
        };
        serde_wasm_bindgen::to_value(&views).map_err(|error| {
            JsValue::from_str(&format!("failed to serialize render views: {error}"))
        })
    })
}

#[wasm_bindgen]
pub fn debug_command(cmd: JsValue) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let cmd: DebugCommand = serde_wasm_bindgen::from_value(cmd)
            .map_err(|error| JsValue::from_str(&format!("invalid debug command: {error}")))?;
        match cmd {
            DebugCommand::ToggleOverlay => {
                game.runtime.debug_enabled = !game.runtime.debug_enabled
            }
            DebugCommand::Pause(value) => game.runtime.paused = value,
            DebugCommand::SlowMo(value) => game.runtime.slow_mo = value,
            DebugCommand::Step => {
                if game.runtime.paused {
                    game.runtime.advance_one_frame(&mut game.rng);
                }
            }
            DebugCommand::ToggleHitboxes => {
                game.runtime.debug_hitboxes = !game.runtime.debug_hitboxes
            }
            DebugCommand::SeekReplayFrame(frame) => {
                if let Some(playback) = game.replay.playback.clone() {
                    let encounter_id = playback.encounter_id.clone();
                    game.runtime = Runtime::new(&game.content, encounter_id)?;
                    game.replay.playback = None;
                    game.replay.playback_cursor = 0;
                    for input in playback.inputs.iter().take(frame as usize).cloned() {
                        let _ = game.step(input);
                    }
                    game.replay.playback = Some(playback);
                    game.replay.playback_cursor = frame as usize;
                }
            }
        }
        Ok(())
    })
}

#[wasm_bindgen]
pub fn start_replay(seed: u64) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        game.rng = Rng64::new(seed);
        game.replay = ReplayState::new(seed);
        game.runtime = Runtime::new(&game.content, game.runtime.encounter_id.clone())?;
        Ok(())
    })
}

#[wasm_bindgen]
pub fn load_replay(replay_blob: JsValue) -> Result<(), JsValue> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let game = state
            .game
            .as_mut()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let replay: ReplayBlob = serde_wasm_bindgen::from_value(replay_blob)
            .map_err(|error| JsValue::from_str(&format!("invalid replay blob: {error}")))?;
        game.rng = Rng64::new(replay.seed);
        game.runtime = Runtime::new(&game.content, replay.encounter_id.clone())?;
        game.replay.playback_cursor = 0;
        game.replay.playback = Some(replay);
        Ok(())
    })
}

#[wasm_bindgen]
pub fn export_replay() -> Result<JsValue, JsValue> {
    STATE.with(|state| {
        let state = state.borrow();
        let game = state
            .game
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        let replay = ReplayBlob {
            version: CONTENT_VERSION,
            seed: game.replay.recording_seed,
            encounter_id: game.runtime.encounter_id.clone(),
            inputs: game.replay.recorded_inputs.clone(),
        };
        serde_wasm_bindgen::to_value(&replay)
            .map_err(|error| JsValue::from_str(&format!("failed to serialize replay: {error}")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::{
        AngleMode, AttackSelectorDef, AuthorArenaDef, AuthorRoot, BossDef, BulletArchetypeDef,
        BulletBehavior, CommandDef, EmitterDef, EmitterSource, EncounterDef, GeneratorDef,
        HelperMotion, ObjectMotion, PatternDef, PatternFamily, PhaseDef, RectDef, TransitionDef,
        TransitionConditionDef, Vec2Def, STATUS_SLOW,
    };

    fn test_runtime() -> Runtime {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![BulletArchetypeDef {
                id: "shot".to_string(),
                sprite: 1,
                radius: 0.2,
                damage: 10.0,
                lifetime_frames: 10,
                speed: 6.0,
                accel: 0.0,
                turn_rate_deg: 90.0,
                delay_frames: 0,
                status_mask: STATUS_SLOW,
                status_duration_frames: 90,
                behavior: BulletBehavior::Default,
                render_layer: schema::RenderLayer::EnemyBullets,
                color_rgba: [1.0, 0.2, 0.2, 1.0],
                die_on_wall: true,
                armor_piercing: false,
            }],
            patterns: vec![PatternDef {
                id: "pattern".to_string(),
                family: PatternFamily::Fire,
                nuke: false,
                duration_frames: 30,
                interruption_damage: Some(20.0),
                emitters: vec![EmitterDef {
                    source: EmitterSource::Boss,
                    cadence_frames: 1,
                    start_frame: 0,
                    end_frame: 1,
                    burst_count: 1,
                    spread_deg: 0.0,
                    base_angle_deg: 180.0,
                    angle_mode: AngleMode::Fixed,
                    spin_speed_deg: 0.0,
                    speed_mode: schema::SpeedMode::Constant,
                    speed_scale_step: 0.0,
                    bullet_id: "shot".to_string(),
                }],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 3,
                    generators: vec![],
                    phases: vec![PhaseDef {
                        id: "phase".to_string(),
                        invulnerable: false,
                        armored: false,
                        helper_gates_damage: false,
                        selector: AttackSelectorDef {
                            fire_patterns: vec!["pattern".to_string()],
                            ice_patterns: vec!["pattern".to_string()],
                            fire_nuke_patterns: vec![],
                            ice_nuke_patterns: vec![],
                            neutral_patterns: vec![],
                        },
                        enter_commands: vec![],
                        transitions: vec![TransitionDef {
                            condition: TransitionConditionDef::HpBelowRatio(0.0),
                            to_phase: "phase".to_string(),
                        }],
                    }],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        Runtime::new(&compiled, "encounter".to_string()).unwrap()
    }

    #[test]
    fn ttl_cleanup_removes_bullets() {
        let mut runtime = test_runtime();
        let mut rng = Rng64::new(7);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert!(runtime.boss.enemy_bullets.len() > 0);
        for _ in 0..20 {
            runtime.step_frame(InputSnapshot::default(), &mut rng);
        }
        assert_eq!(runtime.boss.enemy_bullets.len(), 0);
    }

    #[test]
    fn deterministic_checksum_matches() {
        let mut a = test_runtime();
        let mut b = test_runtime();
        let mut rng_a = Rng64::new(42);
        let mut rng_b = Rng64::new(42);
        for _ in 0..10 {
            a.step_frame(InputSnapshot::default(), &mut rng_a);
            b.step_frame(InputSnapshot::default(), &mut rng_b);
        }
        assert_eq!(a.checksum(), b.checksum());
    }

    #[test]
    fn tile_collision_keeps_player_out_of_wall() {
        let runtime = test_runtime();
        let mut x = 0.4;
        let mut y = 0.4;
        runtime::collision::resolve_actor_vs_tiles(
            &runtime.arena.arena,
            &mut x,
            &mut y,
            constants::PLAYER_RADIUS,
        );
        assert!(x >= constants::PLAYER_RADIUS);
        assert!(y >= constants::PLAYER_RADIUS);
    }

    #[test]
    fn bullet_seam_query_does_not_hit_adjacent_wall() {
        let compiled = schema::compile_author_root(AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "#####".to_string(),
                    "#...#".to_string(),
                    "#...#".to_string(),
                    "#.###".to_string(),
                    "#####".to_string(),
                ],
                player_spawn: Vec2Def { x: 1.5, y: 1.5 },
                boss_spawn: Vec2Def { x: 2.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 5.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![PhaseDef {
                        id: "phase".to_string(),
                        invulnerable: false,
                        armored: false,
                        helper_gates_damage: false,
                        selector: AttackSelectorDef {
                            fire_patterns: vec![],
                            ice_patterns: vec![],
                            fire_nuke_patterns: vec![],
                            ice_nuke_patterns: vec![],
                            neutral_patterns: vec!["placeholder".to_string()],
                        },
                        enter_commands: vec![],
                        transitions: vec![],
                    }],
                },
            }],
        });
        let arena = &compiled.arenas[0];
        assert!(!runtime::collision::bullet_hits_wall(2.0, 2.0, 0.18, arena));
    }

    #[test]
    fn phase_entry_keeps_helpers_without_explicit_despawn() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![
                        PhaseDef {
                            id: "phase_a".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SpawnHelper {
                                helper_id: "bird".to_string(),
                                sprite: 1,
                                hp: 50.0,
                                radius: 0.4,
                                motion: HelperMotion::Hover,
                                orbit_radius: 0.0,
                                orbit_speed_deg: 0.0,
                                bullet_pattern: None,
                                color_rgba: [1.0, 1.0, 1.0, 1.0],
                            }],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::TimerAtLeast(1),
                                to_phase: "phase_b".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "phase_b".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "phase_b");
        assert_eq!(runtime.boss.helpers.len(), 1);
        assert_eq!(runtime.boss.helpers.ids[0], "bird");
    }

    #[test]
    fn spawn_object_replaces_existing_id() {
        let mut runtime = test_runtime();
        runtime.execute_command(CommandDef::SpawnObject {
            object_id: "gate".to_string(),
            sprite: 1,
            hp: 10.0,
            radius: 0.4,
            motion: ObjectMotion::Fixed,
            anchor: Vec2Def { x: 5.0, y: 2.0 },
            orbit_radius: 0.0,
            orbit_speed_deg: 0.0,
            bullet_pattern: None,
            color_rgba: [1.0, 0.0, 0.0, 1.0],
        });
        runtime.execute_command(CommandDef::SpawnObject {
            object_id: "gate".to_string(),
            sprite: 2,
            hp: 20.0,
            radius: 0.5,
            motion: ObjectMotion::Fixed,
            anchor: Vec2Def { x: 6.0, y: 2.0 },
            orbit_radius: 0.0,
            orbit_speed_deg: 0.0,
            bullet_pattern: None,
            color_rgba: [0.0, 0.0, 1.0, 1.0],
        });
        assert_eq!(runtime.boss.objects.len(), 1);
        assert_eq!(runtime.boss.objects.ids[0], "gate");
        assert_eq!(runtime.boss.objects.sprite[0], 2);
        assert_eq!(runtime.boss.objects.max_hp[0], 20.0);
    }

    #[test]
    fn objects_dead_transition_advances_phase() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 0,
                    generators: vec![],
                    phases: vec![
                        PhaseDef {
                            id: "gate".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: true,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SpawnObject {
                                object_id: "seal".to_string(),
                                sprite: 1,
                                hp: 10.0,
                                radius: 0.4,
                                motion: ObjectMotion::Fixed,
                                anchor: Vec2Def { x: 5.0, y: 2.0 },
                                orbit_radius: 0.0,
                                orbit_speed_deg: 0.0,
                                bullet_pattern: None,
                                color_rgba: [1.0, 0.0, 0.0, 1.0],
                            }],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::ObjectsDead,
                                to_phase: "next".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "next".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        runtime.boss.objects.hp[0] = 0.0;
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "next");
    }

    #[test]
    fn sealing_generator_advances_phase() {
        let root = AuthorRoot {
            arenas: vec![AuthorArenaDef {
                id: "arena".to_string(),
                tile_size: 1.0,
                rows: vec![
                    "##########".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "#........#".to_string(),
                    "##########".to_string(),
                ],
                player_spawn: Vec2Def { x: 2.0, y: 2.0 },
                boss_spawn: Vec2Def { x: 6.0, y: 2.0 },
                camera_bounds: RectDef {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 10.0,
                    max_y: 5.0,
                },
            }],
            bullet_archetypes: vec![],
            patterns: vec![PatternDef {
                id: "idle".to_string(),
                family: PatternFamily::Neutral,
                nuke: false,
                duration_frames: 10,
                interruption_damage: None,
                emitters: vec![],
                commands: vec![],
            }],
            encounters: vec![EncounterDef {
                id: "encounter".to_string(),
                arena_id: "arena".to_string(),
                boss: BossDef {
                    hp: 100.0,
                    radius: 0.8,
                    generator_count: 1,
                    generators: vec![GeneratorDef {
                        id: "gen".to_string(),
                        anchor: Vec2Def { x: 4.0, y: 2.0 },
                        hp: 10.0,
                        radius: 0.4,
                    }],
                    phases: vec![
                        PhaseDef {
                            id: "seal".to_string(),
                            invulnerable: true,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SetGeneratorsVulnerable(true)],
                            transitions: vec![TransitionDef {
                                condition: TransitionConditionDef::SealedGeneratorsAtLeast(1),
                                to_phase: "next".to_string(),
                            }],
                        },
                        PhaseDef {
                            id: "next".to_string(),
                            invulnerable: false,
                            armored: false,
                            helper_gates_damage: false,
                            selector: AttackSelectorDef {
                                fire_patterns: vec![],
                                ice_patterns: vec![],
                                fire_nuke_patterns: vec![],
                                ice_nuke_patterns: vec![],
                                neutral_patterns: vec!["idle".to_string()],
                            },
                            enter_commands: vec![CommandDef::SetGeneratorsVulnerable(false)],
                            transitions: vec![],
                        },
                    ],
                },
            }],
        };
        let compiled = schema::compile_author_root(root);
        let mut runtime = Runtime::new(&compiled, "encounter".to_string()).unwrap();
        runtime.boss.generators.hp[0] = 0.0;
        runtime.seal_generator(0);
        let mut rng = Rng64::new(1);
        runtime.step_frame(InputSnapshot::default(), &mut rng);
        assert_eq!(runtime.current_phase().id, "next");
        assert!(runtime.boss.generators.sealed[0]);
        assert!(!runtime.boss.generators.vulnerable[0]);
    }
}
