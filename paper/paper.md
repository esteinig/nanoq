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

Nanopore sequencing is now routinely used in a variety of genomics applications, including whole genome assembly [@human_genome] and real-time infectious disease surveillance [@covid]. One of the first steps in many workflows is to assess the quality of reads and obtain basic summary statistics after basecalling signal data, as well as to filter fragmented or low quality reads. With increasing throughput on scalable nanopore platforms like `GridION` or `PromethION`, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. `Nanoq` is 2-3x faster than `rust-bio-tools` and up to 400x faster than commonly used alternatives (see benchmarks) and offers nanopore-specific read quality scores, common filtering options and output compression. `Nanoq` can therefore be effectively applied to nanopore data from the public domain, as part of automated pipelines, in streaming applications, or to check on the progress of active sequencing runs.

# Statement of need

 [`NanoPack`](https://github.com/wdecoster/nanopack) (`biopython`) [@nanopack], [`Filtlong`](https://github.com/rrwick/Filtlong) ([`Klib`](https://github.com/attractivechaos/klib)) and [`MinIONQC`](https://github.com/roblanf/minion_qc/blob/master/README.md) (basecalling summary) [@minionqc] are common tools used to filter and obtain summary statistics from nanopore reads. However, these tools may be bottlenecked during read iteration (`NanoPack`, `Filtlong`), not immediately applicable due to reliance on basecalling summary files (`MinIONQC`) or implement complex filters and data visualization for research applications. We wrote `nanoq` to accelerate quality control and summary statistics for large nanopore data sets.



# Applications

## Input / output

`Nanoq` accepts a `fast{a,q}.{gz,bz2,xz}` file (`-i`) or stream (`stdin`) and outputs reads to file (`-o`) or stream (`stdout`).

```bash
nanoq -i test1.fq.gz -o test2.fq
cat test1.fq.gz | nanoq > test2.fq
```

Output compression is inferred from file extensions (`gz`, `bz2`, `lzma`).

```bash
nanoq -i test1.fq -o test2.fq.gz
```

Output compression can be specified manually  (`-O`) at different compression levels (`-c`).

```bash
nanoq -i test1.fq -O g -c 9 -o test2.fq.cmp
```

## Read filters and summaries

Reads can be filtered by minimum read length (`-l`), maximum length (`-m`) or mean read quality (`-q`).

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


Summaries without read output can be obtained by directing to `/dev/null` or using the stats flag (`-s`):

```bash
nanoq -i test.fq > /dev/null
nanoq -i test.fq -s
```

Read qualities may be excluded from filters and statistics to speed up read iteration in some cases (`-f`).

```bash
nanoq -i test1.fq.gz -f -s
```


Extended summaries analogous to `NanoStat` can be obtained using multiple verbosity flags (`-v`) to show thresholded read counts or top ranking reads by length or quality (`-t`).

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

Benchmarks evaluate processing speed and memory consumption of a basic read length filter and summary statistics on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) [@zymo] (`GridION`) with comparisons to the commonly used tools [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong). Time to completion and maximum memory consumption were measured using `/usr/bin/time -f "%e %M"`, speedup is relative to the slowest command in the set. We note that summary statistics from `rust-bio-tools` do not compute read quality score and would therefore be comparable to `nanoq-fast`.

Tasks:

  * `stats`: basic read set summaries
  * `filter`: minimum read length filter (into `/dev/null`)

Tools:

* `rust-bio-tools=0.28.0`
* `nanostat=v1.5.0` 
* `nanofilt=2.8.0`
* `filtlong=0.2.1`
* `nanoq=0.7.0`

Commands used for `stats` task:

  * `nanostat` (fq + fq.gz)  --> `NanoStat --fastq test.fq --threads 1` 
  * `nanostat-t8` (fq + fq.gz) --> `NanoStat --fastq test.fq --threads 8` 
  * `rust-bio` (fq) --> `rbt sequence-stats --fastq < test.fq`
  * `rust-bio` (fq.gz) --> `zcat test.fq.gz | rbt sequence-stats --fastq`
  * `nanoq` (fq + fq.gz) --> `nanoq --input test.fq --stats` 
  * `nanoq-fast` (fq + fq.gz) --> `nanoq --input test.fq --stats --fast` 

