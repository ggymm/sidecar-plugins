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

apps=(
  "dns"
  "hash"
  "qrcode"
  "share"
)
for app in "${apps[@]}"; do
    echo "build $app"
    cd "$app" || exit
    cargo build --release
    cd ..

    if [ "$OS" = "win" ]; then
        cp -f "$app/target/release/$app.exe" "${app}-${OS}-${ARCH}.exe"
    else
        cp -f "$app/target/release/$app" "${app}-${OS}-${ARCH}"
    fi
done