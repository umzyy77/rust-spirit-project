use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;

use crate::config::ControlGroup;
use crate::direction::Direction;

const SPEED: f32 = 250.0;

/// Les touches d'une entité (une par direction), interrogées directement
/// chaque frame.
#[derive(Component)]
pub struct Controls(pub ControlGroup);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement);
    }
}

fn movement(keyboard: Res<ButtonInput<Key>>, mut query: Query<(&Controls, &mut Velocity)>) {
    for (controls, mut velocity) in &mut query {
        velocity.linear = controls
            .0
            .iter()
            .zip(Direction::ALL)
            // `Key` plutôt que `KeyCode` : respecte la disposition de clavier
            // de l'utilisateur au lieu de l'emplacement physique de la touche.
            .filter(|(key, _)| keyboard.pressed(Key::Character(key.to_ascii_lowercase().to_string().into())))
            .fold(Vec2::ZERO, |v, (_, direction)| v + unit_vector(direction) * SPEED);
    }
}

/// Vecteur unitaire pointant vers `direction` (convention Bevy : Y vers le haut).
fn unit_vector(direction: Direction) -> Vec2 {
    match direction {
        Direction::Up => Vec2::Y,
        Direction::Left => Vec2::NEG_X,
        Direction::Down => Vec2::NEG_Y,
        Direction::Right => Vec2::X,
    }
}
