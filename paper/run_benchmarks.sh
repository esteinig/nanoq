#!/bin/bash

cd /data

if [ ! -f "/data/test.fq"]; then
    if [ ! -f "/data/test.fq.gz"]; then
        echo "test.fq.gz is missing!"
        exit 1
    fi
    zcat test.fq.gz > test.fq
fi

REPLICATES=100
TIMEFORMAT="%R"

for i in 1..$REPLICATES; do

    # Gzipped time replicate iteration
    echo "Replicate timer: $i"

    # test file stat nanoq
    (time cat test.fq | nanoq) 2> ${i}_nanoq_stat
    tail -1 ${i}_nanoq_stat >> nanoq_stat
    # test file gzipped stat nanoq
    (time cat test.fq.gz | nanoq) 2> ${i}_nanoq_gz_stat
    tail -1 ${i}_nanoq_gz_stat >> nanoq_gz_stat
    # test file stat nanoq crab cast
    (time cat test.fq | nanoq -c) 2> ${i}_nanoq_crab_stat
    tail -1 ${i}_nanoq_crab_stat >> nanoq_crab_stat

    # test file stat nanostat 4 cpu
    (time NanoStat --fastq test.fq -t 4) 2> ${i}_nanostat_stat
    tail -1 ${i}_nanostat_stat >> nanostat_stat
    # test file stat gzipped nanostat 4 cpu
    (time NanoStat --fastq test.fq.gz -t 4) 2> ${i}_nanostat_gz_stat
    tail -1 ${i}_nanostat_gz_stat >> nanostat_gz_stat

    # test file filt nanofilt
    (time NanoFilt test.fq -l 5000 > /dev/null) 2> ${i}_nanofilt_filt
    tail -1 ${i}_nanofilt_filt >> nanofilt_filt
    # test file filt gzipped nanofilt
    (time NanoFilt test.fq.gz -l 5000 > /dev/null) 2> ${i}_nanofilt_gz_filt
    tail -1 ${i}_nanofilt_gz_filt >> nanofilt_gz_filt

    # test file filt nanoq
    (time cat test.fq | nanoq -l 5000 > /dev/null) 2> ${i}_nanoq_filt
    tail -1 ${i}_nanoq_filt >> nanoq_filt
    # test file filt gzipped nanoq
    (time cat test.fq.gz | nanoq -l 5000 > /dev/null) 2> ${i}_nanoq_gz_filt
    tail -1 ${i}_nanoq_gz_filt >> nanoq_gz_filt 
    # test file filt gzipped nanoq crab cast
    (time cat test.fq.gz | nanoq -l 5000 -c > /dev/null) 2> ${i}_nanoq_crab_filt
    tail -1 ${i}_nanoq_crab_filt >> nanoq_crab_filt

     # test file filt filtlong
    (time filtlong --min_length 5000 test.fq > /dev/null) 2> ${i}_filtlong_filt
    tail -1 ${i}_filtlong_filt >> filtlong_filt
    # test file filt gzipped filtlong
    (time filtlong --min_length 5000 test.fq.gz > /dev/null) 2> ${i}_filtlong_gz_filt
    tail -1 ${i}_filtlong_gz_filt >> filtlong_gz_filt

    rm ${i}_*

done

mkdir replicate_benchmarks
mv filtlong_* nanoq_* nanostat_* replicate_benchmarks