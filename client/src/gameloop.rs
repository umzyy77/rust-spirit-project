use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use piston_window::graphics::clear;
use piston_window::*;

use crate::controls;
use crate::direction::Direction;
use crate::drawable::Drawable;
use crate::entity::Entity;
use crate::interpolation::Interpolation;
use crate::network::{Command, NetEvent};
use crate::shapes::Shape;
use crate::spawn;
use common::Position;

/// Nombre de ticks entre deux `LIST` au serveur — `Interpolation` lisse le
/// mouvement des autres joueurs entre deux réponses, donc pas besoin de les relire à chaque frame.
const LIST_POLL_INTERVAL: u32 = 10;

/// Boucle principale : touches → simulation → réseau (non bloquant, voir `network::spawn`) → dessin.
pub fn run(
    window: &mut PistonWindow,
    entities: &mut Vec<Entity>,
    ids: &mut Vec<usize>,
    interpolations: &mut Vec<Interpolation>,
    bindings: &HashMap<Key, Direction>,
    bounds: (f64, f64),
    commands: &Sender<Command>,
    events: &Receiver<NetEvent>,
) {
    let mut frame: u32 = 0;

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            handle_key(key, true, bindings, entities);
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            handle_key(key, false, bindings, entities);
        }

        if let Some(update) = event.update_args() {
            while let Ok(net_event) = events.try_recv() {
                match net_event {
                    NetEvent::Moved(position) => entities[0].set_position(position),
                    NetEvent::List(squares) => spawn::sync_with_server(entities, ids, interpolations, &squares),
                }
            }

            // D'abord les entités distantes, pour que la collision locale les
            // voie à leur position affichée cette frame, pas leur dernière position réseau.
            let step = 1.0 / LIST_POLL_INTERVAL as f64;
            for (entity, interpolation) in entities.iter_mut().zip(interpolations.iter_mut()).skip(1) {
                interpolation.advance(step);
                entity.set_position(interpolation.current());
            }

            let before = entities[0].position();
            let others: Vec<(Position, &dyn Shape)> = entities[1..]
                .iter()
                .map(|other| (other.position(), other.shape()))
                .collect();
            let after = entities[0].resolve_movement(update.dt, bounds, &others);
            entities[0].set_position(after);

            if after != before {
                // Prédiction locale immédiate ; le serveur confirmera ou corrigera via un futur `NetEvent::Moved`.
                let _ = commands.send(Command::Move(after));
            }

            frame += 1;
            if frame % LIST_POLL_INTERVAL == 0 {
                let _ = commands.send(Command::RequestList);
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
            entity
                .shape()
                .draw(entity.position(), entity.color, context.transform, graphics);
        }
    });
}

/// Répercute un appui (`pressed = true`) ou un relâchement (`pressed =
/// false`) de `key` sur notre carré local (`entities[0]`), via
/// `controls::resolve`.
fn handle_key(
    key: Key,
    pressed: bool,
    bindings: &HashMap<Key, Direction>,
    entities: &mut [Entity],
) {
    if let Some(direction) = controls::resolve(key, bindings) {
        entities[0].set_moving(direction, pressed);
    }
}
