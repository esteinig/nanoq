mkdir -p $PREFIX/bin

cargo build --release
mv target/release/nanoq $PREFIX/bin/nanoq