use bevy::prelude::*;

use crate::combat::{spawn_bullet, spawn_muzzle};
use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct Player {
    pub facing: f32,
    pub floor: usize,
    pub ducking: bool,
    pub vy: f32,
    pub airborne: bool,
    pub fire_cd: f32,
    pub fire_anim: f32,
    pub invuln: f32,
    pub walk_phase: f32,
    pub riding: Option<usize>,
    pub alive: bool,
    pub dead_timer: f32,
}

impl Player {
    pub fn spawn_transform() -> Transform {
        Transform::from_xyz(SPAWN_X, FLOOR_Y[1], 1.0)
    }

    pub fn height(&self) -> f32 {
        if self.ducking && !self.airborne && self.riding.is_none() {
            10.0
        } else {
            PLAYER_H
        }
    }

    pub fn width(&self) -> f32 {
        if self.ducking && !self.airborne && self.riding.is_none() {
            12.0
        } else {
            PLAYER_W
        }
    }

    pub fn chest_y(&self, feet_y: f32) -> f32 {
        if self.ducking && !self.airborne {
            feet_y + 6.0
        } else {
            feet_y + 13.0
        }
    }
}

pub fn shaft_near(x: f32, half: f32) -> Option<usize> {
    SHAFT_X.iter().position(|sx| (x - sx).abs() <= half)
}

#[allow(clippy::too_many_arguments)]
pub fn update_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    art: Res<Art>,
    mut run: ResMut<RunState>,
    mut elevators: ResMut<Elevators>,
    mut fx: ResMut<Fx>,
    mut rng: ResMut<Rng>,
    mut commands: Commands,
    mut sfx: MessageWriter<PlaySfx>,
    mut query: Query<(Entity, &mut Player, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok((entity, mut player, mut transform, mut sprite)) = query.single_mut() else {
        return;
    };

    player.fire_cd = (player.fire_cd - dt).max(0.0);
    player.fire_anim = (player.fire_anim - dt).max(0.0);
    player.invuln = (player.invuln - dt).max(0.0);

    if !player.alive {
        set_frame(&mut sprite, 6, player.facing);
        sprite.color = Color::WHITE;
        return;
    }

    let left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    let up_pressed = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW);
    let down_pressed = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS);
    let down_held = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    let fire = keys.pressed(KeyCode::KeyJ) || keys.pressed(KeyCode::KeyX);
    let jump = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyK);

    let mut moving = false;

    if let Some(shaft) = player.riding {
        // Riding a car: position follows it, up/down picks the next floor,
        // left/right steps off once the car is docked.
        let car = &mut elevators.0[shaft];
        transform.translation.x = SHAFT_X[shaft];
        transform.translation.y = car.bottom + CAR_PLATE_OFFSET + 1.0;
        player.ducking = false;
        player.airborne = false;
        if let Some(floor) = car.docked_floor() {
            player.floor = floor;
            if left != right {
                let dir = if right { 1.0 } else { -1.0 };
                let floor_y = FLOOR_Y[floor];
                let x = clamp_x(SHAFT_X[shaft] + dir * 18.0, PLAYER_W * 0.5);
                transform.translation.x = x;
                transform.translation.y = floor_y;
                player.floor = floor;
                player.riding = None;
                car.riders = car.riders.saturating_sub(1);
                player.facing = dir;
            } else if up_pressed && floor > 1 {
                car.request(floor - 1);
                sfx.write(PlaySfx(SfxKind::Call));
            } else if down_pressed && floor + 1 < FLOOR_COUNT {
                car.request(floor + 1);
                sfx.write(PlaySfx(SfxKind::Call));
            }
        }
        set_frame(&mut sprite, 5, player.facing);
    } else {
        // On foot.
        let floor_y = FLOOR_Y[player.floor];
        if player.airborne {
            transform.translation.y += player.vy * dt;
            player.vy -= GRAVITY * dt;
            if transform.translation.y <= floor_y {
                transform.translation.y = floor_y;
                player.vy = 0.0;
                player.airborne = false;
            }
        } else {
            transform.translation.y = floor_y;
        }

        player.ducking = down_held && !player.airborne;
        if left != right {
            let dir = if right { 1.0 } else { -1.0 };
            let speed = if player.ducking {
                PLAYER_SPEED * 0.42
            } else {
                PLAYER_SPEED
            };
            transform.translation.x += dir * speed * dt;
            player.facing = dir;
            moving = true;
            player.walk_phase += dt * 9.0;
        } else {
            player.walk_phase = 0.0;
        }
        transform.translation.x = clamp_x(transform.translation.x, PLAYER_W * 0.5);

        if jump && !player.airborne && !player.ducking {
            player.vy = PLAYER_JUMP;
            player.airborne = true;
            player.ducking = false;
        }

        if fire && player.fire_cd <= 0.0 {
            let chest = player.chest_y(transform.translation.y);
            let muzzle_x = transform.translation.x + player.facing * 9.0;
            spawn_bullet(
                &mut commands,
                &art,
                Vec2::new(muzzle_x, chest),
                player.facing,
                false,
            );
            spawn_muzzle(&mut commands, muzzle_x, chest, player.facing, false);
            player.fire_cd = 0.19;
            player.fire_anim = 0.11;
            fx.shake = fx.shake.max(0.7);
            sfx.write(PlaySfx(SfxKind::Shoot));
        }

        if up_pressed && let Some(shaft) = shaft_near(transform.translation.x, SHAFT_HALF) {
            let car = &mut elevators.0[shaft];
            if car.docked_floor() == Some(player.floor) {
                player.riding = Some(shaft);
                player.floor = car.current_floor();
                car.riders += 1;
                transform.translation.x = SHAFT_X[shaft];
                transform.translation.y = car.bottom + CAR_PLATE_OFFSET + 1.0;
                player.facing = 1.0;
                sfx.write(PlaySfx(SfxKind::Ding));
            } else {
                car.request(player.floor);
                sfx.write(PlaySfx(SfxKind::Call));
                run.show_banner("ELEVATOR CALLED", 1.1);
            }
        }

        if player.ducking {
            set_frame(&mut sprite, 4, player.facing);
        } else if player.fire_anim > 0.0 {
            set_frame(&mut sprite, 3, player.facing);
        } else if moving {
            let frame = 1 + ((player.walk_phase as i32) % 2) as usize;
            set_frame(&mut sprite, frame, player.facing);
        } else {
            set_frame(&mut sprite, 0, player.facing);
        }
    }

    sprite.flip_x = player.facing < 0.0;
    sprite.color = if player.invuln > 0.0 && (player.invuln * 18.0) as i32 % 2 == 0 {
        Color::srgba(1.0, 1.0, 1.0, 0.35)
    } else {
        Color::WHITE
    };

    // Keep riders from being pushed through the building walls.
    transform.translation.x = clamp_x(transform.translation.x, PLAYER_W * 0.5);
    transform.translation.z = 1.0;

    // If a rider somehow died, release the car.
    if !player.alive
        && let Some(shaft) = player.riding.take()
    {
        elevators.0[shaft].riders = elevators.0[shaft].riders.saturating_sub(1);
    }
    let _ = (entity, &mut rng);
}

