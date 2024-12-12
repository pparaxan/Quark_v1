use crate::Quark;
use crate::error::QuarkError;
use crate::cli::QUARKFOLDER;
use std::sync::Arc;
use tiny_http::{Response, Server};

pub fn build_http(quark: &mut Quark) -> Result<(), QuarkError> { // Rewrite this
    let server = Server::http("127.0.0.1:24114").map_err(|_| QuarkError::ServerError)?;
    let addr = server.server_addr();

    let shared_frontend_path = Arc::new(QUARKFOLDER.clone());
    std::thread::spawn({
        let shared_frontend_path = Arc::clone(&shared_frontend_path);
        move || {
            for request in server.incoming_requests() {
                let requested_path = request.url().trim_start_matches('/');
                if let Some(file) = shared_frontend_path.get_file(requested_path) {
                    let response = Response::from_data(file.contents_utf8().unwrap_or_default());
                    if let Err(err) = request.respond(response) {
                        eprintln!("Failed to respond: {}", err);
                    }
                }
            }
        }
    });

    let uri = format!("http://{}/index.html", addr);
    quark.webview.navigate(&uri);
    Ok(())
}
