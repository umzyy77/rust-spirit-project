/// Taille (largeur et hauteur) des carrés réseau — partagée car le serveur
/// en a aussi besoin pour placer les spawns, pas seulement pour le dessin.
pub const SIZE: f64 = 50.0;

/// Position partagée entre `server` et `client`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}
