# File Share Server

基于HTTP的文件分享服务器，支持断点续传和二维码生成。

## 技术栈

- **Rust** - 系统编程语言
- **axum** - 现代异步Web框架
- **tokio** - 异步运行时
- **qrcode** - 二维码生成库
- **image** - 图像处理库
- **uuid** - 唯一标识符生成

## 功能特性

- HTTP文件服务器，支持大文件分享
- 断点续传支持（HTTP Range请求）
- 流式传输，内存占用低
- 自动生成二维码便于移动端访问
- UUID路径保护，防止文件路径泄露
- 自动获取本地IP地址和随机端口
- 正确的文件名下载支持

## 命令行使用

```bash
# 启动文件分享服务器
./share /path/to/your/file
```

输出示例：

```json
{
  "pid": 12345,
  "file_id": "863d6cfb-ceeb-433d-b11f-ed9d12af72e5",
  "base_url": "http://192.168.1.55:55940"
}
```

然后可以访问：

- `http://192.168.1.55:55940/qrcode` - 获取下载链接的二维码
- `http://192.168.1.55:55940/863d6cfb-ceeb-433d-b11f-ed9d12af72e5` - 下载文件

## 构建

```bash
cargo build --release
```