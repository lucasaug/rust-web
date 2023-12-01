use std::{io::prelude::*, net::TcpStream};

use http::{Request, Response, StatusCode};

use log::{debug, info};

use crate::http_server::{
    request::request::{load_request, RequestHandler},
    response::generate_error_response,
    response::response_to_string,
};

type RequestHandlerList = Vec<Box<dyn RequestHandler<String> + Sync + Send>>;

pub struct ConnectionHandler {
    request_handlers: RequestHandlerList,
}

impl ConnectionHandler {
    pub fn new(request_handlers: RequestHandlerList) -> ConnectionHandler {
        ConnectionHandler { request_handlers }
    }

    /// Handles a single incoming HTTP request using a suitable handler.
    ///
    /// Receives the request information as well as the TCP stream from which
    /// the request was read. Handlers supplied to the ConnectionHandler are
    /// tried in order, and the first `Some` response available is returned.
    pub fn handle_request(&self, request: Request<String>, stream: &TcpStream) -> Response<String> {
        let mut response = None;
        for handler in &self.request_handlers {
            response = response.or(handler.handle_request(stream, &request));
            if let Some(_) = response {
                break;
            }
        }

        let mut response =
            response.unwrap_or(generate_error_response(StatusCode::INTERNAL_SERVER_ERROR));

        if request.method() == "HEAD" {
            *response.body_mut() = String::from("");
        }

        response
    }

    /// Reads an HTTP request from a TCP stream and writes the corresponding
    /// response back to it.
    ///
    /// # Panics
    ///
    /// The `handle_connection` method panics if the stream can't be written to
    /// or flushed (for instance if it was closed).
    ///
    pub fn handle_connection(&self, mut stream: TcpStream) {
        info!("New request received");
        let request = load_request(&mut stream);
        debug!("{:?}", request);

        let response = match request {
            Ok(request) => self.handle_request(request, &stream),
            Err(status) => generate_error_response(status),
        };

        let response_text = response_to_string(response);
        info!("Writing response");
        debug!("Response: \n{response_text}\n");

        stream
            .write_all(response_text.as_bytes())
            .expect("Error writing response to the TCP Stream");
        stream.flush().expect("Error flushing TCP stream");

        info!("Finished writing response");
    }
}
