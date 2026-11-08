use std::collections::HashMap;

use piston_window::{Key, PistonWindow, WindowSettings};

use crate::actors::Actor;
use crate::config::EntityConfig;
use crate::direction::Direction;
use crate::{controls, gameloop, spawn};

/// Regroupe tout ce dont la boucle de jeu a besoin (fenêtre, entités,
/// liaisons clavier, limites) — point d'extension si l'app gagne un jour un
/// second point d'entrée (tests, mode headless...).
pub struct App {
    window: PistonWindow,
    entities: Vec<Box<dyn Actor>>,
    bindings: HashMap<Key, (usize, Direction)>,
    bounds: (f64, f64),
}

impl App {
    /// Construit la fenêtre et les sous-systèmes à partir des configurations
    /// déjà validées par `config::parse_args`.
    pub fn bootstrap(configs: &[EntityConfig], width: f64, height: f64) -> Result<Self, String> {
        let window = WindowSettings::new("Squares", [width as u32, height as u32])
            .build()
            .map_err(|err| format!("Erreur de création de la fenêtre : {err}"))?;

        let entities = spawn::build_entities(configs, width, height);
        let bindings = controls::build_bindings(configs);

        Ok(App {
            window,
            entities,
            bindings,
            bounds: (width, height),
        })
    }

    /// Lance la boucle de jeu jusqu'à la fermeture de la fenêtre.
    pub fn run(mut self) {
        gameloop::run(
            &mut self.window,
            &mut self.entities,
            &self.bindings,
            self.bounds,
        );
    }
}
