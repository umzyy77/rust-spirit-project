use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use common::Position;

/// Connexion au serveur : `stream` pour écrire (`MOVE`/`LIST`), `reader`
/// (via `try_clone`) pour lire ses réponses.
pub struct Connection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    id: usize,
}

impl Connection {
    /// Se connecte à `address`, puis lit l'`id` attribué par le serveur.
    pub fn connect(address: &str) -> io::Result<Connection> {
        let stream = TcpStream::connect(address)?;
        // Messages courts et fréquents : désactive Nagle pour éviter ~40ms de pause.
        stream.set_nodelay(true)?;
        let reader = BufReader::new(stream.try_clone()?);
        let mut connection = Connection { stream, reader, id: 0 };
        connection.id = connection.read_id()?;
        Ok(connection)
    }

    /// Identifiant que le serveur a attribué à notre carré à la connexion.
    pub fn id(&self) -> usize {
        self.id
    }

    fn read_id(&mut self) -> io::Result<usize> {
        let line = self.read_line()?;
        line.strip_prefix("ID ")
            .and_then(|id| id.parse().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("\"ID <n>\" attendu a la connexion, recu \"{line}\"")))
    }

    /// Demande au serveur de nous déplacer en `position`. Le serveur fait foi sur
    /// les collisions : renvoie la position réellement acceptée (`POS x y`), à
    /// afficher même si elle diffère de celle demandée.
    pub fn send_move(&mut self, position: Position) -> io::Result<Position> {
        writeln!(self.stream, "MOVE {} {}", position.x(), position.y())?;

        let line = self.read_line()?;
        let coordinates = line
            .strip_prefix("POS ")
            .and_then(|rest| rest.split_once(' '))
            .and_then(|(x, y)| x.parse().ok().zip(y.parse().ok()));

        let Some((x, y)) = coordinates else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("\"POS <x> <y>\" attendu apres MOVE, recu \"{line}\"")));
        };

        Ok(Position::new(x, y))
    }

    /// Demande la position de tous les carrés connus du serveur.
    pub fn list(&mut self) -> io::Result<Vec<(usize, Position)>> {
        writeln!(self.stream, "LIST")?;

        let mut squares = Vec::new();
        loop {
            let line = self.read_line()?;
            if line == "END" {
                return Ok(squares);
            }

            let mut parts = line.split_whitespace();
            let coordinates = parts.next().and_then(|id| id.parse().ok()).zip(
                parts
                    .next()
                    .and_then(|x| x.parse().ok())
                    .zip(parts.next().and_then(|y| y.parse().ok())),
            );

            let Some((id, (x, y))) = coordinates else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("ligne LIST invalide : \"{line}\"")));
            };

            squares.push((id, Position::new(x, y)));
        }
    }

    /// Lit une ligne texte, sans le saut de ligne final. `UnexpectedEof` si
    /// le serveur a fermé la connexion (`read_line` renvoie alors `Ok(0)`).
    fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "le serveur a ferme la connexion"));
        }

        Ok(line.trim_end().to_string())
    }
}

/// Une demande envoyée au thread réseau par la boucle de jeu.
pub enum Command {
    Move(Position),
    RequestList,
}

/// Une réponse du thread réseau, consommée (non bloquant) par la boucle de jeu.
pub enum NetEvent {
    Moved(Position),
    List(Vec<(usize, Position)>),
}

/// Thread dédié aux appels réseau bloquants, pour ne jamais geler le rendu.
pub fn spawn(mut connection: Connection) -> (Sender<Command>, Receiver<NetEvent>) {
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();

    thread::spawn(move || {
        for command in command_rx {
            let result = match command {
                Command::Move(position) => connection.send_move(position).map(NetEvent::Moved),
                Command::RequestList => connection.list().map(NetEvent::List),
            };

            match result {
                Ok(event) => {
                    if event_tx.send(event).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("Erreur reseau : {err}");
                    break;
                }
            }
        }
    });

    (command_tx, event_rx)
}
