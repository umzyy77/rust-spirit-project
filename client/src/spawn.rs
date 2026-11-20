use std::sync::mpsc::{Receiver, TryRecvError};

use crate::entity::{Color, Entity};
use crate::interpolation::Interpolation;
use crate::network::NetEvent;
use crate::shapes::Rect;
use common::{Position, SIZE};

/// Couleurs attribuées aux carrés réseau, par `id % COLORS.len()`.
const COLORS: [Color; 6] = [
    [0.90, 0.25, 0.25, 1.0], // rouge
    [0.25, 0.55, 0.90, 1.0], // bleu
    [0.30, 0.80, 0.35, 1.0], // vert
    [0.95, 0.75, 0.20, 1.0], // jaune
    [0.70, 0.35, 0.85, 1.0], // violet
    [0.95, 0.55, 0.20, 1.0], // orange
];

/// Construit les entités connues à la connexion : notre carré (`local_id`)
/// en premier (indice `0`, celui que `entities[0]` et les touches
/// contrôlent), puis les autres. `ids` garde l'id serveur de chaque entité
/// dans le même ordre, pour que `gameloop` retrouve la bonne entité à partir
/// d'un id reçu du réseau ; `interpolations` sert au lissage des autres (pas
/// de la nôtre, simulée localement).
pub fn build_networked_entities(
    local_id: usize,
    squares: &[(usize, Position)],
) -> (Vec<Entity>, Vec<usize>, Vec<Interpolation>) {
    let ordered = squares
        .iter()
        .filter(|&&(id, _)| id == local_id)
        .chain(squares.iter().filter(|&&(id, _)| id != local_id));

    let mut entities = Vec::with_capacity(squares.len());
    let mut ids = Vec::with_capacity(squares.len());
    let mut interpolations = Vec::with_capacity(squares.len());

    for &(id, position) in ordered {
        let shape = Box::new(Rect { width: SIZE, height: SIZE });
        entities.push(Entity::new(position.x(), position.y(), shape, COLORS[id % COLORS.len()]));
        ids.push(id);
        interpolations.push(Interpolation::at(position));
    }

    (entities, ids, interpolations)
}

/// Met à jour `ids`/`interpolations` à partir d'une réponse `LIST` : un id
/// déjà connu reçoit une nouvelle cible (`Interpolation::retarget`), un id
/// inconnu devient une nouvelle entité. Ne touche jamais `entities[0]`/`ids[0]`
/// (notre carré) : on reste seul autorité sur notre propre position.
pub fn sync_with_server(
    entities: &mut Vec<Entity>,
    ids: &mut Vec<usize>,
    interpolations: &mut Vec<Interpolation>,
    squares: &[(usize, Position)],
) {
    let local_id = ids[0];

    for &(id, position) in squares {
        if id == local_id {
            continue;
        }

        match ids.iter().position(|&existing| existing == id) {
            Some(index) => interpolations[index].retarget(position),
            None => {
                let shape = Box::new(Rect { width: SIZE, height: SIZE });
                entities.push(Entity::new(position.x(), position.y(), shape, COLORS[id % COLORS.len()]));
                ids.push(id);
                interpolations.push(Interpolation::at(position));
            }
        }
    }
}

/// Applique les `NetEvent` déjà arrivés. Renvoie `false` si le thread réseau
/// a disparu (connexion perdue) : on arrête alors de le solliciter pour le
/// reste de la partie, plutôt que de continuer comme si de rien n'était.
pub fn apply_network_events(
    events: &Receiver<NetEvent>,
    entities: &mut Vec<Entity>,
    ids: &mut Vec<usize>,
    interpolations: &mut Vec<Interpolation>,
) -> bool {
    loop {
        match events.try_recv() {
            Ok(NetEvent::Moved(position)) => entities[0].set_position(position),
            Ok(NetEvent::List(squares)) => sync_with_server(entities, ids, interpolations, &squares),
            Err(TryRecvError::Empty) => return true,
            Err(TryRecvError::Disconnected) => {
                eprintln!("Connexion au serveur perdue : on continue hors-ligne.");
                return false;
            }
        }
    }
}

/// Avance l'interpolation des entités distantes (toutes sauf la nôtre, à l'indice 0).
pub fn advance_remote_entities(entities: &mut [Entity], interpolations: &mut [Interpolation], step: f64) {
    for (entity, interpolation) in entities.iter_mut().zip(interpolations.iter_mut()).skip(1) {
        interpolation.advance(step);
        entity.set_position(interpolation.current());
    }
}
