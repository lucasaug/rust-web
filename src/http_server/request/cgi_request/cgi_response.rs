use http::{Request, Response, StatusCode};

use std::{
    collections::HashMap,
    net::TcpStream,
    str::{FromStr, Lines},
};

use log::debug;

use crate::http_server::{
    request::{request::RequestHandler, static_request::static_handler::StaticRequestHandler},
    response::generate_error_response,
};

#[derive(strum_macros::EnumString, Eq, Hash, PartialEq, Debug)]
#[strum(serialize_all = "Train-Case", ascii_case_insensitive)]
pub enum CGIResponseHeader {
    ContentType,
    Location,
    Status,
}

pub type CGIResponseHeaderMap = HashMap<CGIResponseHeader, String>;

#[derive(Debug, PartialEq)]
pub struct CGIScriptResponse {
    headers: CGIResponseHeaderMap,
    body: String,
}

impl CGIScriptResponse {
    fn new(headers: CGIResponseHeaderMap, body: String) -> CGIScriptResponse {
        CGIScriptResponse { headers, body }
    }
}

/// Extracts the CGI headers returned from the CGI script.
///
fn parse_cgi_headers(cgi_output: &mut Lines) -> Result<CGIResponseHeaderMap, ()> {
    let mut headers = CGIResponseHeaderMap::new();

    loop {
        let next_line = if let Some(line_result) = cgi_output.next() {
            line_result
        } else {
            debug!("Malformed CGI response");
            return Err(());
        };

        if next_line.is_empty() {
            break;
        }

        let split_line = next_line.split_once(":");
        match split_line {
            None => {
                debug!("Invalid CGI header");
                return Err(());
            }
            Some((before, after)) => {
                let header_value = CGIResponseHeader::from_str(before);

                if let Ok(header_key) = header_value {
                    headers.insert(header_key, after.trim().to_string());
                } else if let Err(_) = header_value {
                    debug!("Couldn't parse header: {:?}", before);
                }
            }
        }
    }

    Ok(headers)
}

/// Extracts the CGI response from the CGI script output
///
pub fn parse_cgi_response(cgi_output: String) -> Result<CGIScriptResponse, ()> {
    let mut output_lines = cgi_output.lines();
    let response_headers = parse_cgi_headers(&mut output_lines);
    let response_headers = match response_headers {
        Err(_) => return Err(()),
        Ok(headers) => headers,
    };

    let response_body = output_lines.collect::<String>();
    Ok(CGIScriptResponse::new(response_headers, response_body))
}

/// Converts a CGI Local Redirect response into the corresponding HTTP response
///
fn local_redirect(
    stream: &TcpStream,
    static_handler: &StaticRequestHandler,
    location: &str,
) -> Response<String> {
    let static_request = Request::builder()
        .method("GET")
        .uri(location)
        .body(String::from(""));

    match static_request {
        Err(_) => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(static_request) => {
            let response = static_handler.handle_request(stream, &static_request);

            match response {
                None => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
                Some(response) => response,
            }
        }
    }
}

/// Converts a CGI Client Redirect response into the corresponding HTTP
/// response
///
fn client_redirect(location: &str) -> Response<String> {
    let response = Response::builder()
        .status(StatusCode::FOUND)
        .header("location", location)
        .body(String::from(""));

    match response {
        Err(_) => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(response) => response,
    }
}

/// Converts a CGI Document response into the corresponding HTTP response
///
fn document_response(headers: CGIResponseHeaderMap, body: String) -> Response<String> {
    let status = match headers.get(&CGIResponseHeader::Status) {
        None => String::from(StatusCode::OK.as_str()),
        Some(status) => status.clone(),
    };

    let status = match StatusCode::from_str(status.as_str()) {
        Err(_) => return generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(value) => value,
    };

    let content_type = match headers.get(&CGIResponseHeader::ContentType) {
        None => return generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
        Some(value) => value,
    };

    let response = Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body);

    match response {
        Err(_) => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(response) => response,
    }
}

/// Converts a CGI response into the corresponding HTTP response. The type of
/// CGI response is inferred by the CGI headers present in the CGI script
/// output.
///
pub fn convert_cgi_response_to_http(
    stream: &TcpStream,
    static_handler: &StaticRequestHandler,
    cgi_response: CGIScriptResponse,
) -> Response<String> {
    let response_headers = cgi_response.headers;
    let response_body = cgi_response.body;

    if response_headers.contains_key(&CGIResponseHeader::Location) {
        let location = &response_headers[&CGIResponseHeader::Location];
        if location.starts_with("/") {
            local_redirect(stream, static_handler, location)
        } else {
            client_redirect(location)
        }
    } else {
        document_response(response_headers, response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgi_response_is_properly_parsed() {
        let mock_cgi_output = String::from("\
            Content-Type: text/html\n\n\
            Hello!\
        ");
        let result = parse_cgi_response(mock_cgi_output);

        let cgi_response = result.unwrap();

        let expected_headers = CGIResponseHeaderMap::from([
            (CGIResponseHeader::ContentType, String::from("text/html")),
        ]);

        let expected = CGIScriptResponse::new(
            expected_headers,
            String::from("Hello!")
        );

        assert_eq!(cgi_response, expected);
    }
}
