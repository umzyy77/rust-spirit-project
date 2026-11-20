mod config;
mod controls;
mod direction;
mod drawable;
mod entity;
mod gjk;
mod gameloop;
mod interpolation;
mod network;
mod shapes;
mod spawn;

use std::env;
use std::process;

use piston_window::{PistonWindow, WindowSettings};

use network::Connection;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 800.0;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage : client <adresse:port> <4 touches> (ex: 127.0.0.1:7878 ZQSD)");
        process::exit(1);
    }

    let address = &args[1];
    let group = config::parse_args(&args[2]).unwrap_or_else(|err| {
        eprintln!("Erreur de configuration : {err}");
        process::exit(1);
    });

    let mut connection = Connection::connect(address).unwrap_or_else(|err| {
        eprintln!("Impossible de se connecter a {address} : {err}");
        process::exit(1);
    });

    let squares = connection.list().unwrap_or_else(|err| {
        eprintln!("Impossible de recuperer la liste des carres : {err}");
        process::exit(1);
    });

    let (mut entities, mut ids, mut interpolations) = spawn::build_networked_entities(connection.id(), &squares);
    let bindings = controls::build_bindings(&group);

    let mut window: PistonWindow = WindowSettings::new("Squares", [WIDTH as u32, HEIGHT as u32])
        .build()
        .unwrap();

    let (commands, events) = network::spawn(connection);

    gameloop::run(
        &mut window,
        &mut entities,
        &mut ids,
        &mut interpolations,
        &bindings,
        (WIDTH, HEIGHT),
        &commands,
        &events,
    );
}
