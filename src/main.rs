use std::net::TcpListener;
use std::sync::Arc;

use rust_web_cgi::http_server::{
    connection::ConnectionHandler,
    request::{
        cgi_request::cgi_handler::CgiRequestHandler,
        static_request::static_handler::StaticRequestHandler,
    },
};
use rust_web_cgi::threadpool::ThreadPool;

const ADDR_AND_PORT: &str = "127.0.0.1:8080";
const POOL_SIZE: usize = 4;

const STATIC_FOLDER: &str = "public_html";
const CGI_FOLDER: &str = "cgi-bin";
const CGI_PATH: &str = "cgi-bin";

fn main() {
    env_logger::init();

    let listener = TcpListener::bind(ADDR_AND_PORT).unwrap();
    let pool = ThreadPool::new(POOL_SIZE);

    let conn_handler = Arc::new(ConnectionHandler::new(vec![
        Box::new(CgiRequestHandler::new(
            String::from(CGI_PATH),
            String::from(CGI_FOLDER),
            StaticRequestHandler::new(String::from(STATIC_FOLDER)),
        )),
        Box::new(StaticRequestHandler::new(String::from(STATIC_FOLDER))),
    ]));

    println!("Booting up.");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let handler_clone = Arc::clone(&conn_handler);

        pool.execute(move || {
            (*handler_clone).handle_connection(stream);
        });
    }

    println!("Shutting down.");
}
