use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use common::Position;

/// Carrés partagés entre tous les clients connectés : un seul `Mutex`
/// protège tout le `Vec`, donc chaque lecture/écriture est entièrement
/// sérialisée — pas de donnée à moitié écrite possible.
type Squares = Arc<Mutex<Vec<Position>>>;

const ADDRESS: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDRESS).expect("impossible d'ouvrir le port");
    println!("Serveur en ecoute sur {ADDRESS}");

    run(listener, Arc::new(Mutex::new(Vec::new())));
}

/// Boucle d'acceptation : un thread par client connecté, tous partageant
/// `squares`.
fn run(listener: TcpListener, squares: Squares) {
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Connexion refusee : {err}");
                continue;
            }
        };

        let squares = Arc::clone(&squares);
        thread::spawn(move || handle_client(stream, squares));
    }
}

/// Attribue un identifiant au client en créant sa position (0, 0), puis
/// traite ses messages jusqu'à déconnexion.
///
/// Attribuer l'id et ajouter la position doivent se faire sous le *même*
/// verrou : les séparer permettrait à deux connexions simultanées de lire
/// la même longueur de `Vec` avant que l'une n'ait poussé sa position, et
/// de se retrouver avec le même id.
fn handle_client(mut stream: TcpStream, squares: Squares) {
    let id = {
        let mut squares = squares.lock().unwrap();
        squares.push(Position::new(0.0, 0.0));
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

            squares.lock().unwrap()[id] = Position::new(x, y);
            true
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
    fn assigns_sequential_ids_and_tracks_moves() {
        let address = start_server();

        let mut client_a = TcpStream::connect(address).unwrap();
        let mut reader_a = BufReader::new(client_a.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_a), "ID 0");

        let mut client_b = TcpStream::connect(address).unwrap();
        let mut reader_b = BufReader::new(client_b.try_clone().unwrap());
        assert_eq!(read_line(&mut reader_b), "ID 1");

        // MOVE puis LIST sur la MÊME connexion : le serveur traite les
        // lignes d'un client dans l'ordre où elles arrivent, donc attendre
        // la réponse à ce LIST garantit que le MOVE a déjà été appliqué à
        // l'état partagé avant que le client B n'interroge à son tour —
        // sans ça, rien ne garantirait l'ordre entre deux connexions
        // différentes.
        writeln!(client_a, "MOVE 5 7").unwrap();
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
        assert!(lines.contains(&"1 0 0".to_string()));
    }
}
