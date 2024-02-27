use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub base: String,
    pub cors: bool,
    pub error: String,
    pub apis: Vec<Api>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 8080,
            base: String::from("/"),
            cors: false,
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
    HTML,
}


impl ContentType {
    pub fn wrap_response(&self, data: String, cors_header: &str) -> String {
        match self {
            ContentType::TEXT => format!(
                "HTTP/1.1 200 OK\r\n{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                cors_header,
                data.len(),
                data
            ),
            ContentType::JSON => format!(
                "HTTP/1.1 200 OK\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                cors_header,
                data.len(),
                data
            ),
            ContentType::HTML => {
                // 如果类型是HTML，则返回结果应该是一个文件路径
                match File::open(Path::new(data.as_str())) {
                    Ok(file) => {
                        let mut file = file;
                        // 读取文件内容到字符串
                        let mut contents = String::new();
                        file.read_to_string(&mut contents).unwrap();
                        format!(
                            "HTTP/1.1 200 OK\r\n{}Content-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                            cors_header,
                            contents.len(),
                            contents
                        )
                    }
                    Err(_) => String::new(),
                }
            }
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
