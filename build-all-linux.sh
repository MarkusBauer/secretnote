#!/bin/sh

set -e

# INITIALIZE YOUR SYSTEM LIKE THIS
# apt-get install -y musl-tools mingw-w64 gcc-arm-linux-gnueabihf libc6-armhf-cross libc6-dev-armhf-cross
# rustup target add x86_64-unknown-linux-musl
# rustup target add armv7-unknown-linux-gnueabihf
# rustup target add x86_64-pc-windows-gnu


# BUILD SERVER
# Linux + musl
cargo build --release --target x86_64-unknown-linux-musl

# Raspberry
# export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
cargo build --release --target armv7-unknown-linux-gnueabihf

# Windows
cargo build --release --target x86_64-pc-windows-gnu


# BUILD FRONTEND
cd secretnote-fe
npm install
npm run build
cd ..


# PACKAGE STUFF INTO ARCHIVES
echo Packaging ...
mkdir -p releases

cp target/x86_64-unknown-linux-musl/release/secretnote ./
tar -czf releases/secretnote-linux-x86_64.tar.gz secretnote fe
cp target/armv7-unknown-linux-gnueabihf/release/secretnote ./
tar -czf releases/secretnote-linux-armhf.tar.gz secretnote fe
rm ./secretnote
cp target/x86_64-pc-windows-gnu/release/secretnote.exe ./
zip releases/secretnote-windows-x64.zip secretnote.exe fe
rm secretnote.exe

echo "Build finished!"
