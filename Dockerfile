FROM alpine:latest

ENV VERSION=0.10.0
ENV RELEASE=nanoq-${VERSION}-x86_64-unknown-linux-musl.tar.gz

RUN apk update && apk add --no-cache wget
RUN wget https://github.com/esteinig/nanoq/releases/download/${VERSION}/${RELEASE} && \
    tar xf nanoq-${VERSION}-x86_64-unknown-linux-musl.tar.gz && \ 
    mv nanoq-${VERSION}-x86_64-unknown-linux-musl/nanoq /bin  && \
    rm -rf nanoq-${VERSION}-x86_64-unknown-linux-musl.tar.gz nanoq-${VERSION}-x86_64-unknown-linux-musl