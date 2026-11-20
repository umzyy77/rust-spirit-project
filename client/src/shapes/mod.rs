mod circle;
mod rect;

// Jamais construit en production (le client réseau ne fait que des Rect), gardé pour les tests de gjk.
#[allow(unused_imports)]
pub use circle::Circle;
pub use rect::Rect;

use common::Position;

/// Primitive piston pour dessiner la boîte englobante d'une forme (`Ellipse` : voir `Circle`).
#[allow(dead_code)]
pub enum RenderKind {
    Rect,
    Ellipse,
}

/// Une forme convexe centrée sur une `Position`. Toute la géométrie passe par
/// `support` (point le plus loin dans une direction) : c'est la seule chose
/// dont GJK a besoin pour comparer deux formes sans connaître leur type
/// concret. `Drawable` est fourni automatiquement à partir de ce trait.
pub trait Shape {
    /// Point le plus loin de la forme (centrée en `center`) le long de `direction`.
    fn support(&self, center: Position, direction: (f64, f64)) -> Position;

    /// Demi-largeur et demi-hauteur de la boîte englobante.
    fn half_extents(&self) -> (f64, f64);

    fn render_kind(&self) -> RenderKind;
}
