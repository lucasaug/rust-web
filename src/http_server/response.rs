use http::{Response, StatusCode};

pub fn generate_error_response(status_code: StatusCode) -> Response<String> {
    let mut response = Response::new(String::from(""));
    *response.status_mut() = status_code;

    return response;
}

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
