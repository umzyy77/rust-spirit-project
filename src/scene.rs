use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{EntityConfig, MAX_ENTITIES};
use crate::movement::Controls;
use crate::shape::Shape;

pub const WIDTH: f32 = 800.0;
pub const HEIGHT: f32 = 800.0;

const HALF_SIZE: f32 = 25.0;
/// Écart gardé entre une entité et les murs à son spawn, pour ne jamais
/// démarrer exactement au contact (pénétration nulle mais tangente, qui
/// peut déclencher une impulsion de dépénétration parasite au 1er tick).
const SPAWN_MARGIN: f32 = 5.0;
/// Même masse pour toutes les formes : sinon Rapier la calcule depuis
/// l'aire du collider, et un carré domine un cercle de même taille.
const ENTITY_MASS: f32 = 1.0;
/// Murs invisibles, bien au-delà des bords visibles pour éviter tout trou
/// de collision aux coins.
const WALL_THICKNESS: f32 = 1000.0;

const COLORS: [Color; MAX_ENTITIES] = [
    Color::srgb(0.90, 0.25, 0.25), // rouge
    Color::srgb(0.25, 0.55, 0.90), // bleu
    Color::srgb(0.30, 0.80, 0.35), // vert
    Color::srgb(0.95, 0.75, 0.20), // jaune
    Color::srgb(0.70, 0.35, 0.85), // violet
    Color::srgb(0.95, 0.55, 0.20), // orange
];

#[derive(Resource)]
struct Configs(Vec<EntityConfig>);

pub struct ScenePlugin(pub Vec<EntityConfig>);

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Configs(self.0.clone())).add_systems(Startup, setup);
    }
}

/// `half` = demi-côté pour un carré, rayon pour un cercle.
fn build_shape(shape: Shape, meshes: &mut Assets<Mesh>, half: f32) -> (Handle<Mesh>, Collider) {
    match shape {
        Shape::Square => (meshes.add(Rectangle::new(half * 2.0, half * 2.0)), Collider::cuboid(half, half)),
        Shape::Circle => (meshes.add(Circle::new(half)), Collider::ball(half)),
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, configs: Res<Configs>) {
    commands.spawn((Camera2d, Transform::from_xyz(WIDTH / 2.0, HEIGHT / 2.0, 0.0)));

    for (x, y, half_width, half_height) in [
        (-WALL_THICKNESS, HEIGHT / 2.0, WALL_THICKNESS, HEIGHT),          // gauche
        (WIDTH + WALL_THICKNESS, HEIGHT / 2.0, WALL_THICKNESS, HEIGHT),   // droite
        (WIDTH / 2.0, -WALL_THICKNESS, WIDTH, WALL_THICKNESS),           // bas
        (WIDTH / 2.0, HEIGHT + WALL_THICKNESS, WIDTH, WALL_THICKNESS),   // haut
    ] {
        commands.spawn((
            Transform::from_xyz(x, y, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(half_width, half_height),
        ));
    }

    let entities = &configs.0;
    let count = entities.len();

    for (i, entity) in entities.iter().enumerate() {
        let x = if count == 1 {
            WIDTH / 2.0
        } else {
            let span = HALF_SIZE + SPAWN_MARGIN;
            let step = (WIDTH - span * 2.0) / (count - 1) as f32;
            span + i as f32 * step
        };

        let (mesh, collider) = build_shape(entity.shape, &mut meshes, HALF_SIZE);

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(COLORS[i % COLORS.len()])),
            Transform::from_xyz(x, HEIGHT / 2.0, 0.0),
            RigidBody::Dynamic,
            collider,
            ColliderMassProperties::Mass(ENTITY_MASS),
            Velocity::zero(),
            // Vue du dessus : pas de gravité ni de rotation au contact.
            GravityScale(0.0),
            LockedAxes::ROTATION_LOCKED_Z,
            Controls::new(entity.keys),
        ));
    }
}
