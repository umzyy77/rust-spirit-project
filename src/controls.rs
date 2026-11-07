use std::collections::HashMap;

use piston_window::Key;

use crate::config::ControlGroup;
use crate::direction::Direction;

/// Construit la table "touche → (indice de l'entité, direction)" à partir
/// des groupes de touches déjà validés par `config::parse_args`. L'indice
/// vient de la position dans `groups` (via `.enumerate()`), donc il
/// correspond toujours exactement à la position de l'entité construite à
/// partir du même `groups` dans `main.rs` : pas de chiffre écrit à la main,
/// pas de désynchronisation possible.
pub fn build_bindings(groups: &[ControlGroup]) -> HashMap<Key, (usize, Direction)> {
    let mut bindings = HashMap::new();

    for (index, group) in groups.iter().enumerate() {
        for (letter, direction) in group.iter().zip(Direction::ALL.iter()) {
            bindings.insert(char_to_key(*letter), (index, *direction));
        }
    }

    bindings
}

/// Interprète `key` : renvoie l'indice de l'entité concernée et la direction
/// associée, si cette touche fait partie des `bindings`.
pub fn resolve(key: Key, bindings: &HashMap<Key, (usize, Direction)>) -> Option<(usize, Direction)> {
    bindings.get(&key).copied()
}

/// Convertit une lettre A-Z (déjà validée par `config::parse_args`) en la
/// touche `piston` correspondante.
fn char_to_key(c: char) -> Key {
    match c {
        'A' => Key::A,
        'B' => Key::B,
        'C' => Key::C,
        'D' => Key::D,
        'E' => Key::E,
        'F' => Key::F,
        'G' => Key::G,
        'H' => Key::H,
        'I' => Key::I,
        'J' => Key::J,
        'K' => Key::K,
        'L' => Key::L,
        'M' => Key::M,
        'N' => Key::N,
        'O' => Key::O,
        'P' => Key::P,
        'Q' => Key::Q,
        'R' => Key::R,
        'S' => Key::S,
        'T' => Key::T,
        'U' => Key::U,
        'V' => Key::V,
        'W' => Key::W,
        'X' => Key::X,
        'Y' => Key::Y,
        'Z' => Key::Z,
        // `config::parse_args` ne laisse passer que des lettres A-Z.
        _ => unreachable!("lettre non validée : {c}"),
    }
}
