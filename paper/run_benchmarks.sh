#!/bin/bash

cd /data

if [ ! -f "/data/test.fq.gz"]; then
    echo "test.fq.gz is missing!"
    exit 1
fi

# Tools available

if nanoq -h; then
    echo "nanoq returned true"
else
    echo "nanoq returned error"
    exit 1
fi

if filtlong --help; then
    echo "filtlong returned true"
else
    echo "filtlong returned error"
    exit 1
fi

if nanostat --help; then
    echo "nanostat returned true"
else
    echo "nanostat returned error"
    exit 1
fi

if nanofilt --help; then
    echo "nanofilt returned true"
else
    echo "nanofilt returned error"
    exit 1
fi