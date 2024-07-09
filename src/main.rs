// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::spawn,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let thr = spawn(move || {
                    let mut buffer = [0; 1024];
                    let bytes_read = stream.read(&mut buffer).expect("");

                    let request_string = String::from_utf8_lossy(&buffer[0..bytes_read]);
                    let request_vec = request_string.split("\r\n").collect::<Vec<_>>();

                    let request_line = request_vec
                        .iter()
                        .find(|str| str.starts_with("GET"))
                        .unwrap();

                    let requested_resource = request_line.split(" ").nth(1).unwrap();

                    let response = match requested_resource {
                        "/" => "HTTP/1.1 200 OK\r\n\r\n".to_owned(),

                        s if s.starts_with("/echo/") => {
                            let word = s.split("/").nth(2).unwrap();
                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", word.len(), word)
                        }

                        s if s.starts_with("/user-agent") => {
                            let user_agent_line = request_vec
                                .iter()
                                .find(|str| str.starts_with("User-Agent:"))
                                .unwrap_or(&"User-Agent: Unknown");
                            let user_agent = user_agent_line.split(" ").nth(1).unwrap();

                            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent)
                        }

                        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_owned(),
                    };

                    stream.write_all(response.as_bytes()).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
