use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

mod server;
mod entity;

use crate::entity::Server;

use crate::server::handle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "api.yml";
    match File::open(file_name) {
        Ok(f) => {
            let mut file = f;
            // 读取文件内容到字符串
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let server: Server = serde_yaml::from_str(&contents)?;
            // println!("{:?}", server);
            handle(server).await;
        }
        Err(_) => {
            eprintln!("系统找不到指定的文件，已根据默认值生成");
            let server: Server = Server::default();
            let server_str = serde_yaml::to_string(&server).unwrap();
            let mut file = File::create(file_name).unwrap();
            file.write_all(server_str.as_bytes()).unwrap();
        }
    }
    Ok(())
}
