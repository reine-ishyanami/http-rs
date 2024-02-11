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
            host: "127.0.0.1".to_string(),
            port: 8080,
            base: "/".to_string(),
            error: "404 not found".to_string(),
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
            url: "/".to_string(),
            query: Some(vec!["name".to_string(), "age".to_string()]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContentType {
    JSON,
    TEXT,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub timeout: u64,
    pub content_type: ContentType,
    // 响应内容类型，json或text
    pub data: String,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            timeout: 0,
            content_type: ContentType::TEXT,
            data: "hello world".to_string(),
        }
    }
}