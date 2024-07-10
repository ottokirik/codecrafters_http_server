// Uncomment this block to pass the first stage
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use flate2::{write::GzEncoder, Compression};
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

    let encoding = request_vec
        .iter()
        .find(|str| str.starts_with("Accept-Encoding:"))
        .unwrap_or(&"");

    let is_gzip = encoding.contains("gzip");
    let encoding = match is_gzip {
        true => "gzip",
        false => "",
    };

    let request_line = request_vec.first().unwrap();

    let requested_resource = request_line.split_whitespace().nth(1).unwrap();

    let response: String = match request_line {
        s if s.starts_with("GET / ") => HTTPRequestBuilder::default()
            .add_encoding(encoding)
            .build()
            .into(),

        s if s.starts_with("GET /echo/") => {
            let word = requested_resource.split("/").nth(2).unwrap();

            match is_gzip {
                true => {
                    let (compressed, len) = get_gzip(word.to_owned());

                    let res_str: String = HTTPRequestBuilder::default()
                        .add_encoding(encoding)
                        .add_content_type("text/plain")
                        .add_content_length(len)
                        .build()
                        .into();

                    let buffer = [res_str.as_bytes(), compressed.as_slice()].concat();

                    stream.write_all(buffer.as_slice()).unwrap();

                    return;
                }
                false => HTTPRequestBuilder::default()
                    .add_encoding(encoding)
                    .add_content_type("text/plain")
                    .add_content(word)
                    .add_content_length(word.len())
                    .build()
                    .into(),
            }
        }

        s if s.starts_with("GET /user-agent") => {
            let user_agent_line = request_vec
                .iter()
                .find(|str| str.starts_with("User-Agent:"))
                .unwrap_or(&"User-Agent: Unknown");
            let user_agent = user_agent_line.split_whitespace().nth(1).unwrap();

            HTTPRequestBuilder::default()
                .add_encoding(encoding)
                .add_content_type("text/plain")
                .add_content_length(user_agent.len())
                .add_content(user_agent)
                .build()
                .into()
        }

        s if s.starts_with("GET /files") => {
            let file_name = requested_resource.split("/").nth(2).unwrap();
            let env_args: Vec<_> = env::args().collect();
            let dir = env_args.get(2).unwrap();
            let path = format!("{}{}", dir, file_name);

            match fs::read_to_string(path) {
                Ok(content) => HTTPRequestBuilder::default()
                    .add_encoding(encoding)
                    .add_content_type("application/octet-stream")
                    .add_content_length(content.len())
                    .add_content(content.as_str())
                    .build()
                    .into(),

                Err(_) => HTTPRequestBuilder::default()
                    .add_status("404 Not Found")
                    .add_encoding(encoding)
                    .build()
                    .into(),
            }
        }

        s if s.starts_with("POST /files") => {
            let file_name = requested_resource.split("/").nth(2).unwrap();
            let env_args: Vec<_> = env::args().collect();
            let dir = env_args.get(2).unwrap();
            let mut file = File::create(format!("{}{}", dir, file_name)).unwrap();
            let content = request_vec.last().unwrap();

            file.write_all(content.as_bytes()).unwrap();

            HTTPRequestBuilder::default()
                .add_status("201 Created")
                .add_encoding(encoding)
                .build()
                .into()
        }

        _ => HTTPRequestBuilder::default()
            .add_status("404 Not Found")
            .add_encoding(encoding)
            .build()
            .into(),
    };

    stream.write_all(response.as_bytes()).unwrap();
}

#[derive(Debug)]
struct HTTPRequest(String);

impl From<HTTPRequest> for String {
    fn from(request: HTTPRequest) -> Self {
        request.0
    }
}

struct HTTPRequestBuilder {
    protocol: String,
    status: String,
    content_length: String,
    content_type: String,
    content_encoding: String,
    content: String,
}

impl HTTPRequestBuilder {
    fn default() -> Self {
        Self {
            protocol: "HTTP/1.1".to_owned(),
            status: "200 OK\r\n".to_owned(),
            content_length: "".to_owned(),
            content_type: "".to_owned(),
            content_encoding: "".to_owned(),
            content: "".to_owned(),
        }
    }

    fn build(&self) -> HTTPRequest {
        HTTPRequest(format!(
            "{} {}{}{}{}\r\n{}",
            self.protocol,
            self.status,
            self.content_encoding,
            self.content_type,
            self.content_length,
            self.content
        ))
    }

    fn add_encoding(mut self, encoding: &str) -> Self {
        self.content_encoding = format!("Content-Encoding: {}\r\n", encoding);
        self
    }

    fn add_content_type(mut self, content_type: &str) -> Self {
        self.content_type = format!("Content-Type: {}\r\n", content_type);
        self
    }

    fn add_content_length(mut self, content_length: usize) -> Self {
        self.content_length = format!("Content-Length: {}\r\n", content_length);
        self
    }

    fn add_content(mut self, content: &str) -> Self {
        self.content = content.to_owned();
        self
    }

    fn add_status(mut self, status: &str) -> Self {
        self.status = format!("{}\r\n", status);
        self
    }
}

fn get_gzip(data: String) -> (Vec<u8>, usize) {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    let len = compressed.len();

    (compressed, len)
}
