#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    /// Toutes les variantes, dans l'ordre Haut, Gauche, Bas, Droite — c'est
    /// cet ordre qui donne son sens à `config::ControlGroup` (1ère lettre =
    /// Haut, etc.).
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right
    ];

    /// Vecteur unitaire (dx, dy) correspondant à cette direction.
    pub fn delta(&self) -> (f64, f64) {
        match self {
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
        }
    }
}
