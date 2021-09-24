---
title: 'Nanoq: fast quality control for nanopore reads'
tags:
  - ont
  - nanopore
  - reads
  - filter 
  - summary
  - statistics
  - length
  - quality
authors:
  - name: Eike Steinig
    orcid: 0000-0001-5399-5353
    affiliation: 1
  - name: Lachlan Coin
    orcid: 0000-0002-4300-455X
    affiliation: 1
affiliations:
  - name: The Peter Doherty Institute for Infection and Immunity, The University of Melbourne, Australia
    index: 1
date: 23 September 2021
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely used in a variety of applications, including whole genome assembly [@human_genome] and real-time infectious disease surveillance [@covid]. As a consequence, the amount of nanopore sequence data in the public domain has increased rapidly in the last few years. One of the first steps in any workflow is to assess the quality of reads and obtain basic summary statistics after basecalling raw nanopore signal data, as well as to filter fragmented or low quality reads. [`NanoPack`](https://github.com/wdecoster/nanopack) (`biopython` parser) [@nanopack], [`Filtlong`](https://github.com/rrwick/Filtlong) ([`Klib`](https://github.com/attractivechaos/klib) parser) and [`MinIONQC`](https://github.com/roblanf/minion_qc/blob/master/README.md) (summary file parser) [@minionqc] are common tools used to filter and obtain summary statistics from nanopore reads. However, these tools may be bottlenecked during read iteration (`NanoPack`, `Filtlong`), not immediately applicable due to reliance on summary files (`MinIONQC`) or implement complex filters and data visualization. We therefore wrote `nanoq`, a minimal command line application to accelerate quality control and read summaries for large nanopore data sets.

# Statement of need

A common practice for quality control and filtering of nanopore reads is to use a sequencing summary index to speed up iteration over millions of reads, which requires access to the original files. With increasing throughput on scalable nanopore platforms like `GridION` or `PromethION`, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. `Nanoq` is highly competitive in processing speed (see benchmarks) and can be effectively applied to nanopore data from the public domain, as part of automated pipelines, in streaming applications, or from the command line to check on the progress of active sequencing runs.

# Applications

## Input / output

`Nanoq` accepts a `fast{a,q}.{gz,bz2,xz}` file (`-i / --input`) or stream (`stdin`) and outputs reads to file (`-o / --output`) or stream (`stdout`).

```bash
nanoq -i test1.fq.gz -o test2.fq
cat test1.fq.gz | nanoq > test2.fq
```

Output compression is inferred from file extensions (`gz`, `bz2`, `lzma`).

```bash
nanoq -i test1.fq -o test2.fq.gz
```

Output compression can be specified manually  (`-O / --output-type`) at different levels (`-c / --compress-level`).

```bash
nanoq -i test1.fq -O g -c 9 -o test2.fq.cmp
```

## Read filters and summaries

Reads can be filtered by minimum read length (`-l / --min-len`), maximum length (`-m / --max-len`) or mean read quality (`-q / --min-qual`).

```bash
nanoq -i test.fq -l 1000 -q 10 -m 10000 > reads.fq 
```

Read set summary statistics are output to `stderr`: 

```bash
100000 400398234 5154 44888 5 4003 3256 8.90 9.49
```

* number of reads
* number of base pairs
* read length N50
* longest and shorted reads
* mean and median read length
* mean and median read quality 


Summaries without read output can be obtained by directing to `/dev/null` or using the stats flag (`-s / --stats`):

```bash
nanoq -i test.fq > /dev/null
nanoq -i test.fq -s
```

Read qualities may be excluded from filters and statistics to speed up read iteration in some cases (`-f / --fast`).

```bash
nanoq -i test1.fq.gz -f -s
```


Extended summaries analogous to `NanoStat` can be obtained using multiple flags (`-v / --verbose`) to show thresholded read counts and top ranking reads by length or quality (`-t / --top`).

```bash
nanoq -i test.fq -t 5 -f -s -vvv
```

```
Nanoq Read Summary
====================

Number of reads:      100000
Number of bases:      400398234
N50 read length:      5154
Longest read:         44888 
Shortest read:        5
Mean read length:     4003
Median read length:   3256 
Mean read quality:    NaN 
Median read quality:  NaN


Read length thresholds (bp)

> 200       99104             99.1%
> 500       96406             96.4%
> 1000      90837             90.8%
> 2000      73579             73.6%
> 5000      25515             25.5%
> 10000     4987              05.0%
> 30000     47                00.0%
> 50000     0                 00.0%
> 100000    0                 00.0%
> 1000000   0                 00.0%


Top ranking read lengths (bp)

1. 44888       
2. 40044       
3. 37441       
4. 36543       
5. 35630
```


## Active run inspection

`Nanoq` can be used to check on active sequencing runs and barcoded samples.

```bash
find /data/nanopore/run -name *.fastq -print0 | xargs -0 cat | nanoq -s
```

```bash
for i in {01..12}; do
  find /data/nanopore/run -name barcode${i}.fastq -print0 | xargs -0 cat | nanoq -s
done
```

# Benchmarks

Benchmarks evaluate processing speed and memory consumption of a basic read length filter and summary statistics on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) [@zymo] (`GridION`) with comparisons to the commonly used tools [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong). Time to completion and maximum memory consumption were measured using `/usr/bin/time -f "%e %M"`, speedup is relative to the slowest command in the set.

