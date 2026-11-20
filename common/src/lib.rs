/// Position partagée entre `server` (qui ne fait que la suivre) et `client`
/// (qui l'affiche et la fait évoluer localement) — la seule chose que les
/// deux bouts du réseau ont besoin de connaître en commun pour l'instant.
#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    /// Crée une position à `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    /// Coordonnée x.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Coordonnée y.
    pub fn y(&self) -> f64 {
        self.y
    }
}
