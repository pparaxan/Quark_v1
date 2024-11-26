use crate::Quark;
use crate::error::QuarkError;
use include_dir::{Dir, include_dir};
use std::{path::Path, sync::Arc};
use tiny_http::{Response, Server};

static QUARKFOLDER: Dir = include_dir!("$CARGO_MANIFEST_DIR/src_quark");

pub fn setup_static(quark: &mut Quark) -> Result<(), QuarkError> {
    let index_path = QUARKFOLDER
        .get_file(Path::new("index.html"))
        .ok_or(QuarkError::RustEmbedAssetNotFoundError)?
        .contents_utf8()
        .ok_or(QuarkError::RustEmbedAssetError)?;

    quark.webview.set_html(index_path);
    Ok(())
}

pub fn setup_http(quark: &mut Quark) -> Result<(), QuarkError> {
    // Admit, I feed my old code to ChatGPT and it gave me this, and holy crap wtf is this?
    // I'll- I'll somewhat fix this later
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
