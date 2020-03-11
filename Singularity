Bootstrap: docker
From: alpine:latest

%labels
    name nanoq
    version 0.1.0
    author esteinig

%post
    apk add --no-cache rust cargo
    cd /nanoq && cargo build --release
    mv target/release/nanoq /bin

%files
    . /nanoq
