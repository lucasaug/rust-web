use http::{Response, StatusCode};

/// Generates an empty HTTP response with a given status code
///
pub fn generate_error_response(status_code: StatusCode) -> Response<String> {
    let mut response = Response::new(String::from(""));
    *response.status_mut() = status_code;

    return response;
}

/// Converts a structured HTTP response into the text data to be sent back to
/// the requesting client
///
/// # Panics
///
/// The `response_to_string` function will panic if the given response has any
/// headers that can't be converted to a string.
///
pub fn response_to_string(response: Response<String>) -> String {
    let status = response.status();
    let status_value = status.as_str();
    let reason = status.canonical_reason().unwrap_or("");
    let status_line = format!("HTTP/1.1 {status_value} {reason}");

    let mut header_line = String::from("");
    for (header_name, header_value) in response.headers() {
        let header_value = header_value.to_str().expect("Invalid response header");
        let header = format!("{header_name}: {header_value}\r\n");
        header_line.push_str(&header);
    }

    let contents = response.body();

    format!("{status_line}\r\n{header_line}\r\n{contents}")
}
