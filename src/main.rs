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
    },

    Listen {
        #[arg(long, short)]
        ip: String,
        #[arg(long, short)]
        port: u16,
        // #[arg(long, short)]
        // file: PathBuf,
    },

}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum Message {
    File { file_name: PathBuf, data: String },
    Quit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Send { ip, port, file } => {
            let filename = file
                .file_name()
                .expect("Failed to extract filename")
                .to_string_lossy()
                .into_owned();
            let mut file = File::open(file).expect("Failed to open file");
            let mut data = String::new();
            file.read_to_string(&mut data)?;

            let message1 = Message::File {
                file_name: filename.into(),
                data,
            };
            let message2 = Message::Quit;
            let messages = vec![message1, message2];

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
                    Message::File { file_name, data } => {
                        let _ = fs::create_dir("./received_files");
                        let file_path = Path::new("./received_files").join(file_name);
                        println!("writing file {file_path:?}");

                        let mut file = File::create(&file_path).expect("Failed to create file");
                        file.write_all(data.as_bytes())?;
                    }
                    Message::Quit => break,
                    Message::Hello => {
                        println!("Received Hello message.");
                    }
                }
            }
        }
    }

    Ok(())
}
