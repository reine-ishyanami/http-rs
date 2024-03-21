use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::Arc};

use log::{debug, error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
    time::{Duration, sleep},
};

use crate::entity::{HttpMethod, Request, Response, Server};

pub async fn handle(server: Server) {
    let host = server.host;
    let port = server.port;
    // 监听端口
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    info!("http server running on http://{}:{}", host, port);

    // let apis = Arc::new(Mutex::new(server.apis));
    let base_url = Arc::new(server.base);
    let error = Arc::new(server.error);

    let mut cors_header = String::new();
    if server.cors {
        cors_header.push_str("Access-Control-Allow-Origin: *\r\n"); // 允许所有远程地址访问
        cors_header.push_str("Access-Control-Allow-Methods: * \r\n"); // 允许所有请求方法
        cors_header.push_str("Access-Control-Allow-Headers: Content-Type, Authorization, Content-Length, X-Requested-With \r\n"); // 允许如下请求头
        cors_header.push_str("Access-Control-Allow-Credentials: true \r\n"); // 允许跨域请求携带凭据
    }

    let cors_header = Arc::new(cors_header);

    // 初始化一个hashmap，用于构建请求url与内容的映射
    let mut url_map: HashMap<Request, Response> = HashMap::new();
    for api in server.apis.iter() {
        url_map.insert(api.request.clone(), api.response.clone());
    }

    let url_map = Arc::new(Mutex::new(url_map));

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let socket_addr = &socket.peer_addr().unwrap();
        debug!(
            "New request came from {}, port {}",
            socket_addr.ip(),
            socket_addr.port()
        );
        // let apis = apis.clone();
        let base_url = base_url.clone();
        let error = error.clone();
        let cors_header = cors_header.clone();
        let url_map = url_map.clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // 读取请求数据
            if let Ok(_) = socket.read(&mut buf).await {
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
                let status_code = 0u16;

                // 在此作用域中定义error，以便进行修改
                let mut error = error.as_str();

                let mut genarate_response = |resp: &mut Response| {
                    timeout = resp.timeout;
                    let mut data = resp.data.clone();
                    // 判断是否指定返回类型为文件类型，如果是，则读取文件内容
                    if let Some(is_file) = resp.is_file {
                        if is_file {
                            debug!("Reading file content in {}", data);
                            match File::open(Path::new(data.as_str())) {
                                Ok(file) => {
                                    let mut file = file;
                                    // 读取文件内容到字符串
                                    let mut contents = String::new();
                                    file.read_to_string(&mut contents).unwrap();
                                    data = contents.clone();
                                    resp.is_file = Some(false);
                                    resp.data = contents;
                                }
                                Err(_) => {
                                    // 读取不到文件
                                    error = "The file specified cannot be found";
                                    error!("{}", error);
                                    return;
                                }
                            }
                        } else {
                            data = resp.data.clone();
                        }
                    } else {
                        data = resp.data.clone();
                    }
                    response = resp.content_type.wrap_response(data, cors_header.as_str());
                };

                let req = generate_request(path, base_url.as_str(), method, query);
                let mut url_map = url_map.lock().await;
                let resp = url_map.get_mut(&req);
                if let Some(response) = resp {
                    genarate_response(response);
                }
                // 如果超时时间不为0，则模拟接口请求耗时
                if timeout > 0 {
                    sleep(Duration::from_secs(timeout)).await;
                }
                // 如果没有返回结果，则返回错误信息
                if response.len() == 0 {
                    let error = error;
                    response = format!(
                        "HTTP/1.1 404 OK\r\n{}Content-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        cors_header,
                        error.len(),
                        error
                    );
                }
                // 如果状态码不为0，则返回参数不匹配
                if status_code != 0 {
                    let error = "Parameters mismatch";
                    response = format!(
                        "HTTP/1.1 {} OK\r\n{}Content-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        status_code,
                        cors_header,
                        error.len(),
                        error
                    );
                }
                // 返回信息
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
    }
}

///
///
fn generate_request(path: &str, base_url: &str, method: &str, query: &str) -> Request {
    // 获取method
    // let method = HttpMethod::from_str(method).unwrap();
    let method:HttpMethod = serde_yaml::from_str(method).unwrap();
    // 获取url
    let mut url: String;
    if base_url != "/" {
        url = path[base_url.len()..].to_owned();
    } else {
        url = path.to_owned();
    }
    if url.len() != 1 && url.ends_with("/") {
        url.pop();
    }
    // 获取query
    let map = parse_query_string(query);
    let keys: Vec<&String> = map.keys().collect();
    let query: Vec<String> = keys
        .iter()
        .map(|k| k.as_str())
        .map(|s| s.to_owned())
        .collect();
    if map.is_empty() {
        Request {
            method,
            url,
            query: None,
        }
    } else {
        Request {
            method,
            url,
            query: Some(query),
        }
    }
}

///
/// 将query参数收集成map
///
fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&') // 按 "&" 分割查询字符串
        .filter_map(|part| {
            // 过滤并映射每一部分
            let mut split = part.splitn(2, '='); // 按 "=" 分割，最多分割成两部分
            match (split.next(), split.next()) {
                (Some(key), Some(value)) => Some((key.to_owned(), value.to_owned())),
                _ => None, // 忽略不能被分割成两部分的项
            }
        })
        .collect() // 收集成 HashMap
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::parse_query_string;

    #[test]
    fn parse_query_string_test() {
        let json_str = "name=reine&age=23";
        let result = parse_query_string(json_str);
        let mut map = HashMap::new();
        map.insert(String::from("name"), String::from("reine"));
        map.insert(String::from("age"), String::from("23"));
        assert_eq!(result, map);
    }
}
