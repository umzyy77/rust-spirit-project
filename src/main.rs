mod config;
mod controls;
mod direction;
mod entity;
mod gameloop;
mod position;
mod spawn;

use std::env;
use std::process;

use piston_window::{PistonWindow, WindowSettings};

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 800.0;

fn main() {
    let args: Vec<String> = env::args().collect();

    let groups = config::parse_args(&args[1..]).unwrap_or_else(|err| {
        eprintln!("Erreur de configuration : {err}");
        process::exit(1);
    });

    let mut window: PistonWindow = WindowSettings::new("Squares", [WIDTH as u32, HEIGHT as u32])
        .build()
        .unwrap();

    let mut entities = spawn::build_entities(&groups, WIDTH, HEIGHT);
    let bindings = controls::build_bindings(&groups);

    gameloop::run(&mut window, &mut entities, &bindings, (WIDTH, HEIGHT));
}
