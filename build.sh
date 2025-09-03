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


cargo clean
cargo build --release --workspace

apps=(
  "dns"
  "hash"
  "qrcode"
  "share"
  "system"
)
for app in "${apps[@]}"; do
    if [ "$OS" = "win" ]; then
        cp -f "target/release/$app.exe" "${app}-${OS}-${ARCH}.exe"
    else
        cp -f "target/release/$app" "${app}-${OS}-${ARCH}"
    fi
done