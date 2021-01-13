#!/bin/bash

cd /data

if [ ! -f "/data/test.fq.gz"]; then
    echo "test.fq.gz is missing!"
    exit 1
fi

if [ ! -f "/data/test.fq"]; then
    echo "test.fq is missing!"
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

REPLICATES=100

mkdir -p nanoq_reps nanoq_stats_reps filtlong_reps nanostat_reps nanofilt_reps

NANOSTAT_COMMAND="NanoStat -f test.fq -t 1"
NANOFILT_COMMAND="NanoFilt -f test.fq -l 5000 > /dev/null"
NANOQ_STAT_COMMAND="cat test.fq | nanoq"
NANOQ_FILT_COMMAND-"cat test.fq | nanoq -l 5000 > /dev/null"
FILTLONG_COMMAND="filtlong --min_length 5000 test.fq > /dev/null"

TIMEFORMAT="%R"

tm() {
    time $@
}

for i in 1..$REPLICATES; do

    # Gzipped time replicate iteration
    tm echo $i

    # test file stat nanoq
    time cat test.fq | nanoq
    # test file gzipped stat nanoq
    time cat test.fq.gz | nanoq 
    # test file stat nanoq crab cast
    time cat test.fq | nanoq -c

    # test file stat nanostat 4 cpu
    time NanoStat -f test.fq -t 4
    # test file stat gzipped nanostat 4 cpu
    time NanoStat -f test.fq.gz -t 4

    # test file filt nanofilt
    time NanoFilt -f test.fq -l 5000 > /dev/null
    # test file filt gzipped nanofilt
    time NanoFilt -f test.fq.gz -l 5000 > /dev/null

    # test file filt nanoq
    time cat test.fq | nanoq -l 5000 > /dev/null
    # test file filt gzipped nanoq
    time cat test.fq.gz | nanoq -l 5000 > /dev/null
    # test file filt gzipped nanoq crab cast
    time cat test.fq.gz | nanoq -l 5000 -c > /dev/null

     # test file filt filtlong
    time filtlong --min_length 5000 test.fq > /dev/null
    # test file filt gzipped filtlong
    time filtlong --min_length 5000 test.fq.gz > /dev/null

done