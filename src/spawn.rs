use crate::config::{ControlGroup, MAX_ENTITIES};
use crate::entity::{Color, Entity, SIZE};

const COLORS: [Color; MAX_ENTITIES] = [
    [0.90, 0.25, 0.25, 1.0], // rouge
    [0.25, 0.55, 0.90, 1.0], // bleu
    [0.30, 0.80, 0.35, 1.0], // vert
    [0.95, 0.75, 0.20, 1.0], // jaune
    [0.70, 0.35, 0.85, 1.0], // violet
    [0.95, 0.55, 0.20, 1.0], // orange
];

/// Une entité par groupe de touches validé, réparties sur la largeur de la
/// fenêtre (`width`/`height`). Même source (`groups`) que
/// `controls::build_bindings`, donc `entities.len() == groups.len()` par
/// construction.
pub fn build_entities(groups: &[ControlGroup], width: f64, height: f64) -> Vec<Entity> {
    let count = groups.len();
    let y = (height - SIZE) / 2.0;

    (0..count)
        .map(|i| {
            let x = if count == 1 {
                (width - SIZE) / 2.0
            } else {
                let step = (width - SIZE) / (count - 1) as f64;
                i as f64 * step
            };
            Entity::new(x, y, COLORS[i % COLORS.len()])
        })
        .collect()
}
