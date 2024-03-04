use log::{debug, error, info, warn};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::{sleep, Duration},
};

use crate::entity::{Api, Server};

pub async fn handle(server: Server) {
    let host = server.host;
    let port = server.port;
    // 监听端口
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    info!("http sevrer running on http://{}:{}", host, port);

    let apis = Arc::new(Mutex::new(server.apis));
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

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let socket_addr = &socket.peer_addr().unwrap();
        debug!(
            "New request came from {}, port {}",
            socket_addr.ip(),
            socket_addr.port()
        );
        let apis = apis.clone();
        let base_url = base_url.clone();
        let error = error.clone();
        let cors_header = cors_header.clone();
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
                let mut status_code = 0u16;
                let query_map = parse_query_string(query);
                // 判断请求的query参数是否与配置文件中指定的query参数一致
                let mut equals_keys = |opt: Option<Vec<String>>| {
                    let map_keys: Vec<String> = query_map.keys().cloned().collect();
                    if let Some(arr) = opt {
                        if arr == map_keys {
                            debug!("Parameter name, parameter quantity matched successfully");
                            debug!("{:?}", query_map);
                        } else {
                            warn!("Parameter name, number of parameters do not match exactly");
                            status_code = 400;
                        }
                    }
                };

                // 在此作用域中定义error，以便进行修改
                let mut error = error.as_str();

                let mut genarate_response = |ele: &mut Api| {
                    // 判断参数是否匹配成功
                    equals_keys(ele.request.query.clone());
                    timeout = ele.response.timeout;
                    let mut data = ele.response.data.clone();
                    // 判断是否指定返回类型为文件类型，如果是，则读取文件内容
                    if let Some(is_file) = ele.response.is_file {
                        if is_file {
                            debug!("Reading file content in {}", data);
                            match File::open(Path::new(data.as_str())) {
                                Ok(file) => {
                                    let mut file = file;
                                    // 读取文件内容到字符串
                                    let mut contents = String::new();
                                    file.read_to_string(&mut contents).unwrap();
                                    data = contents;
                                    // 缓存第一次读取的内容，避免重复读取，影响性能
                                    ele.response.data = data.clone();
                                    ele.response.is_file = None;
                                }
                                Err(_) => {
                                    // 读取不到文件
                                    error = "The file specified cannot be found";
                                    error!("{}", error);
                                    return;
                                }
                            }
                        } else {
                            data = ele.response.data.clone();
                        }
                    } else {
                        data = ele.response.data.clone();
                    }
                    response = ele
                        .response
                        .content_type
                        .wrap_response(data, cors_header.as_str());
                };
                // 遍历接口配置信息
                for ele in apis.lock().unwrap().iter_mut() {
                    if ele.request.method.to_string() != method
                        || !is_path_equals(path, base_url.as_ref(), &ele.request.url)
                    {
                        continue;
                    }
                    genarate_response(ele);
                    break;
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

///
/// 判断全路径是否匹配得当
fn is_path_equals(path: &str, base_url: &String, sub_url: &String) -> bool {
    // 如果path以/结尾，则去掉最后的/
    let path = if path.ends_with("/") && path.len() > 1 {
        &path[..path.len() - 1]
    } else {
        path
    };
    if base_url == "/" {
        path == sub_url
    } else {
        if sub_url == "/" {
            path == base_url
        } else {
            path == format!("{}{}", base_url, sub_url)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::is_path_equals;

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

    #[test]
    fn is_path_equals_test() {
        let data = [
            ("/hello/", &String::from("/hello"), &String::from("/")),
            ("/hello", &String::from("/hello"), &String::from("/")),
            ("/hello/", &String::from("/"), &String::from("/hello")),
            (
                "/hello/reine",
                &String::from("/hello"),
                &String::from("/reine"),
            ),
            ("/", &String::from("/"), &String::from("/")),
            ("//", &String::from("/"), &String::from("/")),
        ];
        for (a, b, c) in data {
            assert!(is_path_equals(a, b, c));
        }
    }
}
