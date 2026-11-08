use crate::actors::Actor;
use crate::config::{EntityConfig, MAX_ENTITIES};
use crate::entity::Color;

const COLORS: [Color; MAX_ENTITIES] = [
    [0.90, 0.25, 0.25, 1.0], // rouge
    [0.25, 0.55, 0.90, 1.0], // bleu
    [0.30, 0.80, 0.35, 1.0], // vert
    [0.95, 0.75, 0.20, 1.0], // jaune
    [0.70, 0.35, 0.85, 1.0], // violet
    [0.95, 0.55, 0.20, 1.0], // orange
];

/// Demi-largeur (ou rayon) des entités créées par `build_entities`.
const HALF_SIZE: f64 = 25.0;

/// Une entité par configuration validée, réparties sur la largeur de la
/// fenêtre, position = centre. `config.build` sait déjà quel `Actor`
/// construire, aucune distinction carré/cercle ici.
pub fn build_entities(configs: &[EntityConfig], width: f64, height: f64) -> Vec<Box<dyn Actor>> {
    let count = configs.len();
    let size = HALF_SIZE * 2.0;
    let y = height / 2.0;

    (0..count)
        .map(|i| {
            let x = if count == 1 {
                width / 2.0
            } else {
                let step = (width - size) / (count - 1) as f64;
                HALF_SIZE + i as f64 * step
            };
            (configs[i].build)(x, y, COLORS[i % COLORS.len()], HALF_SIZE)
        })
        .collect()
}
