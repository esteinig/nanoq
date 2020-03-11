FROM frolvlad/alpine-glibc

ADD target/release/nanoq /bin/

ENTRYPOINT ["/bin/nanoq"]
CMD []
