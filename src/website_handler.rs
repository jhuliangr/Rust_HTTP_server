use crate::http::{Method, Request, Response, StatusCode};
use super::server::Handler;
use std::fs;

/// Serves static files and handles POST requests with template rendering.
///
/// The `public_path` field acts as the document root (like Nginx's `root` directive).
/// All file access is sandboxed to this directory via path canonicalization.
pub struct WebsiteHandler {
    public_path: String,
}

impl WebsiteHandler {
    pub fn new(public_path: String) -> Self {
        Self { public_path }
    }

    /// Reads a file from the public directory with directory traversal protection.
    ///
    /// An attacker might request `GET /../../etc/passwd` to escape the document root.
    /// We defend against this by resolving the *real* absolute path with `canonicalize()`
    /// (which follows symlinks and resolves `..`), then checking that the resolved path
    /// still starts with our public directory. If it doesn't, the request is blocked.
    fn read_file(&self, file_path: &str) -> Option<String> {
        let full_path = format!("{}/{}", self.public_path, file_path);

        match fs::canonicalize(full_path) {
            Ok(canonical_path) => {
                if canonical_path.starts_with(&self.public_path) {
                    fs::read_to_string(canonical_path).ok()
                } else {
                    eprintln!(
                        "[SECURITY] Directory traversal attempt blocked: {}",
                        file_path
                    );
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Replaces the `{-}` placeholder in an HTML template with the given content.
    /// This is a minimal template engine -- just enough to inject POST data into a page.
    fn render_template(template_html: &str, content: &str) -> String {
        template_html.replace("{-}", content)
    }
}

impl Handler for WebsiteHandler {
    fn handle_request(&mut self, request: &Request) -> Response {
        match request.method() {
            Method::Get => match request.path() {
                "/" => Response::new(StatusCode::Ok, self.read_file("index.html")),
                path => match self.read_file(path) {
                    Some(contents) => Response::new(StatusCode::Ok, Some(contents)),
                    None => Response::new(StatusCode::NotFound, None),
                },
            },

            Method::Post => match request.path() {
                "/" => {
                    let template = self.read_file("post.html")
                        .unwrap_or_else(|| "<pre>{-}</pre>".to_string());

                    let rendered_html = Self::render_template(&template, request.body());
                    Response::new(StatusCode::Ok, Some(rendered_html))
                }
                unregistered_path => {
                    let echo_response = format!(
                        "POST {} received with body: {}",
                        unregistered_path,
                        request.body()
                    );
                    Response::new(StatusCode::Ok, Some(echo_response))
                }
            },

            _ => Response::new(StatusCode::NotFound, None),
        }
    }
}
