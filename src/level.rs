use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::combat::{Door, ExitSign};
use crate::elevator::ElevatorCar;
use crate::enemy::spawn_enemy;
use crate::player::Player;
use crate::state::*;
use crate::ui;
use crate::world::*;

#[derive(Resource)]
pub struct SpawnTimer {
    pub timer: f32,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self { timer: 1.5 }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn load_pending_level(
    mut commands: Commands,
    mut pending: ResMut<PendingLevel>,
    art: Res<Art>,
    mut run: ResMut<RunState>,
    mut elevators: ResMut<Elevators>,
    mut spawn_timer: ResMut<SpawnTimer>,
    old: Query<Entity, With<LevelEntity>>,
    hud: Query<Entity, With<HUDEntity>>,
) {
    let Some(index) = pending.0.take() else {
        return;
    };
    let index = index.min(LEVELS.len() - 1);
    let level = &LEVELS[index];

    for entity in &old {
        commands.entity(entity).despawn();
    }

    run.level = index;
    run.docs_total = docs_in(level) as u32;
    run.docs_collected = 0;
    run.escape_ready = false;
    run.phase = Phase::Normal;
    run.phase_timer = 0.0;
    let total = run.docs_total;
    run.show_banner(
        format!("FIND {} RED FILES - W CALLS THE ELEVATOR", total),
        3.4,
    );

    elevators.0 = [CarState::new(1), CarState::new(4)];
    spawn_timer.timer = 2.2;

    // Building backdrop.
    commands.spawn((
        Sprite {
            image: art.background(index),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -20.0),
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));

    // Getaway car parked at the bottom right curb.
    commands.spawn((
        Sprite {
            image: art.car.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(104.0, -118.0, -6.0),
        PixelPerfect,
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));

    // Exit sign above the extraction point.
    commands.spawn((
        Sprite {
            image: art.exit_sign.clone(),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(120.0, -90.0, -7.0),
        ExitSign,
        PixelPerfect,
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));

    // Doors.
    for spec in level.doors {
        commands.spawn((
            Door {
                floor: spec.floor,
                x: spec.x,
                secret: spec.secret,
                opened: false,
                close_timer: 0.0,
            },
            Sprite {
                image: art.door_sheet(spec.secret, false),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(spec.x, FLOOR_Y[spec.floor], -8.0),
            PixelPerfect,
            LevelEntity,
            DespawnOnExit(AppState::InGame),
        ));
    }

    // Elevator cars.
    for (shaft, &sx) in SHAFT_X.iter().enumerate().take(SHAFT_COUNT) {
        commands.spawn((
            ElevatorCar { shaft },
            Sprite {
                image: art.elevator.clone(),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(sx, elevators.0[shaft].bottom, -7.0),
            PixelPerfect,
            LevelEntity,
            DespawnOnExit(AppState::InGame),
        ));
    }

    // Player.
    commands.spawn((
        Player {
            facing: 1.0,
            floor: 1,
            ducking: false,
            vy: 0.0,
            airborne: false,
            fire_cd: 0.0,
            fire_anim: 0.0,
            invuln: 1.2,
            walk_phase: 0.0,
            riding: None,
            alive: true,
            dead_timer: 0.0,
        },
        Sprite {
            image: art.player.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: art.player_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Player::spawn_transform(),
        PixelPerfect,
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));

    if hud.is_empty() {
        ui::spawn_hud(&mut commands, &art);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_wave(
    time: Res<Time>,
    art: Res<Art>,
    run: Res<RunState>,
    mut rng: ResMut<Rng>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut commands: Commands,
    mut sfx: MessageWriter<PlaySfx>,
    player_query: Query<(&Player, &Transform)>,
    mut doors: Query<(Entity, &mut Door, &mut Sprite)>,
    enemies: Query<&crate::enemy::Enemy>,
) {
    if run.phase != Phase::Normal {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    spawn_timer.timer -= dt;
    if spawn_timer.timer > 0.0 {
        return;
    }
    let level = run.level_def();
    let alive = enemies.iter().filter(|e| e.alive).count();
    if alive >= level.max_enemies {
        spawn_timer.timer = 1.0;
        return;
    }

    let Ok((player, player_tf)) = player_query.single() else {
        return;
    };
    let mut candidates = Vec::new();
    for (entity, door, _) in doors.iter() {
        if door.secret || door.opened {
            continue;
        }
        if door.floor == player.floor && (door.x - player_tf.translation.x).abs() < 46.0 {
            continue;
        }
        candidates.push(entity);
    }
    if candidates.is_empty() {
        spawn_timer.timer = 0.5;
        return;
    }
    let pick = candidates[(rng.next_u32() as usize) % candidates.len()];
    let Ok((_, mut door, mut sprite)) = doors.get_mut(pick) else {
        return;
    };
    door.opened = true;
    door.close_timer = 0.9;
    sprite.image = art.door_gray_open.clone();
    let floor = door.floor;
    let x = clamp_x(
        door.x + if door.x > 0.0 { -13.0 } else { 13.0 },
        ENEMY_W * 0.5,
    );
    let variant = (rng.next_u32() % 2) as u8;
    spawn_enemy(
        &mut commands,
        &art,
        x,
        floor,
        level.enemy_speed * (0.9 + rng.range(0.0, 0.2)),
        level.enemy_accuracy,
        variant,
    );
    sfx.write(PlaySfx(SfxKind::Door));
    spawn_timer.timer = level.enemy_interval * (0.8 + rng.range(0.0, 0.45));
}
