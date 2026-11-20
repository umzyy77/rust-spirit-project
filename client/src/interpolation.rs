use common::Position;

/// Lisse l'affichage d'une entité distante entre deux positions `LIST`,
/// reçues bien moins souvent que les frames — sinon mouvement par à-coups.
pub struct Interpolation {
    previous: Position,
    target: Position,
    /// `0.0` = vient d'être reciblée, `1.0` = arrivée à `target`.
    progress: f64,
}

impl Interpolation {
    /// Position initiale, sans mouvement à lisser.
    pub fn at(position: Position) -> Self {
        Interpolation { previous: position, target: position, progress: 1.0 }
    }

    /// Position affichée à l'instant présent, entre `previous` et `target`.
    pub fn current(&self) -> Position {
        let t = self.progress.clamp(0.0, 1.0);
        Position::new(
            self.previous.x() + (self.target.x() - self.previous.x()) * t,
            self.previous.y() + (self.target.y() - self.previous.y()) * t,
        )
    }

    /// Reçoit une nouvelle cible, en repartant de la position *actuellement
    /// affichée* pour éviter tout saut visible.
    pub fn retarget(&mut self, target: Position) {
        self.previous = self.current();
        self.target = target;
        self.progress = 0.0;
    }

    /// Avance de `step` (typiquement `1.0 / LIST_POLL_INTERVAL`).
    pub fn advance(&mut self, step: f64) {
        self.progress = (self.progress + step).min(1.0);
    }
}
