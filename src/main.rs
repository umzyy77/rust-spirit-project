mod config;
mod controls;
mod direction;
mod position;
mod spawn;
mod square;

use std::collections::HashMap;
use std::env;
use std::process;

use piston_window::*;
use piston_window::graphics::{clear, rectangle};

use direction::Direction;
use square::{Square, SIZE};

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

    let mut squares = spawn::build_squares(&groups, WIDTH, HEIGHT);
    let bindings = controls::build_bindings(&groups);

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            handle_key(key, true, &bindings, &mut squares);
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            handle_key(key, false, &bindings, &mut squares);
        }

        if let Some(update) = event.update_args() {
            let positions: Vec<_> = squares.iter().map(|s| s.position()).collect();
            for square in squares.iter_mut() {
                square.update(update.dt, (WIDTH, HEIGHT), &positions);
            }
        }

        window.draw_2d(&event, |context, graphics, _device| {
            clear([0.1, 0.1, 0.12, 1.0], graphics);
            for square in &squares {
                rectangle(
                    square.color,
                    [square.position().x(), square.position().y(), SIZE, SIZE],
                    context.transform,
                    graphics,
                );
            }
        });
    }
}

/// Répercute un appui (`pressed = true`) ou un relâchement (`pressed =
/// false`) de `key` sur le carré concerné, via `controls::resolve`.
fn handle_key(
    key: Key,
    pressed: bool,
    bindings: &HashMap<Key, (usize, Direction)>,
    squares: &mut [Square],
) {
    if let Some((index, direction)) = controls::resolve(key, bindings) {
        squares[index].set_moving(direction, pressed);
    }
}
