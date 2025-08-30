# Hash Calculator

高性能文件哈希计算工具，支持并行计算多种哈希算法。

## 技术栈

- **digest** - 哈希算法通用接口
- **md5, sha1, sha2** - 具体哈希算法实现
- **crossbeam-channel** - 高性能通道通信
- **hex** - 十六进制编码

## 功能特性

- 同时计算MD5、SHA1、SHA256、SHA512
- 多线程并行处理提高性能
- 大文件分块读取，内存友好
- 512MB缓冲区优化I/O性能

## 命令行使用

```bash
# 计算文件的所有哈希值
./hash /path/to/your/file
```

输出示例：

```
MD5:     d41d8cd98f00b204e9800998ecf8427e
SHA1:    da39a3ee5e6b4b0d3255bfef95601890afd80709
SHA256:  e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
SHA512:  cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
```

## 构建

```bash
cargo build --release
```