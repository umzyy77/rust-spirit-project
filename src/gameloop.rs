use std::collections::HashMap;

use piston_window::graphics::clear;
use piston_window::*;

use crate::actors::Actor;
use crate::controls;
use crate::direction::Direction;

/// Boucle principale : tant que la fenêtre est ouverte, répercute les
/// touches sur les entités concernées, avance la simulation (`update`), puis
/// redessine la scène.
pub fn run(
    window: &mut PistonWindow,
    entities: &mut [Box<dyn Actor>],
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
            let snapshot: Vec<_> = entities
                .iter()
                .map(|e| (e.get_entity().position(), e.shape()))
                .collect();

            for (index, entity) in entities.iter_mut().enumerate() {
                let others: Vec<_> = snapshot
                    .iter()
                    .enumerate()
                    .filter(|&(other_index, _)| other_index != index)
                    .map(|(_, (position, shape))| (*position, shape.clone_dyn()))
                    .collect();
                entity.update(update.dt, bounds, &others);
            }
        }

        draw(window, &event, entities);
    }
}

/// Dessine la scène : fond uni, puis chaque entité se dessine elle-même.
fn draw(window: &mut PistonWindow, event: &Event, entities: &[Box<dyn Actor>]) {
    window.draw_2d(event, |context, graphics, _device| {
        clear([0.1, 0.1, 0.12, 1.0], graphics);
        for entity in entities {
            entity.draw(context.transform, graphics);
        }
    });
}

/// Répercute un appui (`pressed = true`) ou un relâchement (`pressed =
/// false`) de `key` sur l'entité concernée, via `controls::resolve`.
fn handle_key(
    key: Key,
    pressed: bool,
    bindings: &HashMap<Key, (usize, Direction)>,
    entities: &mut [Box<dyn Actor>],
) {
    if let Some((index, direction)) = controls::resolve(key, bindings) {
        entities[index]
            .get_entity_mut()
            .set_moving(direction, pressed);
    }
}
