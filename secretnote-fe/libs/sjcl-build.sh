#!/bin/sh

set -e

test -d sjcl || git clone https://github.com/bitwiseshiftleft/sjcl.git
cd sjcl

./configure --with-bitArray --with-ecc --with-codecBytes --with-codecZ85 --with-codecArrayBuffer
#./configure --with-bitArray --with-ecc --with-codecBytes --with-codecArrayBuffer --without-ccm --without-ocb2 --without-codecBase32 --without-codecHex --without-hmac
make
cd ..
cp sjcl/sjcl.js ./
