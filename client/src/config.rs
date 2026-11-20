use std::collections::HashSet;

use crate::direction::Direction;

/// Nombre de touches à fournir : une par direction existante.
pub const LETTERS_PER_GROUP: usize = Direction::ALL.len();

/// Le groupe de lettres validé pour notre seul carré local — l'ordre des
/// lettres est celui de `Direction::ALL`.
pub type ControlGroup = [char; LETTERS_PER_GROUP];

/// Valide l'unique argument de touches reçu sur la ligne de commande (le
/// client réseau ne contrôle qu'un seul carré, donc un seul groupe).
///
/// Règles de validité :
/// - l'argument doit faire exactement 4 lettres alphabétiques (A-Z) ;
/// - aucune lettre ne doit apparaître deux fois.
pub fn parse_args(arg: &str) -> Result<ControlGroup, String> {
    let letters: Vec<char> = arg.chars().collect();

    if letters.len() != LETTERS_PER_GROUP || !letters.iter().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!(
            "Argument invalide : \"{arg}\". Le carré doit être décrit par exactement {LETTERS_PER_GROUP} lettres (A-Z)."
        ));
    }

    let mut seen_letters = HashSet::new();
    let mut uppercased = Vec::with_capacity(LETTERS_PER_GROUP);
    for c in &letters {
        let upper = c.to_ascii_uppercase();
        if !seen_letters.insert(upper) {
            return Err(format!(
                "La touche '{upper}' est utilisée plusieurs fois. Chaque touche ne doit contrôler qu'une seule direction."
            ));
        }
        uppercased.push(upper);
    }

    // `unwrap` sûr : `uppercased.len() == LETTERS_PER_GROUP`, vérifié plus haut.
    Ok(uppercased.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepte_un_appel_valide() {
        assert_eq!(parse_args("AZER"), Ok(['A', 'Z', 'E', 'R']));
    }

    #[test]
    fn refuse_un_groupe_de_mauvaise_taille() {
        assert!(parse_args("AZE").is_err());
        assert!(parse_args("AZERT").is_err());
    }

    #[test]
    fn refuse_un_caractere_non_alphabetique() {
        assert!(parse_args("AZ3R").is_err());
    }

    #[test]
    fn refuse_une_touche_repetee() {
        assert!(parse_args("AAZE").is_err());
    }

    #[test]
    fn est_insensible_a_la_casse() {
        assert_eq!(parse_args("azer"), Ok(['A', 'Z', 'E', 'R']));
    }
}