Commands used for `filter` task:

  * `filtlong` (fq + fq.gz) --> `filtlong --min_length 5000 test.fq > /dev/null`  
  * `nanofilt` (fq) --> `NanoFilt --fastq test.fq --length 5000 > /dev/null` 
  * `nanofilt` (fq.gz) --> `gunzip -c test.fq.gz | NanoFilt --length 5000 > /dev/null` 
  * `nanoq` (fq + fq.gz) --> `nanoq --input test.fq --min-len 5000 > /dev/null` 
  * `nanoq-fast` (fq + fq.gz) --> `nanoq --input test.fq --min-len 5000 --fast > /dev/null` 

Files:

  * `zymo.fq`: uncompressed (100,000 reads, ~1 Gbp)
  * `zymo.fq.gz`: compressed (100,000 reads, ~1 Gbp)
  * `zymo.full.fq`: uncompressed (3,491,078 reads, ~14 Gbp)

Data preparation:

```bash
wget "https://nanopore.s3.climb.ac.uk/Zymo-GridION-EVEN-BB-SN.fq.gz"
zcat Zymo-GridION-EVEN-BB-SN.fq.gz > zymo.full.fq
head -400000 zymo.full.fq > zymo.fq && gzip -k zymo.fq
```

Elapsed real time and maximum resident set size:

```bash
echo '#!/bin/bash \n /usr/bin/time -f "%e %M" $@' > /usr/bin/t
```

Task and command execution:

Commands were run in replicates of 10 in the provided `Docker` container with a mounted benchmark data volume. An additional cold start iteration was not considered in the final benchmarks. 

```bash
for i in {1..11}; do
  for f in /data/*.fq; do 
    t nanoq -f- s -i $f 2> benchmark
    tail -1 benchmark >> rbt_stat_fq
  done
done
```

## Stats task benchmark

![Nanoq benchmarks on 100,000 reads of the Zymo mock community compared to Filtlong and Nanopack (10 replicates)](benchmarks.png?raw=true "Nanoq benchmarks" )

### `zymo.fq`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat-t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| rust-bio        | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq-fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


### `zymo.fq.gz`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat-t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| rust-bio        | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq-fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


### `zymo.full.fq`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanostat        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanostat-t8     | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| rust-bio        | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq-fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


## Filter task benchmark

![Nanoq benchmarks on 100,000 reads of the Zymo mock community compared to Filtlong and Nanopack (10 replicates)](benchmarks.png?raw=true "Nanoq benchmarks" )


### `zymo.fq`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |



### `zymo.fq.gz`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |


### `zymo.full.fq`

| command         | mem (sd)  | sec (sd)       |  reads / sec    | speedup |
| ----------------|-----------|----------------|-----------------|---------|
| nanofilt        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| filtlong        | fq.gz     | 42.21 (0.37)   | 2,369           | 1.00 x  |
| nanoq           | fq.gz     | 06.30 (0.28)   | 15,873          | 6.70 x  |
| nanoq-fast      | fq.gz     | 03.57 (0.57)   | 100,000         | 10.4 x  |

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through:

* Cargo: `cargo install nanoq`
* Conda: `conda install -c bioconda nanoq`
* Docker: `docker pull esteinig/nanoq`

`Nanoq` is integrated with [pipelines servicing research projects](https://github.com/np-core) at [Queensland Genomics](https://queenslandgenomics.org/clinical-projects-3/north-queensland/) using nanopore sequencing to detect infectious agents in septic patients, reconstruct transmission dynamics of bacterial pathogens, and conduct outbreak sequencing.

# Acknowledgements

We would like to thank the `OneCodex` team for developing `needletail` and Michael Hall (@mbhall88) for compression inspiration from [Rasusa](https://github.com/mbhall88/rasusa). ES was funded by Genomics and a grant by HOT NORTH,  the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932) and Queensland Genomics.

# References

