use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream}
};
use chrono::Utc;
use clap::Parser;
use nameless_server::ThreadPool;

#[derive(Parser)]
#[derive(Debug)]
#[command(version, about, long_about = None)]
struct Args {

    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = 7878)]
    port: u16,

    #[arg(short('n'), long, default_value_t = 4)]
    pool: usize
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

        pool.execute(|| {
            handle_connection(stream);
        })
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
