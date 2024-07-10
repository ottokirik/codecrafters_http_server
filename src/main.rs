// Uncomment this block to pass the first stage
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use http_server_starter_rust::ThreadPool;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                pool.execute(|| handle_connect(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connect(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).expect("");

    let request_string = String::from_utf8_lossy(&buffer[0..bytes_read]);
    let request_vec = request_string.split("\r\n").collect::<Vec<_>>();

    let request_line = request_vec.first().unwrap();

    let requested_resource = request_line.split_whitespace().nth(1).unwrap();

    let response = match request_line {
        s if s.starts_with("GET / ") => "HTTP/1.1 200 OK\r\n\r\n".to_owned(),

        s if s.starts_with("GET /echo/") => {
            let word = requested_resource.split("/").nth(2).unwrap();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                word.len(),
                word
            )
        }

        s if s.starts_with("GET /user-agent") => {
            let user_agent_line = request_vec
                .iter()
                .find(|str| str.starts_with("User-Agent:"))
                .unwrap_or(&"User-Agent: Unknown");
            let user_agent = user_agent_line.split_whitespace().nth(1).unwrap();

            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                user_agent
            )
        }

        s if s.starts_with("GET /files") => {
            let file_name = requested_resource.split("/").nth(2).unwrap();
            let env_args: Vec<_> = env::args().collect();
            let dir = env_args.get(2).unwrap();
            let path = format!("{}{}", dir, file_name);

            let request = match fs::read_to_string(path) {
                Ok(content) => format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                    content.len(),
                    content
                ),
                Err(_) => "HTTP/1.1 404 Not Found\r\n\r\n".to_owned()
            };

            request
        }

        s if s.starts_with("POST /files") => {
            let file_name = requested_resource.split("/").nth(2).unwrap();
            let env_args: Vec<_> = env::args().collect();
            let dir = env_args.get(2).unwrap();
            let mut file = File::create(format!("{}{}", dir, file_name)).unwrap();
            let content = request_vec.last().unwrap();

            file.write_all(content.as_bytes()).unwrap();

            format!("HTTP/1.1 201 Created\r\n\r\n")
        }

        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_owned(),
    };

    stream.write_all(response.as_bytes()).unwrap();
}
