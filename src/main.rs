use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf
};
use chrono::Utc;
use clap::Parser;
use nameless_server::ThreadPool;
use regex::Regex;

#[derive(Parser)]
#[derive(Debug)]
#[command(version, about, long_about = None)]
struct Args {

    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = 7878)]
    port: u16,

    #[arg(short('n'), long, default_value_t = 4)]
    pool: usize,

    #[arg(short('d'), long, default_value_t = String::from("index.html"))]
    default: String
}

fn main() {
    let cli = Args::parse();
    println!("{:?}", cli);
    
    start_http_server(cli);
}

fn start_http_server(cli: Args) {
    let parsed_addr = format!("0.0.0.0:{}", cli.port);
    let listener = TcpListener::bind(&parsed_addr).unwrap();
    println!("server running at http://{parsed_addr}");

    let pool = ThreadPool::new(cli.pool);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let default_filename = cli.default.clone();
        pool.execute(move || {
            handle_connection(stream, &default_filename);
        })
    }
}

fn handle_connection(mut stream: TcpStream, default_filename: &str) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(result) => result.unwrap(),
        _ => String::new()
    };

    println!("[{}] {}", Utc::now().to_rfc3339(), request_line);

    let filename = get_filename(request_line, default_filename);

    let mut status = "HTTP/1.1 200 OK";
    let contents = get_file_contents(&filename).unwrap_or_else(|_| {
        status = "HTTP/1.1 404 NOT FOUND";
        get_file_contents("404").unwrap()
    });
    let length = contents.len();

    let response =
        format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn get_file_contents(base_filename: &str) -> Result<String, std::io::Error> {
    let possible_paths = [
        PathBuf::from(base_filename),
        PathBuf::from(format!("./src/assets/{}.html", base_filename)),
        PathBuf::from(format!("./src/assets/{}", base_filename)),
        PathBuf::from(format!("./{}.html", base_filename)),
    ];

    for path in possible_paths.iter() {
        if path.exists() {
            return fs::read_to_string(path);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found in any specified path"))
}

fn get_filename(request_line: String, default_filename: &str) -> String {
    let re = Regex::new(r"GET\s(/[^?\s]*)\sHTTP/1\.1").unwrap();
    let caps = re.captures(&request_line);

    let mut path = match caps {
        Some(caps) => caps.get(1).map_or("/", |m| m.as_str()).to_string(),
        None => return "404.html".to_string(),
    };

    if path == "/" {
        return default_filename.to_string();
    }

    if path.starts_with('/') {
        path.remove(0);
    }

    if path.ends_with('/') {
        path.pop();
    }

    let has_extension = path.contains('.');

    if !has_extension {
        path.push_str(".html");
    }

    path
}
