use std::{
    fs,
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use http::{header, HeaderName, Request, Response, StatusCode};

use log::debug;

use crate::http_server::{
    request::{
        cgi_request::{
            cgi_metavariables::CGIMetavariableMap,
            cgi_response::{convert_cgi_response_to_http, parse_cgi_response},
        },
        request::RequestHandler,
        static_request::static_handler::StaticRequestHandler,
    },
    response::generate_error_response,
};

use super::cgi_metavariables::CGIMetavariable;

const DEFAULT_PORT: &str = "80";

pub struct CgiRequestHandler {
    cgi_path: String,
    cgi_folder: String,
    static_handler: StaticRequestHandler,
}

impl CgiRequestHandler {
    pub fn new(
        cgi_path: String,
        cgi_folder: String,
        static_handler: StaticRequestHandler,
    ) -> CgiRequestHandler {
        CgiRequestHandler {
            cgi_path,
            cgi_folder,
            static_handler,
        }
    }
}

fn get_header_or_empty_string(request: &Request<String>, header_name: HeaderName) -> String {
    request
        .headers()
        .get(header_name)
        .map_or(String::from(""), |h| h.to_str().unwrap_or("").to_string())
}

fn run_process(
    script_path: PathBuf,
    input_data: &String,
    env_variables: CGIMetavariableMap,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut parent_folder = script_path.clone();
    parent_folder.pop();
    let mut script_process = Command::new(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .env_clear()
        .current_dir(parent_folder)
        .envs(&env_variables)
        .spawn()?;

    let mut stdin = script_process
        .stdin
        .take()
        .ok_or("Error getting stdin for child process")?;
    stdin.write_all(input_data.as_bytes())?;
    drop(stdin);

    let output_handle = script_process.wait_with_output()?;
    let output = String::from_utf8_lossy(&output_handle.stdout);

    Ok(output.to_string())
}

impl CgiRequestHandler {
    fn generate_environment_variables(
        &self,
        stream: &TcpStream,
        request: &Request<String>,
    ) -> CGIMetavariableMap {
        let mut metavariables = CGIMetavariableMap::new();

        let authorization_data = get_header_or_empty_string(request, header::AUTHORIZATION);
        let authorization_pair = authorization_data.split_once(" ");

        match authorization_pair {
            Some((scheme, parameters)) => {
                metavariables.insert(CGIMetavariable::AuthType, scheme.to_string());
                metavariables.insert(CGIMetavariable::RemoteUser, parameters.to_string());
            }
            None => {
                metavariables.insert(CGIMetavariable::AuthType, String::from(""));
            }
        }

        let content_length = request.body().len();
        metavariables.insert(
            CGIMetavariable::ContentLength,
            if content_length > 0 {
                content_length.to_string()
            } else {
                String::from("")
            },
        );

        let content_type = get_header_or_empty_string(request, header::CONTENT_TYPE);
        if content_type != "" {
            metavariables.insert(CGIMetavariable::ContentType, content_type);
        } else {
            metavariables.insert(
                CGIMetavariable::ContentType,
                String::from("application/octet-stream"),
            );
        }

        metavariables.insert(CGIMetavariable::GatewayInterface, String::from("CGI/1.1"));

        metavariables.insert(CGIMetavariable::PathInfo, String::from(""));
        metavariables.insert(CGIMetavariable::PathTranslated, String::from(""));

        metavariables.insert(
            CGIMetavariable::QueryString,
            request.uri().query().unwrap_or("").to_string(),
        );

        let remote_addr = stream
            .peer_addr()
            .map_or(String::from(""), |addr| addr.ip().to_string());
        metavariables.insert(CGIMetavariable::RemoteAddr, remote_addr.clone());
        metavariables.insert(CGIMetavariable::RemoteHost, remote_addr);
        metavariables.insert(CGIMetavariable::RemoteIdent, String::from(""));

        metavariables.insert(CGIMetavariable::RequestMethod, request.method().to_string());

        metavariables.insert(
            CGIMetavariable::ScriptName,
            request.uri().path().to_string(),
        );

        let host_value = request
            .headers()
            .get(header::HOST)
            .map_or(String::from(""), |host| {
                host.to_str().unwrap_or("").to_string()
            });
        let (server_name, server_port) = match host_value.split_once(":") {
            Some((server_name, server_port)) => (server_name.to_string(), server_port.to_string()),
            None => (host_value, DEFAULT_PORT.to_string()),
        };
        metavariables.insert(CGIMetavariable::ServerName, server_name);
        metavariables.insert(CGIMetavariable::ServerPort, server_port);

        metavariables.insert(CGIMetavariable::ServerProtocol, String::from("HTTP/1.0"));

        metavariables.insert(
            CGIMetavariable::ServerSoftware,
            String::from("Rust Web CGI/0.0.1"),
        );

        return metavariables;
    }

    fn run_cgi_script(
        &self,
        stream: &TcpStream,
        request: &Request<String>,
        script_path: PathBuf,
    ) -> Response<String> {
        let envs = self.generate_environment_variables(stream, request);

        match run_process(script_path, request.body(), envs) {
            Err(_) => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
            Ok(output) => {
                debug!("CGI output: {}", output);
                let cgi_response = parse_cgi_response(output);

                match cgi_response {
                    Err(_) => generate_error_response(StatusCode::INTERNAL_SERVER_ERROR),
                    Ok(cgi_response) => {
                        convert_cgi_response_to_http(stream, &self.static_handler, cgi_response)
                    }
                }
            }
        }
    }
}

impl RequestHandler<String> for CgiRequestHandler {
    fn handle_request(
        &self,
        stream: &TcpStream,
        request: &Request<String>,
    ) -> Option<Response<String>> {
        let uri_path = request.uri().path();
        let file_path = &uri_path[1..].strip_prefix(&self.cgi_path)?;
        let file_path = file_path.strip_prefix("/").unwrap_or(file_path);

        debug!("CGI script to be loaded: {}", file_path);
        let folder_path = fs::canonicalize(&self.cgi_folder).expect("CGI folder does not exist");
        let file_path = fs::canonicalize(Path::new(&self.cgi_folder).join(file_path));
        let abs_file_path = match file_path {
            Err(_) => {
                debug!("No CGI script found");
                return Some(generate_error_response(StatusCode::NOT_FOUND));
            }
            Ok(path) => {
                if path.starts_with(folder_path) {
                    path
                } else {
                    return Some(generate_error_response(StatusCode::NOT_FOUND));
                }
            }
        };

        debug!("Searching for {:?}", abs_file_path);
        Some(self.run_cgi_script(stream, request, abs_file_path))
    }
}
