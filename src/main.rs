use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread
};
use chrono::Utc;
use clap::Parser;

#[derive(Parser)]
#[derive(Debug)]
#[command(version, about, long_about = None)]
struct Args {

    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = 7878)]
    port: u16
}

fn main() {
    let cli = Args::parse();
    println!("{:?}", cli);
    
    let parsed_addr = format!("0.0.0.0:{}", cli.port);
    start_http_server(parsed_addr);
}

fn start_http_server(addr: String) {
    let listener = TcpListener::bind(&addr).unwrap();
    println!("server running at http://{addr}");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(result) => result.unwrap(),
        _ => String::new()
    };

    println!("[{}] {}", Utc::now().to_rfc3339(), request_line);

    let (status, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(format!("./src/assets/{}", filename)).unwrap();
    let length = contents.len();

    let response =
        format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
