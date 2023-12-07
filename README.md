# A minimalistic CGI-capable HTTP server in Rust

This is a simple HTTP server with support for CGI programs written in Rust. It is a side project of mine and an opportunity to put my Rust learning to the test. It is very limited, just for fun and not intended to be used in a production environment :)

The [HTTP server implementation](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) from the [Rust book](https://doc.rust-lang.org/book/) has been used as a starter code, and the `ThreadPool` implementation is exactly the same.

## Dependencies

To build this server, Rust should be [installed](https://www.rust-lang.org/tools/install).

## Running

To run the server:

```
cargo run --release
```

To run in debug mode with logging:

```
RUST_LOG=debug cargo run
```

The server will be listening on port 8080.

## Usage

The static files are stored in the `public_html` folder and will be served at the root of the domain. The CGI executables are stored in the `cgi-bin` folder and will be server at the `/cgi-bin/` path of the domain. The user running the server binary should have execution permissions for the files in this folder (otherwise an HTTP 500 error will be returned). The server will listen on port 8080 by default. All these parameters can be changed by changing the corresponding constants in the `src/main.rs` file.

See the files in the `cgi-bin` for some examples on how to write a CGI program.

## CGI server specifications

### Implemented Metavariables

The following CGI variables are implemented and sent to the CGI program as environment variables. See section 4.1 (Request Meta-Variables) on the [CGI RFC](https://datatracker.ietf.org/doc/html/rfc3875) for more information.

- AUTH_TYPE
- CONTENT_LENGTH
- CONTENT_TYPE
- GATEWAY_INTERFACE
- PATH_INFO
- PATH_TRANSLATED
- QUERY_STRING
- REMOTE_ADDR
- REMOTE_HOST
- REMOTE_IDENT
- REMOTE_USER
- REQUEST_METHOD
- SCRIPT_NAME
- SERVER_NAME
- SERVER_PORT
- SERVER_PROTOCOL
- SERVER_SOFTWARE

### Implemented response headers

The following CGI response headers are accepted from the CGI program output. Any other headers are ignored. See section 6 (CGI Response) on the [CGI RFC](https://datatracker.ietf.org/doc/html/rfc3875) for more information.

- CONTENT_TYPE
- LOCATION
- STATUS

The response type will be inferred from the returned headers, and can be a **document response**, **local redirect response** or **client redirect response**. **Client redirect responses with document** are not supported. Information on the types of CGI responses can also be found on section 6 of the CGI RFC.


## Limitations

### UTF-8 only

Data coming from a TCP connection is expected to be valid UTF-8. That unfortunately means no file uploads. Invalid UTF-8 data coming from the TCP stream will cause the server to respond with a **400 Bad Request** response.

### Buffer size

An 8KB buffer is used to store data received from the incoming streams. Sending more than that to the server will cause it to respond with a **413 Payload Too Large** response. This buffer size can be changed in the `src/http_server/request/request.rs` file by changing the value of the `BUFFER_SIZE` constant.

### Local redirect responses can only redirect to static resources

A CGI response indicating a local redirect will always expect the destination to be a static resource.

### Client redirect response with document is not supported

The **Client redirect response with document** is not supported by this server implementation.

