mod config;
mod direction;
mod movement;
mod scene;
mod shape;

use std::env;
use std::process;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use movement::MovementPlugin;
use scene::{HEIGHT, ScenePlugin, WIDTH};

fn main() {
    let args: Vec<String> = env::args().collect();
    let configs = config::parse_args(&args[1..]).unwrap_or_else(|err| {
        eprintln!("Erreur de configuration : {err}");
        process::exit(1);
    });

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH as u32, HEIGHT as u32).into(),
                    title: "Squares".into(),
                    ..default()
                }),
                ..default()
            }),
            // Rapier est calibré pour le mètre ; sans ça, des corps Dynamic
            // poussés en continu s'enfoncent l'un dans l'autre.
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0),
            ScenePlugin(configs),
            MovementPlugin,
        ))
        .run();
}
