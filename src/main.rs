use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use chrono::Utc;
use clap::Parser;
use nameless_server::ThreadPool;
use regex::Regex;
use lazy_static::lazy_static;

const DEFAULT_PORT: u16 = 7878;
const DEFAULT_POOL_SIZE: usize = 4;
const DEFAULT_FILENAME: &str = "index.html";
const NOT_FOUND_FILENAME: &str = "404.html";
const HTTP_VERSION_1_1: &str = "1.1";
const HTTP_STATUS_OK: &str = "200 OK";
const HTTP_STATUS_NOT_FOUND: &str = "404 NOT FOUND";

#[derive(Parser)]
#[derive(Debug)]
#[command(version, about, long_about = None)]
struct Args {

    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = DEFAULT_PORT)]
    port: u16,

    #[arg(short('n'), long, default_value_t = DEFAULT_POOL_SIZE)]
    pool: usize,

    #[arg(short('d'), long, default_value_t = String::from(DEFAULT_FILENAME))]
    default: String,

    #[arg(long, default_value_t = false)]
    verbose: bool
}

fn main() {
    let cli = Args::parse();

    if cli.verbose {
        println!("{:?}", cli);
    }

    start_http_server(cli);
}

fn start_http_server(cli: Args) {
    let parsed_addr = format!("0.0.0.0:{}", cli.port);
    let listener = TcpListener::bind(&parsed_addr).unwrap();
    println!("server running at http://{parsed_addr}\n");

    let pool = ThreadPool::new(cli.pool);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let default_filename = cli.default.clone();
        pool.execute(move || {
            handle_connection(stream, &default_filename);
        })
    }
}

lazy_static! {
    static ref ASSETS_DATA: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("index.html", include_str!("../src/assets/index.html"));
        map.insert("404.html", include_str!("../src/assets/404.html"));
        map.insert("w3.css", include_str!("../src/assets/w3.css"));
        map
    };
}

fn handle_connection(mut stream: TcpStream, default_filename: &str) {
    let request_line = read_request_line(&mut stream);
    println!("[{}] Request   - {}", Utc::now().to_rfc3339(), request_line);

    let (method, filename, http_version) = parse_request_line(request_line, default_filename);

    let (status, contents) = get_response_content(&filename, &method, &http_version);

    let length = contents.len();
    println!("[{}]  Response - {} {}", Utc::now().to_rfc3339(), status, length);

    let response = build_response(&status, length, &contents);
    send_response(&mut stream, &response);
}

fn read_request_line(stream: &mut TcpStream) -> String {
    let buf_reader = BufReader::new(stream);
    buf_reader
        .lines()
        .next()
        .map_or_else(String::new, |result| result.unwrap_or_else(|_| String::new()))
}

fn get_response_content(filename: &str, method: &str, http_version: &str) -> (String, String) {
    let mut status = format!("HTTP/{} {}", http_version, HTTP_STATUS_OK);
    let contents = match method {
        "GET" => {
            get_file_contents(filename).unwrap_or_else(|_| {
                status = format!("HTTP/{} {}", http_version, HTTP_STATUS_NOT_FOUND);
                match get_file_contents(NOT_FOUND_FILENAME) {
                    Ok(content) => content,
                    Err(_) => String::from(*ASSETS_DATA.get(NOT_FOUND_FILENAME).unwrap())
                }
            })
        },
        _ => String::from(*ASSETS_DATA.get(NOT_FOUND_FILENAME).unwrap())
    };
    (status, contents)
}

fn build_response(status: &str, length: usize, contents: &str) -> String {
    format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}")
}

fn send_response(stream: &mut TcpStream, response: &str) {
    stream.write_all(response.as_bytes()).unwrap();
}

fn get_file_contents(base_filename: &str) -> Result<String, std::io::Error> {
    // Check embedded assets first
    if let Some(content) = ASSETS_DATA.get(base_filename) {
        return Ok(content.to_string());
    }

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

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found in any specified path or embedded assets",
    ))
}

fn parse_request_line(request_line: String, default_filename: &str) -> (String, String, String) {
    let re = Regex::new(r"(GET|POST|PUT|DELETE|HEAD|CONNECT|OPTIONS|TRACE|PATCH)\s(/[^?\s]*)\sHTTP/(1\.1|2|3)").unwrap();
    let caps = re.captures(&request_line);
    
    let method: String;
    let http_version: String;

    let path_str = match caps {
        Some(caps) => {
            method = caps.get(1).map_or(String::from("GET"), |m| m.as_str().to_string());
            http_version = caps.get(3).map_or(String::from(HTTP_VERSION_1_1), |m| m.as_str().to_string());
            caps.get(2).map_or("/", |m| m.as_str()).to_string()
        },
        None => return (String::from("GET"), NOT_FOUND_FILENAME.to_string(), String::from(HTTP_VERSION_1_1)),
    };

    if path_str == "/" {
        return (method, default_filename.to_string(), http_version);
    }

    let mut path_buf = PathBuf::from(path_str.trim_start_matches('/'));

    if path_buf.extension().is_none() {
        path_buf.set_extension("html");
    }

    (method, path_buf.to_str().unwrap_or(NOT_FOUND_FILENAME).to_string(), http_version)
}
