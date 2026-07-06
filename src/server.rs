use anyhow::Result;
use std::fs;
use std::io::Read;
use std::path::Path;
use tiny_http::{Server, Response};

fn get_mime_type(path: &str) -> &str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

pub fn serve(build_dir: &str, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;
    println!("Server started: http://localhost:{}", port);
    println!("Press Ctrl+C to stop");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let url = if url == "/" {
            "/index.html"
        } else {
            &url
        };

        let file_path = Path::new(build_dir).join(url.trim_start_matches('/'));

        if file_path.exists() && file_path.is_file() {
            let mut file = fs::File::open(&file_path)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;

            let mime = get_mime_type(url);
            let response = Response::from_data(content)
                .with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &format!("{}; charset=utf-8", mime)[..],
                    )
                    .unwrap(),
                );
            request.respond(response)?;
        } else {
            let response = Response::from_string("404 Not Found")
                .with_status_code(404);
            request.respond(response)?;
        }
    }

    Ok(())
}
