# QR Code Decoder

二维码图片解码工具，从图片中提取二维码内容。

## 技术栈

- **rqrr** - 二维码检测和解码库
- **image** - 图像处理库

## 功能特性

- 自动检测图片中的二维码
- 高精度解码算法
- 支持多种图片格式
- 转换为灰度图像进行处理

## 命令行使用

```bash
# 解码二维码图片
./qrcode /path/to/qrcode.png
```

输出示例：

```
https://example.com/some-url
```

如果图片中没有找到二维码或解码失败，会显示相应的错误信息。

## 构建

```bash
cargo build --release
```