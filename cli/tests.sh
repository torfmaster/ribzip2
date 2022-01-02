#!/bin/bash
set -eux

cargo run -- compress samples/idiot.txt  
mv samples/idiot.txt.bz2 temp/
bunzip2 temp/idiot.txt.bz2
rm temp/idiot.txt

cargo run -- compress samples/pepper.txt
cargo run -- decompress samples/pepper.txt.bz2
rm samples/pepper.txt.bz2
rm samples/pepper.txt.out

cargo run -- compress samples/idiot.txt
cargo run -- decompress samples/idiot.txt.bz2
rm samples/idiot.txt.bz2
rm samples/idiot.txt.out
