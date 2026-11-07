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
