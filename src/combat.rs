use bevy::prelude::*;

use crate::enemy::Enemy;
use crate::player::Player;
use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct Bullet {
    pub vx: f32,
    pub enemy: bool,
    pub life: f32,
}

#[derive(Component)]
pub struct Particle {
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub gravity: f32,
}

#[derive(Component)]
pub struct Door {
    pub floor: usize,
    pub x: f32,
    pub secret: bool,
    pub opened: bool,
    pub close_timer: f32,
}

#[derive(Component)]
pub struct ExitSign;

pub fn spawn_bullet(commands: &mut Commands, art: &Art, pos: Vec2, facing: f32, enemy: bool) {
    commands.spawn((
        Bullet {
            vx: facing * if enemy { 132.0 } else { 255.0 },
            enemy,
            life: 1.6,
        },
        Sprite {
            image: if enemy {
                art.bullet_enemy.clone()
            } else {
                art.bullet_player.clone()
            },
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 2.0),
        PixelPerfect,
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));
}

pub fn spawn_muzzle(commands: &mut Commands, x: f32, y: f32, facing: f32, enemy: bool) {
    let color = if enemy {
        Color::srgb(1.0, 0.35, 0.25)
    } else {
        Color::srgb(1.0, 0.86, 0.35)
    };
    for i in 0..4 {
        let spread = (i as f32 - 1.5) * 7.0;
        spawn_particle(
            commands,
            Vec2::new(x + facing * 2.0, y + spread * 0.25),
            Vec2::new(facing * (18.0 + i as f32 * 9.0), spread),
            color,
            2.0,
            0.16,
        );
    }
}

pub fn spawn_burst(commands: &mut Commands, pos: Vec2, color: Color, count: usize, power: f32) {
    for i in 0..count {
        let a = i as f32 * 2.39996;
        let speed = power * (0.55 + (i % 4) as f32 * 0.15);
        spawn_particle(
            commands,
            pos,
            Vec2::new(a.cos() * speed, a.sin() * speed + 12.0),
            color,
            if i % 3 == 0 { 2.0 } else { 1.0 },
            0.45,
        );
    }
}

fn spawn_particle(
    commands: &mut Commands,
    pos: Vec2,
    vel: Vec2,
    color: Color,
    size: f32,
    life: f32,
) {
    commands.spawn((
        Particle {
            vx: vel.x,
            vy: vel.y,
            life,
            max_life: life,
            gravity: 90.0,
        },
        Sprite::from_color(color, Vec2::splat(size)),
        Transform::from_xyz(pos.x, pos.y, 3.0),
        PixelPerfect,
        LevelEntity,
        DespawnOnExit(AppState::InGame),
    ));
}

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.life -= dt;
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += particle.vx * dt;
        transform.translation.y += particle.vy * dt;
        particle.vy -= particle.gravity * dt;
        let t = (particle.life / particle.max_life).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(t);
    }
}

