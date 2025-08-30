# DNS Resolver

DNS域名解析工具，支持多DNS服务器并发查询和延迟测试。

## 技术栈

- **trust-dns-resolver** - DNS解析库
- **tokio** - 异步运行时
- **clap** - 命令行参数解析
- **regex** - 正则表达式解析ping输出

## 功能特性

- 并发查询多个DNS服务器
- 自动ping测试获取延迟和丢包率
- 支持自定义DNS服务器列表
- 多语言支持（中英文）
- 结果按延迟和丢包率排序

## 命令行使用

```bash
# 基本用法
./dns example.com

# 使用自定义DNS服务器文件
./dns example.com -n nameservers.txt
```

## 构建

```bash
cargo build --release
```