fn set_frame(sprite: &mut Sprite, index: usize, _facing: f32) {
    if let Some(atlas) = &mut sprite.texture_atlas
        && atlas.index != index
    {
        atlas.index = index;
    }
}

pub fn respawn_player(
    mut query: Query<(&mut Player, &mut Transform, &mut Sprite)>,
    mut elevators: ResMut<Elevators>,
) {
    let Ok((mut player, mut transform, mut sprite)) = query.single_mut() else {
        return;
    };
    player.alive = true;
    player.floor = 1;
    player.ducking = false;
    player.airborne = false;
    player.vy = 0.0;
    player.invuln = 2.4;
    player.dead_timer = 0.0;
    player.fire_cd = 0.0;
    player.riding = None;
    player.facing = 1.0;
    transform.translation = Vec3::new(SPAWN_X, FLOOR_Y[1], 1.0);
    sprite.color = Color::WHITE;
    set_frame(&mut sprite, 0, 1.0);
    for car in &mut elevators.0 {
        car.riders = 0;
    }
}

pub fn snap_pixels(mut query: Query<&mut Transform, With<PixelPerfect>>) {
    for mut transform in &mut query {
        transform.translation.x = transform.translation.x.round();
        transform.translation.y = transform.translation.y.round();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaft_proximity() {
        assert_eq!(shaft_near(-42.0, 10.0), Some(0));
        assert_eq!(shaft_near(60.0, 10.0), Some(1));
        assert_eq!(shaft_near(0.0, 10.0), None);
    }

    #[test]
    fn duck_hitbox_is_short() {
        let p = Player {
            facing: 1.0,
            floor: 0,
            ducking: true,
            vy: 0.0,
            airborne: false,
            fire_cd: 0.0,
            fire_anim: 0.0,
            invuln: 0.0,
            walk_phase: 0.0,
            riding: None,
            alive: true,
            dead_timer: 0.0,
        };
        assert_eq!(p.height(), 10.0);
        assert_eq!(p.chest_y(0.0), 6.0);
    }
}
