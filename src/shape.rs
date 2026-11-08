/// Donnée choisie par l'utilisateur ; sa traduction en mesh/collider
/// concrets est à la charge de la scène (seule à dépendre de Bevy).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Square,
    Circle,
}

impl Shape {
    /// La première sert de forme par défaut.
    pub const ALL: [Shape; 2] = [Shape::Square, Shape::Circle];

    /// Nom utilisé en ligne de commande. `match` exhaustif : oublier une
    /// variante ici ne compile pas.
    pub fn name(self) -> &'static str {
        match self {
            Shape::Square => "square",
            Shape::Circle => "circle",
        }
    }

    /// Trouve la forme dont le nom correspond à `name`, insensible à la casse.
    pub fn from_name(name: &str) -> Option<Shape> {
        Shape::ALL.into_iter().find(|shape| shape.name().eq_ignore_ascii_case(name))
    }
}
