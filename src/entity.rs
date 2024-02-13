use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub base: String,
    pub error: String,
    pub apis: Vec<Api>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 8080,
            base: String::from("/"),
            error: String::from("404 not found"),
            apis: vec![Api::default()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Api {
    pub request: Request,
    pub response: Response,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            request: Request::default(),
            response: Response::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    PATCH,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl HttpMethod {
    pub fn to_string(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub query: Option<Vec<String>>,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: String::from("/"),
            query: Some(vec![String::from("name"), String::from("age")]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContentType {
    JSON,
    TEXT,
}

impl ContentType {
    pub fn wrap_response(&self, data: String) -> String {
        match self {
            ContentType::TEXT => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                data.len(),
                data
            ),
            ContentType::JSON => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                data.len(),
                data
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub timeout: u64,
    // 响应内容类型，json或text
    pub content_type: ContentType,
    pub data: String,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            timeout: 0,
            content_type: ContentType::TEXT,
            data: String::from("hello world"),
        }
    }
}
