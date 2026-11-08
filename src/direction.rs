/// Une direction de déplacement, associée à une touche du clavier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    /// Ordre Haut/Gauche/Bas/Droite : donne son sens à `config::ControlGroup`
    /// (1ère lettre = Haut, etc.).
    pub const ALL: [Direction; 4] = [Direction::Up, Direction::Left, Direction::Down, Direction::Right];
}
