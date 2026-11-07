pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    /// Crée une position à `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    /// Déplace la position de `dx` sur x et `dy` sur y.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    /// Borne `x` à `[0, max_x]` et `y` à `[0, max_y]`.
    pub fn clamp(&mut self, max_x: f64, max_y: f64) {
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }
}
