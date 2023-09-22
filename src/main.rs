use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{
    collections::HashSet,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Send {
        #[arg(long, short)]
        port: u16,
    },

    Listen {
        #[arg(long, short)]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Send { port } => {
            let mut stream =
                TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");

            let message = "Hello, server!";
            stream
                .write_all(message.as_bytes())
                .expect("Failed to send data");

            let mut buffer = [0; 512];
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    println!(
                        "Received {} bytes from server: {:?}",
                        bytes_read,
                        &buffer[..bytes_read]
                    );
                }
                Err(e) => {
                    println!("Failed to read data from server: {:?}", e);
                }
            }
        }
        Command::Listen { port } => {
            let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");

            println!("Server listening on 0.0.0.0:8080...");

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("Accepted connection from: {:?}", stream.peer_addr());

                        let mut buffer = [0; 512];
                        match stream.read(&mut buffer) {
                            Ok(bytes_read) => {
                                let s = String::from_utf8(buffer[..bytes_read].into()).unwrap();
                                dbg!(s);
                                println!(
                                    "Received {} bytes: {:?}",
                                    bytes_read,
                                    &buffer[..bytes_read]
                                );
                                stream
                                    .write_all(&buffer[..bytes_read])
                                    .expect("Failed to write data");
                            }
                            Err(e) => {
                                println!("Failed to read data: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to establish connection: {:?}", e);
                    }
                }
            }
        }
    }
}
