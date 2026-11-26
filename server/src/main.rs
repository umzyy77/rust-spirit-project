use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use common::{Position, SIZE};

/// Un carré connecté : sa position, et la dernière fois qu'on a reçu un
/// message de son client (pour détecter une coupure réseau brutale, sans
/// FIN ni RST, que la fermeture normale du socket ne voit jamais).
#[derive(Clone, Copy)]
struct Square {
    position: Position,
    last_seen: Instant,
}

/// Carrés partagés entre tous les clients connectés, derrière un seul `Mutex`.
/// `None` = slot libre (client déconnecté ou expiré).
type Squares = Arc<Mutex<Vec<Option<Square>>>>;

const ADDRESS: &str = "127.0.0.1:7878";

/// Limite de joueurs simultanés : sans elle, une rafale de connexions
/// ouvrirait un thread par client sans borne (le risque d'épuisement des
/// ressources contre lequel le chapitre 21.2 du Rust book protège avec un
/// `ThreadPool` — inadapté ici, nos connexions sont longues, pas des
/// requêtes courtes ; limiter le nombre de joueurs est la bonne équivalence).
const MAX_PLAYERS: usize = 16;

/// Au-delà de ce délai sans message, un client est considéré déconnecté.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);
/// Fréquence à laquelle on vérifie les clients inactifs.
const REAP_INTERVAL: Duration = Duration::from_secs(1);

fn main() {
    let listener = TcpListener::bind(ADDRESS).expect("impossible d'ouvrir le port");
    println!("Serveur en ecoute sur {ADDRESS}");

    run(listener, Arc::new(Mutex::new(Vec::new())));
}

/// Boucle d'acceptation : un thread par client connecté, tous partageant
/// `squares`, plus un thread de nettoyage des clients inactifs (`reap_stale_clients`).
fn run(listener: TcpListener, squares: Squares) {
    thread::spawn({
        let squares = Arc::clone(&squares);
        move || reap_stale_clients(squares)
    });

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Connexion refusee : {err}");
                continue;
            }
        };

        // Messages courts et fréquents : désactive Nagle pour éviter ~40ms de pause.
        if let Err(err) = stream.set_nodelay(true) {
            eprintln!("Impossible de desactiver Nagle : {err}");
        }

        if squares.lock().unwrap().iter().flatten().count() >= MAX_PLAYERS {
            eprintln!("Serveur plein (max {MAX_PLAYERS} joueurs) : connexion refusee.");
            continue; // `stream` est abandonné ici, ce qui ferme la connexion
        }

        let squares = Arc::clone(&squares);
        thread::spawn(move || handle_client(stream, squares));
    }
}

/// Libère (marque `None`) les slots dont le client n'a plus donné signe de
/// vie depuis `CLIENT_TIMEOUT` : seul filet de sécurité pour une coupure
/// réseau brutale, que la fermeture normale du socket ne détecte jamais.
fn reap_stale_clients(squares: Squares) {
    loop {
        thread::sleep(REAP_INTERVAL);

        let mut squares = squares.lock().unwrap();
        for slot in squares.iter_mut() {
            slot.take_if(|square| square.last_seen.elapsed() > CLIENT_TIMEOUT);
        }
    }
}

/// Tous les carrés font `SIZE` de côté : un simple test AABB suffit, pas besoin de GJK.
fn overlaps(a: Position, b: Position) -> bool {
    (a.x() - b.x()).abs() < SIZE && (a.y() - b.y()).abs() < SIZE
}

/// Première position libre pour un nouveau carré, en avançant par pas de `SIZE`.
fn pick_spawn_position(existing: &[Position]) -> Position {
    let mut x = SIZE / 2.0;

    loop {
        let candidate = Position::new(x, SIZE / 2.0);
        if !existing.iter().any(|&other| overlaps(candidate, other)) {
            return candidate;
        }

        x += SIZE;
    }
}

