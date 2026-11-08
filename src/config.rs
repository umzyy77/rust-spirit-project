use std::collections::HashSet;

use crate::actors::{Builder, Square, SHAPES};
use crate::direction::Direction;

/// Nombre de lettres de l'alphabet (A-Z).
const ALPHABET_SIZE: usize = 26;
/// Nombre de touches à fournir par entité : une par direction existante.
pub const LETTERS_PER_GROUP: usize = Direction::ALL.len();
/// Nombre maximal d'entités constructibles : au-delà, il ne reste plus assez
/// de lettres uniques pour former un nouveau groupe sans réutiliser une
/// touche déjà prise (cf. la règle "aucune touche répétée" plus bas).
pub const MAX_ENTITIES: usize = ALPHABET_SIZE / LETTERS_PER_GROUP;

/// Une suite de lettres validée — l'ordre des lettres est celui de
/// `Direction::ALL`.
pub type ControlGroup = [char; LETTERS_PER_GROUP];

/// Constructeur par défaut quand l'argument ne précise pas de forme.
const DEFAULT_SHAPE: Builder = Square::boxed;

/// Trouve le constructeur associé à `name` dans `actors::SHAPES`, insensible
/// à la casse.
fn find_shape(name: &str) -> Option<Builder> {
    SHAPES
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|&(_, build)| build)
}

/// Touches + constructeur de forme d'une entité, issus d'un seul argument de
/// ligne de commande. Pas de `PartialEq`/`Debug` dérivés : comparer des
/// pointeurs de fonction n'est pas fiable (l'optimiseur peut fusionner deux
/// fonctions identiques et les rendre "égales" par erreur).
#[derive(Clone, Copy)]
pub struct EntityConfig {
    pub keys: ControlGroup,
    pub build: Builder,
}

/// Transforme les arguments bruts de la ligne de commande (sans le nom du
/// programme) en configurations validées — une par entité à créer.
///
/// Chaque argument est `LETTRES` ou `LETTRES:forme` (`forme` = `circle` ou
/// `square`, insensible à la casse ; `square` par défaut si omis).
///
/// Règles de validité :
/// - il faut au moins un argument ;
/// - la partie lettres doit faire exactement 4 lettres alphabétiques (A-Z) ;
/// - aucune lettre ne doit apparaître deux fois, ni dans un même groupe, ni
///   entre deux groupes différents ;
/// - la forme, si précisée, doit être `circle` ou `square`.
pub fn parse_args(args: &[String]) -> Result<Vec<EntityConfig>, String> {
    if args.is_empty() {
        return Err(
            "Aucune entité à créer. Usage : squares.exe AZER QSDF:circle WXCV (une suite de 4 lettres par entité, suivie en option de \":circle\" ou \":square\" — carré par défaut)."
                .to_string(),
        );
    }

    let mut seen_letters = HashSet::new();
    let mut configs = Vec::with_capacity(args.len());

    for arg in args {
        let (letters_part, shape_part) = match arg.split_once(':') {
            Some((letters, shape)) => (letters, Some(shape)),
            None => (arg.as_str(), None),
        };

        let letters: Vec<char> = letters_part.chars().collect();

        if letters.len() != LETTERS_PER_GROUP || !letters.iter().all(|c| c.is_ascii_alphabetic()) {
            return Err(format!(
                "Argument invalide : \"{arg}\". Chaque entité doit être décrite par exactement {LETTERS_PER_GROUP} lettres (A-Z), suivies en option de \":circle\" ou \":square\"."
            ));
        }

        let build = match shape_part {
            None => DEFAULT_SHAPE,
            Some(name) => find_shape(name).ok_or_else(|| {
                format!("Forme inconnue : \"{name}\". Utilise \"circle\" ou \"square\" (carré par défaut).")
            })?,
        };

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
        let keys: ControlGroup = uppercased.try_into().unwrap();
        configs.push(EntityConfig { keys, build });
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
        assert_eq!(
            keys,
            vec![['A', 'Z', 'E', 'R'], ['Q', 'S', 'D', 'F'], ['W', 'X', 'C', 'V']]
        );
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
}
