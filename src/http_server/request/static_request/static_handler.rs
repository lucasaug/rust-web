use std::{fs, net::TcpStream, path::Path};

use http::{Request, Response, StatusCode};

use log::debug;

use crate::http_server::{request::request::RequestHandler, response::generate_error_response};

pub struct StaticRequestHandler {
    static_folder: String,
}

impl StaticRequestHandler {
    pub fn new(static_folder: String) -> StaticRequestHandler {
        StaticRequestHandler { static_folder }
    }
}

impl RequestHandler<String> for StaticRequestHandler {
    /// Handles an incoming request. Always returns a `Some` variant, since the
    /// static handler is kind of a fallback handler. If the requested page
    /// isn't available, it should return a 404 response.
    ///
    /// # Panics
    ///
    /// The `handle_request` method panics if the `self.static_folder` path
    /// does not exist, or if it encounters a problem during the building of
    /// the HTTP response (which shouldn't happen since an incorrect header
    /// would be the only possible problem in this case, and the response
    /// headers for static requests are hard-coded).
    ///
    fn handle_request(
        &self,
        _stream: &TcpStream,
        request: &Request<String>,
    ) -> Option<Response<String>> {
        if request.method() != "GET" && request.method() != "HEAD" {
            return Some(generate_error_response(StatusCode::METHOD_NOT_ALLOWED));
        }

        let uri_path = request.uri().path();
        let file_path = if uri_path == "/" {
            "index.html"
        } else {
            &uri_path[1..] // Remove leading slash
        };

        let static_folder_path =
            fs::canonicalize(&self.static_folder).expect("Static files path does not exist");
        let file_path = fs::canonicalize(Path::new(&self.static_folder).join(file_path));
        let abs_file_path = match file_path {
            Err(_) => return Some(generate_error_response(StatusCode::NOT_FOUND)),
            Ok(path) => {
                if path.starts_with(static_folder_path) {
                    path
                } else {
                    return Some(generate_error_response(StatusCode::NOT_FOUND));
                }
            }
        };

        debug!("Searching for {:?}", abs_file_path);
        let contents = fs::read_to_string(abs_file_path);
        debug!("Read result: {:?}", contents);
        let contents = match contents {
            Err(_) => return Some(generate_error_response(StatusCode::INTERNAL_SERVER_ERROR)),
            Ok(contents) => contents,
        };

        let contents_len = contents.len();
        let sent_content = if request.method() == "HEAD" {
            String::from("")
        } else {
            contents
        };

        Some(
            Response::builder()
                .status(StatusCode::OK)
                .header("content-length", contents_len)
                .body(sent_content)
                .expect("Error generating success response"),
        )
    }
}
