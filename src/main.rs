mod actors;
mod app;
mod config;
mod controls;
mod direction;
mod entity;
mod gameloop;
mod position;
mod spawn;

use std::env;
use std::process;

use app::App;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 800.0;

fn main() {
    let args: Vec<String> = env::args().collect();

    let configs = config::parse_args(&args[1..]).unwrap_or_else(|err| {
        eprintln!("Erreur de configuration : {err}");
        process::exit(1);
    });

    let app = App::bootstrap(&configs, WIDTH, HEIGHT).unwrap_or_else(|err| {
        eprintln!("Erreur d'initialisation : {err}");
        process::exit(1);
    });

    app.run();
}
