use std::fs::File;
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
        #[arg(long, short)]
        file: PathBuf,
    },

    Listen {
        #[arg(long, short)]
        port: u16,
        #[arg(long, short)]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Send { port, file } => {
            let mut stream =
                TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");

            let filename = file
                .file_name()
                .expect("Failed to extract filename")
                .to_string_lossy()
                .into_owned();

            // Send the filename to the server
            stream.write_all(filename.as_bytes()).expect("Failed to send filename");

            let mut file = File::open(file).expect("Failed to open file");

            let mut buffer = [0; 512];
            loop {
                match file.read(&mut buffer) {
                    Ok(0) => break, // End of file
                    Ok(n) => {
                        stream.write_all(&buffer[..n]).expect("Failed to send data");
                    }
                    Err(e) => {
                        println!("Failed to read data from file: {:?}", e);
                        break;
                    }
                }
            }
        }
        Command::Listen { port, file } => {
            let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");

            println!("Server listening on 0.0.0.0:8080...");

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("Accepted connection from: {:?}", stream.peer_addr());

                        let mut buffer = [0; 512];
                        match stream.read(&mut buffer) {
                            Ok(bytes_read) => {
                                let filename = String::from_utf8_lossy(&buffer[..bytes_read]);
                                let _ = fs::create_dir("./received_files");
                                let file_path = Path::new("./received_files").join(filename.trim());
                                dbg!(&file_path);

                                let mut file = File::create(&file_path).expect("Failed to create file");

                                // Receive and write the file contents
                                loop {
                                    match stream.read(&mut buffer) {
                                        Ok(0) => break, // End of file
                                        Ok(n) => {
                                            file.write_all(&buffer[..n]).expect("Failed to write data");
                                        }
                                        Err(e) => {
                                            println!("Failed to read data: {:?}", e);
                                            break;
                                        }
                                    }
                                }

                                println!("Received file: {:?}", file_path);
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
