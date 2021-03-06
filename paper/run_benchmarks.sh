#!/bin/bash

# RUN inside container (~5 hours runtime)

cd /data

if [ ! -f "/data/test.fq" ]; then
    if [ ! -f "/data/test.fq.gz" ]; then
        echo "test.fq.gz is missing!"
        exit 1
    fi
    zcat test.fq.gz > test.fq
fi

TIMEFORMAT="%R"

for i in $(seq 1 100); do

    # Gzipped time replicate iteration
    echo "Replicate timer: $i"

    echo "Nanoq stats"

    # test file stat nanoq
    (time cat test.fq | nanoq) 2> ${i}_nanoq_stat
    tail -1 ${i}_nanoq_stat >> nanoq_fq_stat
    # test file gzipped stat nanoq
    (time cat test.fq.gz | nanoq) 2> ${i}_nanoq_gz_stat
    tail -1 ${i}_nanoq_gz_stat >> nanoq_gz_stat
    # test file stat nanoq crab cast
    (time cat test.fq | nanoq -c) 2> ${i}_nanoq_crab_stat
    tail -1 ${i}_nanoq_crab_stat >> nanoq_crab_stat

    echo "Nanostat stats"

    # test file stat nanostat 4 cpu
    (time NanoStat --fastq test.fq -t 4) 2> ${i}_nanostat_stat
    tail -1 ${i}_nanostat_stat >> nanostat_fq_stat
    # test file stat gzipped nanostat 4 cpu
    (time NanoStat --fastq test.fq.gz -t 4) 2> ${i}_nanostat_gz_stat
    tail -1 ${i}_nanostat_gz_stat >> nanostat_gz_stat

    
    echo "Nanofilt filters"

    # test file filt nanofilt
    (time cat test.fq | NanoFilt -l 5000 > /dev/null) 2> ${i}_nanofilt_filt
    tail -1 ${i}_nanofilt_filt >> nanofilt_fq_filt
    # test file filt gzipped nanofilt
    (time zcat test.fq.gz | NanoFilt -l 5000 > /dev/null) 2> ${i}_nanofilt_gz_filt
    tail -1 ${i}_nanofilt_gz_filt >> nanofilt_gz_filt

    echo "Nanoq filters"

    # test file filt nanoq
    (time cat test.fq | nanoq -l 5000 > /dev/null) 2> ${i}_nanoq_filt
    tail -1 ${i}_nanoq_filt >> nanoq_fq_filt
    # test file filt gzipped nanoq
    (time cat test.fq.gz | nanoq -l 5000 > /dev/null) 2> ${i}_nanoq_gz_filt
    tail -1 ${i}_nanoq_gz_filt >> nanoq_gz_filt 
    # test file filt gzipped nanoq crab cast (no native gz support)
    (time zcat test.fq.gz | nanoq -l 5000 -c > /dev/null) 2> ${i}_nanoq_crab_filt
    tail -1 ${i}_nanoq_crab_filt >> nanoq_crab_filt

    echo "Filtlong filters"

     # test file filt filtlong
    (time filtlong --min_length 5000 test.fq > /dev/null) 2> ${i}_filtlong_filt
    tail -1 ${i}_filtlong_filt >> filtlong_fq_filt
    # test file filt gzipped filtlong
    (time filtlong --min_length 5000 test.fq.gz > /dev/null) 2> ${i}_filtlong_gz_filt
    tail -1 ${i}_filtlong_gz_filt >> filtlong_gz_filt

    rm ${i}_*

done

mkdir replicate_benchmarks
mv filtlong_* nanoq_* nanostat_* nanofilt_* replicate_benchmarks