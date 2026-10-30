use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::combat::{spawn_bullet, spawn_muzzle};
use crate::player::Player;
use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct Enemy {
    pub floor: usize,
    pub facing: f32,
    pub speed: f32,
    pub shoot_cd: f32,
    pub aim: f32,
    pub reaction: f32,
    pub fire_anim: f32,
    pub walk_phase: f32,
    pub alive: bool,
    pub dead_timer: f32,
    pub riding: Option<usize>,
    pub variant: u8,
}

pub fn spawn_enemy(
    commands: &mut Commands,
    art: &Art,
    x: f32,
    floor: usize,
    speed: f32,
    accuracy: f32,
    variant: u8,
) -> Entity {
    let transform = Transform::from_xyz(x, FLOOR_Y[floor], 1.0);
    commands
        .spawn((
            Enemy {
                floor,
                facing: -1.0,
                speed,
                shoot_cd: 0.8,
                aim: 0.0,
                reaction: 0.9 - accuracy * 0.35,
                fire_anim: 0.0,
                walk_phase: 0.0,
                alive: true,
                dead_timer: 0.0,
                riding: None,
                variant,
            },
            Sprite {
                image: art.enemy_sheet(variant),
                texture_atlas: Some(TextureAtlas {
                    layout: art.enemy_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            transform,
            PixelPerfect,
            LevelEntity,
            DespawnOnExit(AppState::InGame),
        ))
        .id()
}

#[allow(clippy::too_many_arguments)]
pub fn update_enemies(
    time: Res<Time>,
    art: Res<Art>,
    run: Res<RunState>,
    mut elevators: ResMut<Elevators>,
    mut rng: ResMut<Rng>,
    mut commands: Commands,
    mut sfx: MessageWriter<PlaySfx>,
    player_query: Query<(&Player, &Transform), Without<Enemy>>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform, &mut Sprite), Without<Player>>,
) {
    if run.phase != Phase::Normal {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let Ok((player, player_tf)) = player_query.single() else {
        return;
    };
    if !player.alive {
        return;
    }
    let player_x = player_tf.translation.x;
    let player_floor = player.floor;

    for (entity, mut enemy, mut transform, mut sprite) in &mut enemies {
        enemy.fire_anim = (enemy.fire_anim - dt).max(0.0);
        enemy.shoot_cd = (enemy.shoot_cd - dt).max(0.0);

        if !enemy.alive {
            enemy.dead_timer -= dt;
            set_enemy_frame(&mut sprite, 4);
            if enemy.dead_timer <= 0.0 {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if let Some(shaft) = enemy.riding {
            let car = &mut elevators.0[shaft];
            transform.translation.x = SHAFT_X[shaft];
            transform.translation.y = car.bottom + CAR_PLATE_OFFSET + 1.0;
            enemy.facing = if player_x >= SHAFT_X[shaft] {
                1.0
            } else {
                -1.0
            };
            if let Some(floor) = car.docked_floor() {
                enemy.floor = floor;
                if floor == player_floor {
                    let dir = if player_x >= SHAFT_X[shaft] {
                        1.0
                    } else {
                        -1.0
                    };
                    enemy.riding = None;
                    car.riders = car.riders.saturating_sub(1);
                    transform.translation.x = clamp_x(SHAFT_X[shaft] + dir * 17.0, ENEMY_W * 0.5);
                    transform.translation.y = FLOOR_Y[floor];
                    enemy.floor = floor;
                } else if !car.moving {
                    car.request(player_floor);
                }
            }
            sprite.flip_x = enemy.facing < 0.0;
            set_enemy_frame(&mut sprite, 0);
            transform.translation.z = 1.0;
            continue;
        }

        let floor_y = FLOOR_Y[enemy.floor];
        transform.translation.y = floor_y;
        let mut walking = false;

        if enemy.floor == player_floor {
            let dx = player_x - transform.translation.x;
            let adx = dx.abs();
            if adx > 0.5 {
                enemy.facing = dx.signum();
            }
            if adx > 58.0 {
                transform.translation.x += enemy.facing * enemy.speed * dt;
                enemy.walk_phase += dt * 9.0;
                walking = true;
            }
            if adx < 172.0 {
                enemy.aim += dt;
                if enemy.aim >= enemy.reaction && enemy.shoot_cd <= 0.0 {
                    let muzzle =
                        Vec2::new(transform.translation.x + enemy.facing * 9.0, floor_y + 14.0);
                    spawn_bullet(&mut commands, &art, muzzle, enemy.facing, true);
                    spawn_muzzle(&mut commands, muzzle.x, muzzle.y, enemy.facing, true);
                    enemy.shoot_cd = 1.05 + rng.range(0.0, 0.85);
                    enemy.aim = 0.0;
                    enemy.fire_anim = 0.13;
                    sfx.write(PlaySfx(SfxKind::EnemyShoot));
                }
            } else {
                enemy.aim = 0.0;
            }
        } else {
            // Walk to a shaft, call the car, and ride toward the player.
            let shaft = if (transform.translation.x - SHAFT_X[0]).abs()
                < (transform.translation.x - SHAFT_X[1]).abs()
            {
                0
            } else {
                1
            };
            let sx = SHAFT_X[shaft];
            if (transform.translation.x - sx).abs() > 8.0 {
                let dir = (sx - transform.translation.x).signum();
                enemy.facing = dir;
                transform.translation.x += dir * enemy.speed * dt;
                enemy.walk_phase += dt * 9.0;
                walking = true;
            } else {
                transform.translation.x = sx;
                enemy.facing = if player_x >= sx { 1.0 } else { -1.0 };
                let car = &mut elevators.0[shaft];
                if car.docked_floor() == Some(enemy.floor) {
                    enemy.riding = Some(shaft);
                    car.riders += 1;
                    car.request(player_floor);
                } else if !car.moving && car.riders == 0 {
                    car.request(enemy.floor);
                }
            }
        }

        transform.translation.x = clamp_x(transform.translation.x, ENEMY_W * 0.5);
        transform.translation.z = 1.0;
        sprite.flip_x = enemy.facing < 0.0;
        if enemy.fire_anim > 0.0 {
            set_enemy_frame(&mut sprite, 3);
        } else if walking {
            let frame = 1 + ((enemy.walk_phase as i32) % 2) as usize;
            set_enemy_frame(&mut sprite, frame);
        } else {
            set_enemy_frame(&mut sprite, 0);
        }
    }
}

fn set_enemy_frame(sprite: &mut Sprite, index: usize) {
    if let Some(atlas) = &mut sprite.texture_atlas
        && atlas.index != index
    {
        atlas.index = index;
    }
}