Tasks:

  * `stats`: basic read set summaries
  * `filter`: minimum read length filter (into `/dev/null`)

Files:
  * `zymo.fq`: uncompressed (100,000 reads)
  * `zymo.fq.gz`: compressed (100,000 reads)
  * `zymo.full.fq`: uncompressed (3,491,078 reads)

Commands used for `stats` task:

  * `nanostat` (fq + fq.gz)  --> `NanoStat --fastq test.fq --threads 1` 
  * `nanostat t8` (fq + fq.gz) --> `NanoStat --fastq test.fq --threads 8` 
  * `nanoq` (fq + fq.gz) --> `nanoq --input test.fq --stats` 
  * `nanoq fast` (fq + fq.gz) --> `nanoq --input test.fq --stats --fast` 

Commands used for `filter` task:

  * `filtlong` (fq + fq.gz) --> `filtlong --min_length 5000 test.fq > /dev/null`  
  * `nanofilt` (fq) --> `NanoFilt --fastq test.fq --length 5000 > /dev/null` 
  * `nanofilt` (fq.gz) --> `gunzip -c test.fq.gz | NanoFilt --length 5000 > /dev/null` 
  * `nanoq` (fq + fq.gz) --> `nanoq --input test.fq --min-len 5000 > /dev/null` 
  * `nanoq fast` (fq + fq.gz) --> `nanoq --input test.fq --min-len 5000 --fast > /dev/null` 



## Zymo: 100,000 reads (~ 1 Gbp)

![Nanoq benchmarks on 100,000 reads of the Zymo mock community compared to Filtlong and Nanopack (10 replicates)](benchmarks.png?raw=true "Nanoq benchmarks" )

### Stats task uncompressed (fq):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


### Stats task compressed (fq.gz):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |



### Filter task uncompressed (fq):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |



### Filter task compressed (fq.gz):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


## Zymo: 3,491,078 reads (~ 14 Gbp)

![Nanoq benchmarks on 100,000 reads of the Zymo mock community compared to Filtlong and Nanopack (10 replicates)](benchmarks.png?raw=true "Nanoq benchmarks" )

### Stats task uncompressed (fq):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


### Filter task uncompressed (fq):

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through:

* Cargo: `cargo install nanoq`
* Conda: `conda install -c bioconda nanoq`
* Docker: `docker pull esteinig/nanoq`

`Nanoq` is integrated with [pipelines servicing research projects](https://github.com/np-core) at [Queensland Genomics](https://queenslandgenomics.org/clinical-projects-3/north-queensland/) using nanopore sequencing to detect infectious agents in septic patients, reconstruct transmission dynamics of bacterial pathogens, and conduct outbreak sequencing.

# Acknowledgements

We would like to thank the `OneCodex` team for developing `needletail` and Michael Hall (@mbhall88) for compression inspiration from [Rasusa](https://github.com/mbhall88/rasusa). ES was funded by Queensland Genomics: Far North Queensland and a joint grant by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932).

# References

