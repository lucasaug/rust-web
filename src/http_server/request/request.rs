use std::{io::prelude::*, net::TcpStream};

use http::{Request, Response, StatusCode, Version};

use log::debug;

const BUFFER_SIZE: usize = 8 * 1024; // 8KB

pub trait RequestHandler<T> {
    fn handle_request(&self, stream: &TcpStream, request: &Request<T>) -> Option<Response<T>>;
}

pub fn load_request(mut stream: &TcpStream) -> Result<Request<String>, StatusCode> {
    let mut buffer = [0; BUFFER_SIZE + 1];
    if let Err(_) = stream.read(&mut buffer) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let request_string = if let Ok(text) = String::from_utf8(buffer.to_vec()) {
        text.trim_end_matches(char::from(0)).to_owned()
    } else {
        debug!("Error reading UTF-8 from request buffer");
        return Err(StatusCode::BAD_REQUEST);
    };

    if request_string.len() > BUFFER_SIZE {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let mut lines_iter = request_string.lines();
    let start_line = if let Some(line_result) = lines_iter.next() {
        line_result
    } else {
        debug!("No data received");
        return Err(StatusCode::BAD_REQUEST);
    };

    let split_line: Vec<&str> = start_line.split_whitespace().collect();
    if split_line.len() != 3 {
        debug!("Invalid start line_result length");
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut request = Request::builder().method(split_line[0]).uri(split_line[1]);

    let http_version = match split_line[2] {
        "HTTP/0.9" => Version::HTTP_09,
        "HTTP/1.0" => Version::HTTP_10,
        "HTTP/1.1" => Version::HTTP_11,
        "HTTP/2" => Version::HTTP_2,
        "HTTP/3" => Version::HTTP_3,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    request = request.version(http_version);

    loop {
        let next_line = if let Some(line_result) = lines_iter.next() {
            line_result
        } else {
            return match request.body(String::from("")) {
                Err(_) => {
                    debug!("Request ended at metadata section");
                    Err(StatusCode::BAD_REQUEST)
                }
                Ok(request) => Ok(request),
            };
        };

        if next_line.is_empty() {
            break;
        }

        let split_line = next_line.split_once(":");
        request = match split_line {
            None => {
                debug!("Invalid header line_result format");
                return Err(StatusCode::BAD_REQUEST);
            }
            Some((before, after)) => request.header(before, after),
        }
    }

    let mut body = String::from("");
    while let Some(line_entry) = lines_iter.next() {
        body.push_str(&line_entry);
    }

    match request.body(body) {
        Err(_) => {
            debug!("Malformed request");
            Err(StatusCode::BAD_REQUEST)
        }
        Ok(request) => Ok(request),
    }
}
