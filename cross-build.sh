#!/bin/bash
# 交叉编译脚本 - 用于本地编译多平台二进制文件

# 确保已安装目标平台
echo "确保已安装相关目标平台..."
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin

# 为Windows编译
echo "正在为Windows编译..."
cargo build --release --target x86_64-pc-windows-msvc
if [ $? -eq 0 ]; then
  echo "Windows版本编译成功: target/x86_64-pc-windows-msvc/release/cfmail.exe"
else
  echo "Windows版本编译失败"
  exit 1
fi

# 为Linux编译（如果在macOS或其他非Linux系统上，需要安装额外工具）
if [ "$(uname)" != "Linux" ]; then
  echo "警告: 在非Linux平台上编译Linux版本可能需要安装额外的工具"
fi
echo "正在为Linux编译..."
cargo build --release --target x86_64-unknown-linux-gnu
if [ $? -eq 0 ]; then
  echo "Linux版本编译成功: target/x86_64-unknown-linux-gnu/release/cfmail"
else
  echo "Linux版本编译失败"
fi

# 为macOS编译（如果在非macOS系统上，可能需要安装额外工具）
if [ "$(uname)" != "Darwin" ]; then
  echo "警告: 在非macOS平台上编译macOS版本可能需要安装额外的工具"
fi
echo "正在为macOS编译..."
cargo build --release --target x86_64-apple-darwin
if [ $? -eq 0 ]; then
  echo "macOS版本编译成功: target/x86_64-apple-darwin/release/cfmail"
else
  echo "macOS版本编译失败"
fi

echo "创建发布目录..."
mkdir -p release

echo "复制编译好的文件到发布目录..."
cp target/x86_64-pc-windows-msvc/release/cfmail.exe release/cfmail-windows.exe
cp target/x86_64-unknown-linux-gnu/release/cfmail release/cfmail-linux
cp target/x86_64-apple-darwin/release/cfmail release/cfmail-macos
chmod +x release/cfmail-linux release/cfmail-macos

echo "复制配置和文档..."
cp config.toml release/config.toml.example
cp README.md release/

echo "编译完成！请查看 ./release 目录获取编译结果" 