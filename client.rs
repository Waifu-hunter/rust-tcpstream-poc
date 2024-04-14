// File: client.rs
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn read_server_response(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    println!("Server closed the connection");
                    break;
                }
                println!("{}", String::from_utf8_lossy(&buf[..n]));
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        }
    }
}

fn connect_to_server() -> io::Result<TcpStream> {
    // tcp stream using native tls
    TcpStream::connect("127.0.0.1:8080")
}

fn reconnect_loop(mut stream: TcpStream) {
    loop {
        thread::sleep(Duration::from_secs(5)); // Wait for 5 seconds

        // Attempt to reconnect if the server closed the connection
        if let Err(_) = stream.write(&[0]) {
            println!("Reconnecting to server...");
            match connect_to_server() {
                Ok(new_stream) => {
                    stream = new_stream;
                    println!("Reconnected to server!");
                }
                Err(e) => {
                    eprintln!("Failed to reconnect: {}", e);
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut stream = connect_to_server()?;
    println!("Connected to server!");

    let cloned_stream = stream.try_clone().expect("Failed to clone stream");
    let cloned_stream2 = stream.try_clone().expect("Failed to clone stream");

    // Spawn a separate thread to continuously read server responses
    let _ = thread::spawn(move || read_server_response(cloned_stream));

    // Spawn a background task to manage reconnections
    thread::spawn(move || reconnect_loop(cloned_stream2));

    // Main thread sends messages to the server
    let mut input = String::new();
    loop {
        io::stdin().read_line(&mut input)?;
        stream.write_all(input.as_bytes())?;
        input.clear();
    }
}