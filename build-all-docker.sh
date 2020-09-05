#!/bin/sh

set -e

cross build --release --target x86_64-unknown-linux-musl
cross build --release --target i686-unknown-linux-musl
cross build --release --target armv7-unknown-linux-musleabihf
cross build --release --target aarch64-unknown-linux-musl
cross build --release --target x86_64-pc-windows-gnu


# BUILD FRONTEND
docker run --rm -it -v "$(pwd)":/code node:14 /bin/sh -c 'cd /code/secretnote-fe && npm install && npm run build'


# PACKAGE STUFF INTO ARCHIVES
echo Packaging ...
mkdir -p releases

cp target/x86_64-unknown-linux-musl/release/secretnote ./
tar -czf releases/secretnote-linux-x86_64.tar.gz secretnote fe

cp target/i686-unknown-linux-musl/release/secretnote ./
tar -czf releases/secretnote-linux-i686.tar.gz secretnote fe

cp target/armv7-unknown-linux-musleabihf/release/secretnote ./
tar -czf releases/secretnote-linux-armv7.tar.gz secretnote fe

cp target/aarch64-unknown-linux-musl/release/secretnote ./
tar -czf releases/secretnote-linux-aarch64.tar.gz secretnote fe
rm ./secretnote

cp target/x86_64-pc-windows-gnu/release/secretnote.exe ./
zip -r releases/secretnote-windows-x64.zip secretnote.exe fe
rm secretnote.exe

echo "Build finished!"
echo "Releases:"
ls -lah releases/*.gz releases/*.zip
