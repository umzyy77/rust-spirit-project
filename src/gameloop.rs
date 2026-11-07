use std::collections::HashMap;

use piston_window::graphics::{clear, rectangle};
use piston_window::*;

use crate::controls;
use crate::direction::Direction;
use crate::entity::{Entity, SIZE};

/// Boucle principale : tant que la fenêtre est ouverte, répercute les
/// touches sur les entités concernées, avance la simulation (`update`), puis
/// redessine la scène.
pub fn run(
    window: &mut PistonWindow,
    entities: &mut [Entity],
    bindings: &HashMap<Key, (usize, Direction)>,
    bounds: (f64, f64),
) {
    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            handle_key(key, true, bindings, entities);
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            handle_key(key, false, bindings, entities);
        }

        if let Some(update) = event.update_args() {
            let positions: Vec<_> = entities.iter().map(|e| e.position()).collect();
            for entity in entities.iter_mut() {
                entity.update(update.dt, bounds, &positions);
            }
        }

        draw(window, &event, entities);
    }
}

/// Dessine la scène : fond uni, puis chaque entité à sa position actuelle.
fn draw(window: &mut PistonWindow, event: &Event, entities: &[Entity]) {
    window.draw_2d(event, |context, graphics, _device| {
        clear([0.1, 0.1, 0.12, 1.0], graphics);
        for entity in entities {
            let position = entity.position();
            rectangle(
                entity.color,
                [position.x(), position.y(), SIZE, SIZE],
                context.transform,
                graphics,
            );
        }
    });
}

/// Répercute un appui (`pressed = true`) ou un relâchement (`pressed =
/// false`) de `key` sur l'entité concernée, via `controls::resolve`.
fn handle_key(
    key: Key,
    pressed: bool,
    bindings: &HashMap<Key, (usize, Direction)>,
    entities: &mut [Entity],
) {
    if let Some((index, direction)) = controls::resolve(key, bindings) {
        entities[index].set_moving(direction, pressed);
    }
}