pub fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut bullet, mut transform) in &mut query {
        transform.translation.x += bullet.vx * dt;
        bullet.life -= dt;
        if bullet.life <= 0.0
            || transform.translation.x < BUILDING_LEFT - 12.0
            || transform.translation.x > BUILDING_RIGHT + 12.0
        {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn collide_bullets(
    mut commands: Commands,
    mut run: ResMut<RunState>,
    mut fx: ResMut<Fx>,
    mut elevators: ResMut<Elevators>,
    mut sfx: MessageWriter<PlaySfx>,
    bullets: Query<(Entity, &Bullet, &Transform)>,
    mut enemies: Query<(&mut Enemy, &Transform), Without<Player>>,
    mut players: Query<(&mut Player, &Transform, &mut Sprite), Without<Enemy>>,
) {
    let Ok((mut player, player_tf, mut player_sprite)) = players.single_mut() else {
        return;
    };
    let px = player_tf.translation.x;
    let py = player_tf.translation.y;

    for (bullet_entity, bullet, bullet_tf) in &bullets {
        let bx = bullet_tf.translation.x;
        let by = bullet_tf.translation.y;

        if bullet.enemy {
            if player.alive && player.invuln <= 0.0 {
                let w = player.width();
                let h = player.height();
                if feet_overlap(px, py, w, h, bx, by - 1.0, 5.0, 2.0) {
                    player.alive = false;
                    player.dead_timer = 1.35;
                    if let Some(atlas) = &mut player_sprite.texture_atlas {
                        atlas.index = 6;
                    }
                    player_sprite.color = Color::WHITE;
                    if let Some(shaft) = player.riding.take() {
                        elevators.0[shaft].riders = elevators.0[shaft].riders.saturating_sub(1);
                    }
                    run.lives = run.lives.saturating_sub(1);
                    run.phase = Phase::Dying;
                    run.phase_timer = 1.35;
                    run.show_banner("AGENT DOWN", 1.2);
                    fx.shake = 7.0;
                    fx.flash = 0.35;
                    sfx.write(PlaySfx(SfxKind::Death));
                    spawn_burst(
                        &mut commands,
                        Vec2::new(px, py + 10.0),
                        Color::srgb(1.0, 0.35, 0.3),
                        10,
                        55.0,
                    );
                    commands.entity(bullet_entity).despawn();
                    continue;
                }
            }
        } else {
            for (mut enemy, enemy_tf) in &mut enemies {
                if !enemy.alive {
                    continue;
                }
                let ex = enemy_tf.translation.x;
                let ey = enemy_tf.translation.y;
                if feet_overlap(ex, ey, ENEMY_W, ENEMY_H, bx, by - 1.0, 5.0, 2.0) {
                    enemy.alive = false;
                    enemy.dead_timer = 0.75;
                    if let Some(shaft) = enemy.riding.take() {
                        elevators.0[shaft].riders = elevators.0[shaft].riders.saturating_sub(1);
                    }
                    run.add_score(200);
                    fx.shake = fx.shake.max(2.2);
                    sfx.write(PlaySfx(SfxKind::Hit));
                    spawn_burst(
                        &mut commands,
                        Vec2::new(ex, ey + 12.0),
                        Color::srgb(1.0, 0.55, 0.3),
                        12,
                        62.0,
                    );
                    commands.entity(bullet_entity).despawn();
                    break;
                }
            }
        }
    }
}

pub fn update_doors(
    time: Res<Time>,
    art: Res<Art>,
    mut run: ResMut<RunState>,
    mut commands: Commands,
    mut sfx: MessageWriter<PlaySfx>,
    mut doors: Query<(&mut Door, &mut Sprite)>,
    player_query: Query<(&Player, &Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok((player, player_tf)) = player_query.single() else {
        return;
    };
    for (mut door, mut sprite) in &mut doors {
        if !door.secret && door.opened {
            door.close_timer -= dt;
            if door.close_timer <= 0.0 {
                door.opened = false;
            }
        }

        if door.secret
            && !door.opened
            && player.alive
            && player.riding.is_none()
            && player.floor == door.floor
            && (player_tf.translation.x - door.x).abs() < 11.0
        {
            door.opened = true;
            run.docs_collected += 1;
            run.add_score(500);
            run.show_banner("TOP SECRET FILE RECOVERED", 1.8);
            sfx.write(PlaySfx(SfxKind::Pickup));
            spawn_burst(
                &mut commands,
                Vec2::new(door.x, FLOOR_Y[door.floor] + 10.0),
                Color::srgb(1.0, 0.88, 0.45),
                14,
                42.0,
            );
            if run.docs_collected >= run.docs_total {
                run.escape_ready = true;
                run.show_banner("ALL FILES SECURED - REACH THE GETAWAY CAR", 3.4);
                sfx.write(PlaySfx(SfxKind::LevelClear));
            }
        }

        let wanted = art.door_sheet(door.secret, door.opened);
        if sprite.image != wanted {
            sprite.image = wanted;
        }
    }
}

pub fn update_exit_sign(
    time: Res<Time>,
    run: Res<RunState>,
    mut query: Query<&mut Sprite, With<ExitSign>>,
) {
    for mut sprite in &mut query {
        if run.escape_ready {
            let pulse = 0.75 + (time.elapsed_secs() * 5.0).sin() * 0.25;
            sprite.color = Color::srgba(1.0, 1.0, 1.0, pulse);
        } else {
            sprite.color = Color::srgba(0.55, 0.6, 0.62, 0.5);
        }
    }
}
