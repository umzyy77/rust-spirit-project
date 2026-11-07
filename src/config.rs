use std::collections::HashSet;

use crate::direction::Direction;

/// Nombre de lettres de l'alphabet (A-Z).
const ALPHABET_SIZE: usize = 26;
/// Nombre de touches à fournir par carré : une par direction existante.
pub const LETTERS_PER_GROUP: usize = Direction::ALL.len();
/// Nombre maximal de carrés constructibles : au-delà, il ne reste plus assez
/// de lettres uniques pour former un nouveau groupe sans réutiliser une
/// touche déjà prise (cf. la règle "aucune touche répétée" plus bas).
pub const MAX_SQUARES: usize = ALPHABET_SIZE / LETTERS_PER_GROUP;

/// Une suite de lettres validée — l'ordre des lettres est celui de
/// `Direction::ALL`.
pub type ControlGroup = [char; LETTERS_PER_GROUP];

/// Transforme les arguments bruts de la ligne de commande (sans le nom du
/// programme) en suites de touches validées — une par carré à créer.
///
/// Règles de validité :
/// - il faut au moins une suite de 4 lettres ;
/// - chaque argument doit faire exactement 4 lettres alphabétiques (A-Z) ;
/// - aucune lettre ne doit apparaître deux fois, ni dans un même groupe, ni
///   entre deux groupes différents.
pub fn parse_args(args: &[String]) -> Result<Vec<ControlGroup>, String> {
    if args.is_empty() {
        return Err(
            "Aucun carré à créer. Usage : squares.exe AZER QSDF WXCV (une suite de 4 lettres par carré)."
                .to_string(),
        );
    }

    let mut seen_letters = HashSet::new();
    let mut groups = Vec::with_capacity(args.len());

    for arg in args {
        let letters: Vec<char> = arg.chars().collect();

        if letters.len() != LETTERS_PER_GROUP || !letters.iter().all(|c| c.is_ascii_alphabetic()) {
            return Err(format!(
                "Argument invalide : \"{arg}\". Chaque carré doit être décrit par exactement {LETTERS_PER_GROUP} lettres (A-Z)."
            ));
        }

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
        let group: ControlGroup = uppercased.try_into().unwrap();
        groups.push(group);
    }

    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn accepte_un_appel_valide() {
        let result = parse_args(&args(&["AZER", "QSDF", "WXCV"]));
        assert_eq!(
            result,
            Ok(vec![
                ['A', 'Z', 'E', 'R'],
                ['Q', 'S', 'D', 'F'],
                ['W', 'X', 'C', 'V'],
            ])
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
        assert_eq!(result[0], ['A', 'Z', 'E', 'R']);
    }
}
