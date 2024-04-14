// File: server.rs
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

fn handle_client(mut stream: TcpStream, client_uuid: Uuid, clients: Arc<Mutex<HashMap<Uuid, TcpStream>>>) {
    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(n) if n > 0 => {
                if n == 1 {
                    // Ignore empty messages
                    continue;
                }
                println!("Received {} bytes from client {}", n, client_uuid);
                // show the uuid of the client and the decoded message
                let message = String::from_utf8_lossy(&buf[..n]);
                println!("Received message from client {}", client_uuid);

                // show client uuids list
                let clients = clients.lock().unwrap();
                let client_uuids: Vec<Uuid> = clients.keys().cloned().collect();
                println!("Connected clients: {:?}", client_uuids);

                if message.starts_with("broadcast:") {
                    for (uuid, mut client) in clients.iter() {
                        if let Err(e) = client.write_all(&buf[..n]) {
                            eprintln!("Error writing to client {}: {}", uuid, e);
                        }
                    }
                } else {
                    if let Err(e) = stream.write_all(&buf[..n]) {
                        eprintln!("Error writing to stream: {}", e);
                        break;
                    }
                }
            }
            Ok(_) | Err(_) => {
                // On EOF or error, remove the client from the hashmap
                let mut clients = clients.lock().unwrap();
                clients.remove(&client_uuid);
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080...");

    let clients: Arc<Mutex<HashMap<Uuid, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.expect("Failed to accept connection");
        let client_uuid = Uuid::new_v4();
        println!("New client connected with UUID: {}", client_uuid);
        
        // Store the client in the hashmap
        let clients_clone = Arc::clone(&clients);
        clients.lock().unwrap().insert(client_uuid, stream.try_clone().expect("Failed to clone stream"));
        thread::spawn(move || {
            handle_client(stream, client_uuid, clients_clone);
        });
    }

    Ok(())
}
