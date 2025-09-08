# nameless-server

`nameless-server` is a simple HTTP server written in Rust. It can serve static files and is configurable via command-line arguments.

## Features

*   Serves static files (HTML, CSS)
*   Configurable port, thread pool size, and default file
*   Basic 404 error handling

## Command Line Arguments

The server can be configured using the following command-line arguments:

*   `-p`, `--port <PORT>`
    *   Sets the port for the server to listen on.
    *   Type: `u16` (unsigned 16-bit integer)
    *   Default: `7878`
    *   Example: `--port 8080`

*   `-n`, `--pool <POOL>`
    *   Sets the number of threads in the thread pool used to handle incoming connections.
    *   Type: `usize` (unsigned integer, platform-dependent size)
    *   Default: `4`
    *   Example: `--pool 8`

*   `-d`, `--default <DEFAULT>`
    *   Sets the default filename to serve when the root path `/` is requested.
    *   Type: `String`
    *   Default: `index.html`
    *   Example: `--default home.html`

*   `--verbose`
    *   Enables verbose output, printing the parsed command-line arguments at startup.
    *   Type: `boolean` (flag)
    *   Default: `false`
    *   Example: `--verbose`

## Usage Example

To run the server on port `8080` with a thread pool of `8` and `default.html` as the default file, with verbose output:

```bash
cargo run -- --port 8080 --pool 8 --default default.html --verbose
```

To run with default settings:

```bash
cargo run
