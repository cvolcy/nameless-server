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
    default: String,

    #[arg(long, default_value_t = false)]
    verbose: bool
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

static ASSETS_DATA: [(&'static str, &'static str); 3] = [
    ("./assets/index.html", include_str!("../src/assets/index.html")),
    ("./assets/404.html", include_str!("../src/assets/404.html")),
    ("./assets/w3.css", include_str!("../src/assets/w3.css"))
];

fn handle_connection(mut stream: TcpStream, default_filename: &str) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(result) => result.unwrap(),
        _ => String::new()
    };

    println!("[{}] {}", Utc::now().to_rfc3339(), request_line);

    let (method, filename, http_version) = parse_request_line(request_line, default_filename);

    let mut status = format!("HTTP/{} 200 OK", http_version);
    let contents = match method.as_str() {
        "GET" => {
            get_file_contents(&filename).unwrap_or_else(|_| {
                status = format!("HTTP/{} 404 NOT FOUND", http_version);
                match get_file_contents("404") {
                    Ok(content) => content,
                    Err(_) => String::from(ASSETS_DATA.get(1).unwrap().1)
                }
            })
        },
        _ => String::from(ASSETS_DATA.get(1).unwrap().1)
    };

    let length = contents.len();

    let response =
        format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn get_file_contents(base_filename: &str) -> Result<String, std::io::Error> {
    let paths_to_check = [
        PathBuf::from(format!("./src/assets/{}", base_filename)),
        PathBuf::from(format!("./{}", base_filename)),
        PathBuf::from(format!("./{}.html", base_filename)),
    ];

    for path in paths_to_check.iter() {
        if path.exists() {
            return fs::read_to_string(path);
        }
    }

    // If not found on filesystem, check ASSETS_DATA
    let asset_data_key = format!("./assets/{}", base_filename);
    if let Some((_, content)) = ASSETS_DATA.iter().find(|(key, _)| *key == asset_data_key) {
        return Ok(content.to_string());
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found in any specified path or embedded assets"))
}

fn parse_request_line(request_line: String, default_filename: &str) -> (String, String, String) {
    let re = Regex::new(r"(GET|POST|PUT|DELETE|HEAD|CONNECT|OPTIONS|TRACE|PATCH)\s(/[^?\s]*)\sHTTP/(1\.1|2|3)").unwrap();
    let caps = re.captures(&request_line);
    
    let method: String;
    let http_version: String;

    let mut path = match caps {
        Some(caps) => {
            method = caps.get(1).map_or(String::from("GET"), |m| m.as_str().to_string());
            http_version = caps.get(3).map_or(String::from("1.1"), |m| m.as_str().to_string());
            caps.get(2).map_or("/", |m| m.as_str()).to_string()
        },
        None => return (String::from("GET"), "404.html".to_string(), String::from("1.1")),
    };

    if path == "/" {
        return (method, default_filename.to_string(), http_version);
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

    (method, path, http_version)
}
