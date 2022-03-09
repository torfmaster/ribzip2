#!/bin/bash
set -eux

cargo run -- compress samples/idiot.txt k-means --iterations 6 --num-tables 6
mv samples/idiot.txt.bz2 temp/
bunzip2 temp/idiot.txt.bz2
rm temp/idiot.txt

cargo run -- compress samples/idiot.txt  
mv samples/idiot.txt.bz2 temp/
bunzip2 temp/idiot.txt.bz2
rm temp/idiot.txt

cargo run -- compress samples/pepper.txt
mv samples/pepper.txt.bz2 temp/
cargo run -- decompress temp/pepper.txt.bz2
rm temp/pepper.txt.bz2
rm temp/pepper.txt

cargo run -- compress samples/idiot.txt
mv samples/idiot.txt.bz2 temp/
cargo run -- decompress temp/idiot.txt.bz2
rm temp/idiot.txt.bz2
rm temp/idiot.txt
