use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
        ip: String,
        #[arg(long, short)]
        port: u16,
        #[arg(long, short)]
        file: PathBuf,
        #[arg(long, short)]
        dest_path: PathBuf,
    },

    Listen {
        #[arg(long, short)]
        ip: String,
        #[arg(long, short)]
        port: u16,
    },

    AskQuit {
        #[arg(long, short)]
        ip: String,
        #[arg(long, short)]
        port: u16,
    },

    Scan {
        #[arg(long, short)]
        port: u16,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum Message {
    Hello,
    File { dest_path: PathBuf, data: String },
    Quit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Send { ip, port, file, dest_path } => {
            let mut file = File::open(file).expect("Failed to open file");
            let mut data = String::new();
            file.read_to_string(&mut data)?;

            let message1 = Message::File {
                dest_path,
                data,
            };
            let messages = vec![message1];

            for message in messages.iter() {
                let mut stream = TcpStream::connect(format!("{ip}:{port}"))
                    .expect("Failed to connect to server");

                let json = serde_json::to_string(message)?;

                stream
                    .write_all(json.as_bytes())
                    .expect("Failed to send data");
            }
        }
        Command::AskQuit { ip, port } => {
            let message1 = Message::Quit;
            
            let messages = vec![message1];

            for message in messages.iter() {
                let mut stream = TcpStream::connect(format!("{ip}:{port}"))
                    .expect("Failed to connect to server");

                let json = serde_json::to_string(message)?;

                stream
                    .write_all(json.as_bytes())
                    .expect("Failed to send data");
            }
        }
        Command::Listen { ip, port } => {
            let listener =
                TcpListener::bind(format!("{ip}:{port}")).expect("Failed to bind to address");

            println!("Server listening ...");

            for stream in listener.incoming() {
                let mut stream = stream?;
                println!("Accepted connection from: {:?}", stream.peer_addr());
                let mut message = String::new();
                stream.read_to_string(&mut message)?;
                let message: Message = serde_json::from_str(&message)?;

                match &message {
                    Message::File { dest_path, data } => {
                        println!("writing file {dest_path:?}");

                        fs::create_dir_all(dest_path.parent().unwrap())?;

                        let mut file = File::create(dest_path).expect("Failed to create file");
                        file.write_all(data.as_bytes())?;
                    }
                    Message::Quit => break,
                    Message::Hello => {
                        println!("Received Hello message.");
                    }
                }
            }
        }
        Command::Scan { port: target_port } => {
            let ip = local_ip_address::local_ip()?;
            let ip = ip.to_string();
            let network_addr = &ip[0..ip.len() - ip.chars().rev().position(|c| c == '.').unwrap()];

            let message = serde_json::to_string(&Message::Hello)?;

            let mut handles = Vec::new();
            for i in 1..254 {
                let target_ip = format!("{}{}", network_addr, i);
                let target_addr = format!("{}:{}", target_ip, target_port);

                let message = message.clone();

                let handle = thread::spawn(move || {
                    // println!("trying to connect to {target_addr}");
                    match TcpStream::connect_timeout(
                        &target_addr.to_socket_addrs().unwrap().next().unwrap(),
                        std::time::Duration::from_secs_f32(1.0),
                    ) {
                        Ok(mut stream) => {
                            println!("{target_addr} is online :)");
                            stream
                                .write_all(message.as_bytes())
                                .expect("Failed to send data");
                        }
                        Err(_) => {
                            // println!("could not connect: {target_addr}")
                            // Connection failed, no device at this address
                        }
                    }
                });

                handles.push(handle);
            }

            handles.into_iter().try_for_each(|h| h.join()).unwrap();
        }
    }

    Ok(())
}
