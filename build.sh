#!/bin/sh

case "$(uname)" in
"Darwin")
  OS="mac"
  ;;
"Linux")
  OS="linux"
  ;;
"MINGW64" | "MINGW32" | "MSYS"*)
  OS="win"
  ;;
*)
  OS="unknown"
  ;;
esac

case "$(uname -m)" in
"arm64" | "aarch64")
  ARCH="arm64"
  ;;
"x86_64" | "amd64")
  ARCH="amd64"
  ;;
*)
  ARCH="unknown"
  ;;
esac

echo 'build dns'
cd dns || exit
#cargo clean
cargo build --release
cd ..

echo 'build hash'
cd hash || exit
#cargo clean
cargo build --release
cd ..

echo 'build qrcode'
cd qrcode || exit
#cargo clean
cargo build --release
cd ..

if [ "$OS" = "win" ]; then
  cp -f dns/target/release/dns.exe "dns-${OS}-${ARCH}.exe"
  cp -f hash/target/release/hash.exe "hash-${OS}-${ARCH}.exe"
  cp -f qrcode/target/release/qrcode.exe "qrcode-${OS}-${ARCH}.exe"
else
  cp -f dns/target/release/dns "dns-${OS}-${ARCH}"
  cp -f hash/target/release/hash "hash-${OS}-${ARCH}"
  cp -f qrcode/target/release/qrcode "qrcode-${OS}-${ARCH}"
fi
