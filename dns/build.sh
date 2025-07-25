#!/bin/sh

cargo clean
cargo build --release

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

if [ "$OS" = "win" ]; then
  cp -f target/release/dns.exe "dns-${OS}-${ARCH}.exe"
else
  cp -f target/release/dns "dns-${OS}-${ARCH}"
fi
