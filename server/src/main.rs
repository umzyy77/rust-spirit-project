use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use common::{Position, SIZE};

/// Carrés partagés entre tous les clients connectés, derrière un seul `Mutex`.
type Squares = Arc<Mutex<Vec<Position>>>;

const ADDRESS: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDRESS).expect("impossible d'ouvrir le port");
    println!("Serveur en ecoute sur {ADDRESS}");

    run(listener, Arc::new(Mutex::new(Vec::new())));
}

/// Boucle d'acceptation : un thread par client connecté, tous partageant `squares`.
fn run(listener: TcpListener, squares: Squares) {
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

        let squares = Arc::clone(&squares);
        thread::spawn(move || handle_client(stream, squares));
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
/// même id ou la même position — puis traite ses messages jusqu'à déconnexion.
fn handle_client(mut stream: TcpStream, squares: Squares) {
    let id = {
        let mut squares = squares.lock().unwrap();
        let position = pick_spawn_position(&squares);
        squares.push(position);
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

    println!("Client {id} deconnecte.");
}

/// Traite un message du client `id`. `false` signifie "ferme la connexion"
/// (message mal formé irrécupérable ou écriture réseau échouée).
fn handle_message(line: &str, id: usize, squares: &Squares, stream: &mut TcpStream) -> bool {
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
                let others: Vec<Position> =
                    squares.iter().enumerate().filter(|&(other_id, _)| other_id != id).map(|(_, &p)| p).collect();

                let resolved = resolve_collisions(Position::new(x, y), &others);
                squares[id] = resolved;
                resolved
            };

            writeln!(stream, "POS {} {}", resolved.x(), resolved.y()).is_ok()
        }
        Some("LIST") => {
            let snapshot: Vec<Position> = squares.lock().unwrap().clone();

            for (other_id, position) in snapshot.iter().enumerate() {
                if writeln!(stream, "{other_id} {} {}", position.x(), position.y()).is_err() {
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
}
