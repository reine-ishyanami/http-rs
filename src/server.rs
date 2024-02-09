use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex, time::{sleep,Duration}
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    host: String,
    port: u16,
    base: String,
    error: String,
    apis: Vec<Api>,
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
struct Api {
    request: Request,
    response: Response,
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
enum HttpMethod {
    GET,POST,PUT,DELETE,HEAD,PATCH,OPTIONS,CONNECT,TRACE
}

impl HttpMethod {
    pub fn to_string(&self) -> &str{
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
struct Request {
    method: HttpMethod,
    url: String,
    query: Option<Vec<String>>
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: "/".to_string(),
            query: Some(vec!["name".to_string(), "age".to_string()])
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Response {
    timeout: u64,
    content_type: String, // 响应内容类型，json或text
    data: String, 
}

impl Default for Response {
    fn default() -> Self {
        Self {
            timeout: 0,
            content_type: "text".to_string(),
            data: "hello world".to_string(),
        }
    }
}

pub async fn handle(server: Server) {
    let host = server.host;
    let port = server.port;
    // 监听端口
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    println!("Listening on {}:{}", host, port);

    let apis = Arc::new(Mutex::new(server.apis));
    let base_url = Arc::new(Mutex::new(server.base));
    let error = Arc::new(Mutex::new(server.error));

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let apis = apis.clone();
        let base_url = base_url.clone();
        let error = error.clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // 读取请求数据
            match socket.read(&mut buf).await {
                Ok(_) => {
                    // 将请求数据转换为字符串
                    let request = String::from_utf8_lossy(&buf);
                    // 解析请求行（第一行）
                    let request_line = request.lines().next().unwrap_or_default();
                    let mut parts = request_line.split_whitespace();
                    let method = parts.next().unwrap_or_default(); // 请求方法
                    let path_full = parts.next().unwrap_or_default(); // 完整路径（可能包含查询参数）

                    // 分割路径和查询字符串
                    let (path, query) = match path_full.split_once('?') {
                        Some((p, q)) => (p, q),
                        None => (path_full, ""),
                    };


                    let mut response = String::new();
                    let mut timeout = 0u64;
                    let mut status_code = 0u16;
                    let query_map = parse_query_string(query);
                    let mut equals_keys = |opt|{
                        let map_keys: Vec<String> = query_map.keys().cloned().collect();
                        if let Some(arr) = opt {
                            if arr == map_keys {
                                println!("参数名称，参数数量匹配成功");
                                println!("{:?}", query_map);
                            } else {
                                eprintln!("参数名称，参数数量不完全匹配");
                                status_code = 400;
                            }
                        }
                    };

                    for ele in apis.lock().await.iter() {
                        match split_at_second_slash(path) {
                            // 多层url
                            UrlSplit::Pair(first, second) => {   
                                // 请求方法正确，请求路径正确
                                if ele.request.method.to_string() == method
                                    && *base_url.lock().await == first
                                    && ele.request.url == second
                                {
                                    // 判断参数是否匹配成功
                                    equals_keys(ele.request.query.clone());
                                    timeout = ele.response.timeout;
                                    let data = &ele.response.data;
                                    response = match ele.response.content_type.as_str() {
                                        "text" | "TEXT" => 
                                            format!(                                          
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                                data.len(),
                                                data
                                            ),
                                        "json" | "JSON" => 
                                            format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                                data.len(),
                                                data
                                            ),
                                        _ => panic!("配置文件有误")
                                    };
                                    break;
                                }
                            }
                            // 单层url
                            UrlSplit::Single(one) => {
                                if ele.request.method.to_string() == method {
                                    let base_url = format!("{}{}", *base_url.lock().await, ele.request.url);
                                    let base_url = if base_url.ends_with("/") {
                                        &base_url[..base_url.len()-1]
                                    }else {
                                        base_url.as_str()
                                    };
                                    if one == base_url {   
                                        // 判断参数是否匹配成功
                                        equals_keys(ele.request.query.clone());
                                        timeout = ele.response.timeout;
                                        let data = &ele.response.data;
                                        response = match ele.response.content_type.as_str() {
                                            "text" | "TEXT" => 
                                                format!(                                          
                                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                                    data.len(),
                                                    data
                                                ),
                                            "json" | "JSON" => 
                                                format!(
                                                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                                    data.len(),
                                                    data
                                                ),
                                            _ => panic!("配置文件有误")
                                        };
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if timeout > 0 {
                        sleep(Duration::from_secs(timeout)).await;
                    }
                    if response.len() == 0 {
                        let error = error.lock().await;                   
                        response = format!(
                            "HTTP/1.1 404 OK\r\nContent-Length: {}\r\n\r\n{}",
                            error.len(),
                            error
                        );
                    }
                    if status_code != 0 {                   
                        let error = "参数不匹配";
                        response = format!(
                            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
                            status_code,
                            error.len(),
                            error
                        );
                    }
                    socket.write_all(response.as_bytes()).await.unwrap();
                }
                Err(e) => println!("Failed to read from socket: {}", e),
            }
        });
    }
}


///
/// 将query参数收集成map
/// 
fn parse_query_string(query: &str) -> HashMap<String, String> {
    query.split('&') // 按 "&" 分割查询字符串
         .filter_map(|part| { // 过滤并映射每一部分
             let mut split = part.splitn(2, '='); // 按 "=" 分割，最多分割成两部分
             match (split.next(), split.next()) {
                 (Some(key), Some(value)) => Some((key.to_owned(), value.to_owned())),
                 _ => None, // 忽略不能被分割成两部分的项
             }
         })
         .collect() // 收集成 HashMap
}


enum UrlSplit<'a> {
    Pair(&'a str, &'a str),
    Single(&'a str)
}

///
/// 在第二个/处对字符串进行切割
fn split_at_second_slash(s: &str) -> UrlSplit {
    let mut iter = s.match_indices('/');
    let first_slash = iter.nth(1); // 跳过第一个斜杠，直接找到第二个斜杠
    match first_slash {
        Some((idx, _)) => {
            let (a,b) = s.split_at(idx);
            UrlSplit::Pair(a, b)
        }
        None => UrlSplit::Single(s), // 如果没有找到第二个斜杠，则返回 None
    }
}
