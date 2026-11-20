mod circle;
mod rect;

pub use circle::Circle;
pub use rect::Rect;

use crate::position::Position;

/// Quelle primitive piston utiliser pour dessiner la boîte englobante d'une
/// forme. Aucune dépendance à piston ici — juste un choix parmi deux formes
/// de dessin ; `crate::drawable` fait le lien avec la bibliothèque.
pub enum RenderKind {
    Rect,
    Ellipse,
}

/// Une forme convexe centrée sur une `Position`. Toute la géométrie
/// spécifique à une forme passe par `support` : le point le plus loin de la
/// forme dans une direction donnée. C'est la seule information dont un test
/// de collision générique (GJK) a besoin — il n'a jamais à connaître le type
/// concret de l'autre forme comparée.
///
/// Ajouter une nouvelle forme = un nouveau fichier dans ce dossier qui
/// implémente ce trait. `Drawable` est fourni automatiquement pour toute
/// forme (voir `crate::drawable`) à partir de `half_extents`/`render_kind` —
/// rien d'autre à écrire pour que le dessin fonctionne.
pub trait Shape {
    /// Point le plus loin de la forme (centrée en `center`) le long de `direction`.
    fn support(&self, center: Position, direction: (f64, f64)) -> Position;

    /// Demi-largeur et demi-hauteur de la boîte englobante, pour le clamp
    /// aux limites de la fenêtre et pour le dessin.
    fn half_extents(&self) -> (f64, f64);

    /// Primitive à utiliser pour dessiner la boîte englobante de cette forme.
    fn render_kind(&self) -> RenderKind;
}
