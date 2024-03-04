# 简易接口mock工具

## 配置文件示例
`api.yml`

```yml
log_level: debug # 日志级别
host: 127.0.0.1 # 主机
port: 8080 # 端口
base: / # 基础路径
cors: true # 是否开启跨域
error: 404 not found # 当接口不存在时，返回的错误提示信息
apis: # 定义使用rest api，以yml数组形式，可以定义多个
  - request: # 请求信息
      method: GET # 接受的请求方法
      query: # 本次请求需要的参数列表（缺省）
        - name
      url: /json # 二级路径
    response: 响应信息
      timeout: 0 # 模拟接口处理耗时，单位为秒
      content_type: JSON # 响应内容类型，接受JSON，TEXT，HTML
      is_file: true # 是否从文件中读取返回数据（缺省），缺失此字段则默认为文本
      data: result.json # 如果是从文件中读取返回数据，则此处填写文件路径，否则填写要返回的文本
```

## 构建

1. 安装rust开发环境

2. 生成可执行文件，执行以下命令后，在`target/release`目录下将生成`http-rs`可执行文件

    ```bash
    cargo build --release
    ```

## 启动

1. 使用`http-rs`启动时，如果没有配置文件则会自动生成默认配置文件，配置完成后再次启动即可

2. 使用`http-rs -h`查看帮助信息

3. 使用`http-rs -f <file>`指定配置文件路径并启动