/// Le serveur fait foi sur les collisions entre joueurs : un déplacement qui
/// chevaucherait un autre carré est recalé ici avant d'être accepté. Plusieurs
/// passes car corriger un chevauchement peut en réintroduire un autre.
fn resolve_collisions(mut candidate: Position, others: &[Position]) -> Position {
    for _ in 0..4 {
        let mut corrected = false;

        for &other in others {
            if !overlaps(candidate, other) {
                continue;
            }

            let dx = candidate.x() - other.x();
            let dy = candidate.y() - other.y();
            let overlap_x = SIZE - dx.abs();
            let overlap_y = SIZE - dy.abs();

            candidate = if overlap_x < overlap_y {
                let sign = if dx < 0.0 { -1.0 } else { 1.0 };
                Position::new(other.x() + sign * SIZE, candidate.y())
            } else {
                let sign = if dy < 0.0 { -1.0 } else { 1.0 };
                Position::new(candidate.x(), other.y() + sign * SIZE)
            };
            corrected = true;
        }

        if !corrected {
            break;
        }
    }

    candidate
}

/// Crée la position du client (son id = son index dans `squares`) sous un
/// seul verrou — sinon deux connexions simultanées pourraient obtenir le
/// même id ou la même position — puis traite ses messages jusqu'à
/// déconnexion, où son slot est marqué libre (`None`) pour ne plus
/// apparaître dans `LIST`.
fn handle_client(mut stream: TcpStream, squares: Squares) {
    let id = {
        let mut squares = squares.lock().unwrap();
        let existing: Vec<Position> = squares.iter().flatten().map(|square| square.position).collect();
        let position = pick_spawn_position(&existing);
        squares.push(Some(Square { position, last_seen: Instant::now() }));
        squares.len() - 1
    };

    if writeln!(stream, "ID {id}").is_err() {
        return;
    }

    let Ok(reader_stream) = stream.try_clone() else {
        return;
    };

    for line in BufReader::new(reader_stream).lines() {
        let Ok(line) = line else { break };

        if !handle_message(&line, id, &squares, &mut stream) {
            break;
        }
    }

    squares.lock().unwrap()[id] = None;
    println!("Client {id} deconnecte.");
}

