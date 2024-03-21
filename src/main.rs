use std::{
    env,
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use chrono::Local;
use env_logger::Builder;

use crate::entity::Server;
use crate::server::handle;

mod entity;
mod server;

#[tokio::main]
async fn main() {
    handle_program_args().await;
}

///
/// 处理程序启动参数
///
async fn handle_program_args() {
    let args: Vec<String> = env::args().collect();

    let a1 = args.get(1);
    let a2 = args.get(2);
    match a1 {
        Some(cmd) => match cmd.as_str() {
            "--help" | "-h" => println!("{}", get_help()),
            "--file" | "-f" => match a2 {
                Some(file_path) => {
                    if let Err(e) = start(file_path, false).await {
                        panic!("程序启动异常{}", e);
                    }
                }
                None => eprintln!("请指定配置文件路径"),
            },
            _ => eprintln!("启动参数错误"),
        },
        None => {
            if let Err(e) = start("api.yml", true).await {
                panic!("程序启动异常{}", e);
            }
        }
    }
}

///
/// 启动服务器
async fn start(file_name: &str, default: bool) -> Result<(), Box<dyn Error>> {
    match File::open(Path::new(file_name)) {
        Ok(f) => {
            let mut file = f;
            // 读取文件内容到字符串
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let server: Server = serde_yaml::from_str(&contents)?;
            Builder::new()
                .parse_filters(server.log_level.as_str())
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "{} [{}] - {}",
                        Local::now().format("%Y-%m-%dT%H:%M:%S"),
                        record.level(),
                        record.args()
                    )
                })
                .init();
            handle(server).await;
        }
        Err(_) => {
            if default {
                eprintln!("系统找不到指定的文件，已根据默认值生成");
                let server: Server = Server::default();
                let server_str = serde_yaml::to_string(&server).unwrap();
                let mut file = File::create(file_name).unwrap();
                file.write_all(server_str.as_bytes()).unwrap();
            } else {
                eprintln!("系统找不到指定的文件");
            }
        }
    }
    Ok(())
}

///
///
/// 帮助信息
fn get_help() -> String {
    let help_msg = r#"
http-server [option]

option:
    -h, --help: 查看帮助信息
    -f, --file <path>: 指定配置文件路径
    "#;
    String::from(help_msg)
}
