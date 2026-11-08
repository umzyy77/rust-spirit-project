use std::collections::HashSet;

use crate::direction::Direction;
use crate::shape::Shape;

const ALPHABET_SIZE: usize = 26;
/// Une touche par direction existante.
pub const KEYS_PER_ENTITY: usize = Direction::ALL.len();
/// Au-delà, il ne reste plus assez de lettres uniques pour un nouveau groupe.
pub const MAX_ENTITIES: usize = ALPHABET_SIZE / KEYS_PER_ENTITY;

/// Les touches d'une entité, dans l'ordre de `Direction::ALL`.
pub type ControlGroup = [char; KEYS_PER_ENTITY];

/// La configuration d'une entité, déduite d'un seul argument de ligne de commande.
#[derive(Clone, Copy)]
pub struct EntityConfig {
    pub keys: ControlGroup,
    pub shape: Shape,
}

/// Transforme les arguments de la ligne de commande (sans le nom du
/// programme) en une configuration par entité à créer.
///
/// Chaque argument a la forme `LETTRES` ou `LETTRES:forme`, où `LETTRES` est
/// une suite d'exactement `KEYS_PER_ENTITY` lettres (A-Z, insensible à la
/// casse, aucune répétée sur l'ensemble des arguments) et `forme` vaut
/// `circle` ou `square` (`square` par défaut si omis).
pub fn parse_args(args: &[String]) -> Result<Vec<EntityConfig>, String> {
    if args.is_empty() {
        return Err(format!(
            "Aucune entité à créer. Usage : squares.exe AZER QSDF:circle WXCV (une suite de {KEYS_PER_ENTITY} lettres par entité, suivie en option de \":circle\" ou \":square\" — carré par défaut)."
        ));
    }

    let mut seen_keys = HashSet::new();
    let mut configs = Vec::with_capacity(args.len());

    for arg in args {
        let (letters_part, shape_part) = match arg.split_once(':') {
            Some((letters, shape)) => (letters, Some(shape)),
            None => (arg.as_str(), None),
        };

        let letters: Vec<char> = letters_part.chars().collect();
        if letters.len() != KEYS_PER_ENTITY || !letters.iter().all(char::is_ascii_alphabetic) {
            return Err(format!(
                "Argument invalide : \"{arg}\". Chaque entité doit être décrite par exactement {KEYS_PER_ENTITY} lettres (A-Z), suivies en option de \":circle\" ou \":square\"."
            ));
        }

        let shape = match shape_part {
            None => Shape::ALL[0],
            Some(name) => Shape::from_name(name)
                .ok_or_else(|| format!("Forme inconnue : \"{name}\". Utilise \"circle\" ou \"square\" (carré par défaut)."))?,
        };

        let mut keys = [' '; KEYS_PER_ENTITY];
        for (slot, letter) in keys.iter_mut().zip(&letters) {
            let key = letter.to_ascii_uppercase();
            if !seen_keys.insert(key) {
                return Err(format!(
                    "La touche '{key}' est utilisée plusieurs fois. Chaque touche ne doit contrôler qu'une seule direction."
                ));
            }
            *slot = key;
        }

        configs.push(EntityConfig { keys, shape });
    }

    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn accepte_un_appel_valide() {
        let result = parse_args(&args(&["AZER", "QSDF", "WXCV"])).unwrap();
        let keys: Vec<ControlGroup> = result.iter().map(|c| c.keys).collect();
        assert_eq!(keys, vec![['A', 'Z', 'E', 'R'], ['Q', 'S', 'D', 'F'], ['W', 'X', 'C', 'V']]);
    }

    #[test]
    fn refuse_un_groupe_de_mauvaise_taille() {
        assert!(parse_args(&args(&["AZE"])).is_err());
        assert!(parse_args(&args(&["AZERT"])).is_err());
    }

    #[test]
    fn refuse_un_caractere_non_alphabetique() {
        assert!(parse_args(&args(&["AZ3R"])).is_err());
    }

    #[test]
    fn refuse_une_touche_repetee_dans_le_meme_groupe() {
        assert!(parse_args(&args(&["AAZE"])).is_err());
    }

    #[test]
    fn refuse_une_touche_repetee_entre_deux_groupes() {
        assert!(parse_args(&args(&["AZER", "RTYU"])).is_err());
    }

    #[test]
    fn refuse_un_appel_sans_argument() {
        assert!(parse_args(&args(&[])).is_err());
    }

    #[test]
    fn est_insensible_a_la_casse() {
        let result = parse_args(&args(&["azer"])).unwrap();
        assert_eq!(result[0].keys, ['A', 'Z', 'E', 'R']);
    }

    #[test]
    fn accepte_une_forme_explicite_et_insensible_a_la_casse() {
        let result = parse_args(&args(&["AZER:Circle"])).unwrap();
        assert_eq!(result[0].shape, Shape::Circle);
    }

    #[test]
    fn refuse_une_forme_inconnue() {
        assert!(parse_args(&args(&["AZER:triangle"])).is_err());
    }
}