/// Traite un message du client `id`. `false` signifie "ferme la connexion"
/// (message mal formé irrécupérable ou écriture réseau échouée).
fn handle_message(line: &str, id: usize, squares: &Squares, stream: &mut TcpStream) -> bool {
    // Un message, quel qu'il soit, prouve que le client est toujours là.
    if let Some(square) = squares.lock().unwrap()[id].as_mut() {
        square.last_seen = Instant::now();
    }

    let mut parts = line.split_whitespace();

    match parts.next() {
        Some("MOVE") => {
            let coordinates = parts.next().and_then(|x| x.parse().ok()).zip(parts.next().and_then(|y| y.parse().ok()));

            let Some((x, y)) = coordinates else {
                eprintln!("MOVE invalide recu du client {id} : \"{line}\"");
                return true;
            };

            let resolved = {
                let mut squares = squares.lock().unwrap();
                let others: Vec<Position> = squares
                    .iter()
                    .enumerate()
                    .filter(|&(other_id, _)| other_id != id)
                    .filter_map(|(_, slot)| slot.map(|square| square.position))
                    .collect();

                let resolved = resolve_collisions(Position::new(x, y), &others);
                squares[id].as_mut().unwrap().position = resolved;
                resolved
            };

            writeln!(stream, "POS {} {}", resolved.x(), resolved.y()).is_ok()
        }
        Some("LIST") => {
            let snapshot: Vec<Option<Square>> = squares.lock().unwrap().clone();

            for (other_id, slot) in snapshot.iter().enumerate() {
                let Some(square) = slot else { continue };
                if writeln!(stream, "{other_id} {} {}", square.position.x(), square.position.y()).is_err() {
                    return false;
                }
            }
            writeln!(stream, "END").is_ok()
        }
        _ => {
            eprintln!("Message inconnu du client {id} : \"{line}\"");
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_server() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || run(listener, Arc::new(Mutex::new(Vec::new()))));
        address
    }

    fn read_line(reader: &mut impl BufRead) -> String {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        line.trim_end().to_string()
    }

    #[test]
    fn picks_a_free_spawn_position() {
        assert_eq!(pick_spawn_position(&[]), Position::new(25.0, 25.0));

        let one_occupied = [Position::new(25.0, 25.0)];
        assert_eq!(pick_spawn_position(&one_occupied), Position::new(75.0, 25.0));

        let two_occupied = [Position::new(25.0, 25.0), Position::new(75.0, 25.0)];
        assert_eq!(pick_spawn_position(&two_occupied), Position::new(125.0, 25.0));
    }

    #[test]
    fn resolve_collisions_pushes_out_of_overlap() {
        let candidate = Position::new(60.0, 25.0);
        let corrected = resolve_collisions(candidate, &[Position::new(75.0, 25.0)]);
        assert_eq!(corrected, Position::new(25.0, 25.0));
    }

    #[test]
    fn resolve_collisions_leaves_non_overlapping_position_untouched() {
        let candidate = Position::new(200.0, 200.0);
        let corrected = resolve_collisions(candidate, &[Position::new(75.0, 25.0)]);
        assert_eq!(corrected, candidate);
    }

    #[test]
    fn server_rejects_a_move_that_would_overlap_another_player() {
        let address = start_server();

        let mut client_a = TcpStream::connect(address).unwrap();
        let mut reader_a = BufReader::new(client_a.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_a), "ID 0"); // spawn (25, 25)

        let client_b = TcpStream::connect(address).unwrap();
        let mut reader_b = BufReader::new(client_b.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_b), "ID 1"); // spawn (75, 25)

        // A s'approche trop de B : le serveur doit refuser et recaler.
        writeln!(client_a, "MOVE 70 25").unwrap();
        assert_eq!(read_line(&mut reader_a), "POS 25 25");
    }

    #[test]
    fn assigns_sequential_ids_and_tracks_moves() {
        let address = start_server();

        let mut client_a = TcpStream::connect(address).unwrap();
        let mut reader_a = BufReader::new(client_a.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_a), "ID 0");

        let mut client_b = TcpStream::connect(address).unwrap();
        let mut reader_b = BufReader::new(client_b.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_b), "ID 1");

        // Attendre la réponse au LIST garantit que le MOVE est déjà appliqué
        // avant que B n'interroge à son tour.
        writeln!(client_a, "MOVE 5 7").unwrap();
        assert_eq!(read_line(&mut reader_a), "POS 5 7");
        writeln!(client_a, "LIST").unwrap();
        while read_line(&mut reader_a) != "END" {}

        writeln!(client_b, "LIST").unwrap();
        let mut lines = Vec::new();
        loop {
            let line = read_line(&mut reader_b);
            if line == "END" {
                break;
            }
            lines.push(line);
        }

        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&"0 5 7".to_string()));
        // B spawn juste après A (qui occupait (25, 25)) : pas de chevauchement.
        assert!(lines.contains(&"1 75 25".to_string()));
    }

    #[test]
    fn disconnecting_removes_the_square_from_list() {
        let address = start_server();

        let client_a = TcpStream::connect(address).unwrap();
        let mut reader_a = BufReader::new(client_a); // un seul handle sur le socket de A
        assert_eq!(read_line(&mut reader_a), "ID 0");
        drop(reader_a); // ferme reellement la connexion (pas de clone qui la garderait ouverte)

        // Le serveur ne détecte la déconnexion qu'en essayant de lire sur le
        // socket fermé, dans son propre thread : on retente jusqu'à ce que
        // LIST ne renvoie plus le carré 0.
        for _ in 0..50 {
            let mut client_b = TcpStream::connect(address).unwrap();
            let mut reader_b = BufReader::new(client_b.try_clone().unwrap());
            read_line(&mut reader_b); // "ID <n>"

            writeln!(client_b, "LIST").unwrap();
            let mut lines = Vec::new();
            loop {
                let line = read_line(&mut reader_b);
                if line == "END" {
                    break;
                }
                lines.push(line);
            }

            if lines.iter().all(|line| !line.starts_with("0 ")) {
                return;
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }

        panic!("le carre du client deconnecte est toujours dans LIST");
    }

    #[test]
    fn rejects_connections_beyond_max_players() {
        let address = start_server();

        // Gardées ouvertes : sinon leur slot se libérerait et la limite ne serait jamais atteinte.
        let mut clients = Vec::new();
        for i in 0..MAX_PLAYERS {
            let client = TcpStream::connect(address).unwrap();
            let mut reader = BufReader::new(client.try_clone().unwrap());
            assert_eq!(read_line(&mut reader), format!("ID {i}"));
            clients.push(client);
        }

        let one_too_many = TcpStream::connect(address).unwrap();
        let mut reader = BufReader::new(one_too_many);
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).unwrap();
        assert_eq!(bytes_read, 0, "le serveur aurait du fermer la connexion en trop, pas repondre \"{line}\"");
    }
}
