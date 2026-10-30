use bevy::prelude::*;

use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct ElevatorCar {
    pub shaft: usize,
}

pub fn update_elevators(
    time: Res<Time>,
    mut elevators: ResMut<Elevators>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let dt = time.delta_secs().min(0.05);
    for car in &mut elevators.0 {
        if car.update(dt).is_some() {
            sfx.write(PlaySfx(SfxKind::Ding));
        }
    }
}

pub fn sync_elevators(
    elevators: Res<Elevators>,
    art: Res<Art>,
    mut query: Query<(&ElevatorCar, &mut Transform, &mut Sprite)>,
) {
    for (car, mut transform, mut sprite) in &mut query {
        let state = &elevators.0[car.shaft];
        transform.translation.x = SHAFT_X[car.shaft];
        transform.translation.y = state.bottom;
        let wanted = if state.moving {
            art.elevator_move.clone()
        } else {
            art.elevator.clone()
        };
        if sprite.image != wanted {
            sprite.image = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::world::CarState;

    #[test]
    fn empty_car_stays_put() {
        let mut car = CarState::new(3);
        for _ in 0..120 {
            assert!(car.update(1.0 / 60.0).is_none());
        }
        assert_eq!(car.docked_floor(), Some(3));
    }
}
