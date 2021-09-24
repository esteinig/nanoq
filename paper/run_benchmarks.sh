#!/bin/bash

# alias t='/usr/bin/time -f "%e %M"' 

cd /benchmarks

for file in /data/zymo.fq /data/zymo.fq.gz /data/zymo.full.fq; do
    for i in $(seq 1 10); do
        ext="${file#*.}"

        echo "File: $file Replicate: $i"

        echo "Nanoq stats"
        t nanoq -i $file -s 2> ${i}_nanoq_stat
        tail -1 ${i}_nanoq_stat >> nanoq_stat_${ext}

        echo "Nanoq stats fast"
        t nanoq -i $file -f -s 2> ${i}_nanoq_fstat
        tail -1 ${i}_nanoq_stat >> nanoqf_stat_${ext}

        echo "Nanostat stats"
        t NanoStat --fastq $file -t 1 2> ${i}_nanostat_stat
        tail -1 ${i}_nanostat_stat >> nanostat_stat_${ext}

        echo "Nanostat stats"
        t NanoStat --fastq $file -t 8 2> ${i}_nanostat_stat
        tail -1 ${i}_nanostat_stat >> nanostatf_stat_${ext}
        
        echo "Nanoq read length filter"
        (t nanoq -i $file -l 5000 > /dev/null) 2> ${i}_nanoq_filt
        tail -1 ${i}_nanoq_filt >> nanoq_filt_${ext}

        echo "Nanoq fast read length filter"
        (t nanoq -i $file -f -l 5000 > /dev/null) 2> ${i}_nanoq_filt
        tail -1 ${i}_nanoq_filt >> nanoqf_filt_${ext}

        echo "Nanofilt read length filter"
        if [[ $ext == "fq.gz" ]]; then
            (t gunzip -c $file | NanoFilt -l 5000 > /dev/null) 2> ${i}_nanofilt_filt
        else
            (t NanoFilt -l 5000 $file > /dev/null) 2> ${i}_nanofilt_filt
        fi
        tail -1 ${i}_nanofilt_filt >> nanofilt_filt_${ext}

        echo "Filtlong read length filter"
        (t filtlong --min_length 5000 $file> /dev/null) 2> ${i}_filtlong_filt
        tail -1 ${i}_filtlong_filt >> filtlong_filt_${ext}

        rm ${i}_*

    done
done
