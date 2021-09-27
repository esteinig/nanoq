---
title: 'Nanoq: ultra-fast quality control for nanopore reads'
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

Nanopore sequencing is now routinely used in a variety of genomics applications, including whole genome assembly [@human_genome] and real-time infectious disease surveillance [@covid]. One of the first steps in many workflows is to assess the quality of reads and to obtain summary statistics, as well as to filter fragmented or low quality reads. With increasing throughput on scalable nanopore platforms like `GridION` or `PromethION`, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. When computing minimum summary statistics, `Nanoq` is ~2x faster than `rust-bio-tools` (~5 Gbp and >1 million reads per second) and up to ~440x faster than commonly used alternatives. It offers nanopore-specific quality scores, read filtering options and output compression. `Nanoq` can be applied to data from the public domain, as part of automated pipelines, in streaming applications, or to rapidly check progress of active sequencing runs.

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

Read qualities may be excluded from filters and statistics to speed up read iteration (`-f`).

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
find /data/nanopore/run -name *.fastq -print0 | xargs -0 cat | nanoq -f -s
```

```bash
for i in {01..12}; do
  find /data/nanopore/run -name barcode${i}.fastq -print0 | xargs -0 cat | nanoq -f -s
done
```

# Benchmarks

Benchmarks evaluate processing speed and memory consumption of a basic read length filter and summary statistics on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) [@zymo] (`GridION`) with comparisons to [`rust-bio-tools`](https://github.com/rust-bio/rust-bio-tools), [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong). Time to completion and maximum memory consumption were measured using `/usr/bin/time -f "%e %M"`, speedup is relative to the slowest command in the set. We note that summary statistics from `rust-bio-tools` do not compute read quality score and are therefore comparable to `nanoq-fast`.

Tasks:

  * `stats`: basic read set summaries
  * `filter`: minimum read length filter (into `/dev/null`)

Tools:

* `rust-bio-tools=0.28.0`
* `nanostat=1.5.0` 
* `nanofilt=2.8.0`
* `filtlong=0.2.1`
* `nanoq=0.8.0`

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

  * `zymo.fq`: uncompressed (100,000 reads, ~400 Mbp)
  * `zymo.fq.gz`: compressed (100,000 reads, ~400 Mbp)
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

Commands were run in replicates of 10 with a mounted benchmark data volume in the provided `Docker` container. An additional cold start iteration for each command was not considered in the final benchmarks. 

```bash
for i in {1..11}; do
  for f in /data/*.fq; do 
    t nanoq -f- s -i $f 2> benchmark
    tail -1 benchmark >> nanoq_stat_fq
  done
done
```

## Benchmark results

![Nanoq benchmarks on 3.5 million reads of the Zymo mock community (10 replicates)](benchmarks_zymo_full.png?raw=true "Nanoq benchmarks full" )

### `stats` + `zymo.full.fq`

| command         | mem (sd)         | sec (sd)           |  reads / sec    | speedup  |
| ----------------|------------------|--------------------|-----------------|----------|
| nanostat        | 741.4 (0.09)     | 1260. (13.9)       | 2,770           | 01.00 x  |
| nanostat-t8     | 741.4 (0.10)     | 1249. (9.12)       | 2,795           | 01.00 x  |
| nanoq           | 35.83 (0.06)     | 94.51 (0.43)       | 36,938          | 13.34 x  |
| rust-bio        | 43.20 (0.08)     | 06.54 (0.05)       | 533,803         | 192.7 x  |
| nanoq-fast      | **22.18** (0.07) | **02.85** (0.02)   | 1,224,939       | 442.1 x  |

### `filter` + `zymo.full.fq`

| command         | mb (sd)           | sec (sd)           |  reads / sec    | speedup  |
| ----------------|-------------------|--------------------|-----------------|----------|
| nanofilt        | 67.47 (0.13)      | 1160. (20.2)       | 3,009           | 01.00 x  |
| filtlong        | 1516. (5.98)      | 420.6 (4.53)       | 8,360           | 02.78 x  |
| nanoq           | 11.93 (0.06)      | 94.93 (0.45)       | 36,775          | 12.22 x  |
| nanoq-fast      | **08.05** (0.05)  | **03.90** (0.30)   | 895,148         | 297.5 x  |


![Nanoq benchmarks on 100,000 reads of the Zymo mock community (10 replicates)](benchmarks_zymo.png?raw=true "Nanoq benchmarks subset" )

### `stats` + `zymo.fq`

| command         | mem (sd)         | sec (sd)           |  reads / sec    | speedup  |
| ----------------|------------------|--------------------|-----------------|----------|
| nanostat        | 79.64 (0.14)     | 36.22 (0.27)       | 2,760           | 01.00 x  |
| nanostat-t8     | 79.53 (0.24)     | 36.06 (0.34)       | 2,776           | 01.00 x  |
| nanoq           | 04.26 (0.09)     | 02.69 (0.02)       | 37,147          | 13.46 x  |
| rust-bio        | 16.61 (0.08)     | 00.22 (0.00)       | 100,000         | 36.23 x  |
| nanoq-fast      | **03.81** (0.05) | **00.08** (0.00)   | 100,000         | 36.23 x  |


### `stats` + `zymo.fq.gz`

| command         | mem (sd)         | sec (sd)           |  reads / sec    | speedup  |
| ----------------|------------------|--------------------|-----------------|----------|
| nanostat        | 79.46 (0.22)     | 40.98 (0.31)       | 2,440           | 01.00 x  |
| nanostat-t8     | 79.43 (0.18)     | 41.01 (0.30)       | 2,438           | 01.00 x  |
| nanoq           | 04.44 (0.09)     | 05.74 (0.04)       | 17,421          | 07.14 x  |
| rust-bio        | **01.59** (0.06) | 05.06 (0.04)       | 19,762          | 08.09 x  |
| nanoq-fast      | 03.95 (0.07)     | **03.15** (0.02)   | 31,746          | 13.01 x  |

### `filter` + `zymo.fq`

| command         | mb (sd)           | sec (sd)           |  reads / sec    | speedup  |
| ----------------|-------------------|--------------------|-----------------|----------|
| nanofilt        | 66.29 (0.15)      | 33.01 (0.24)       | 3,029           | 01.00 x  |
| filtlong        | 274.5 (0.04)      | 08.49 (0.01)       | 11,778          | 03.89 x  |
| nanoq           | 03.61 (0.04)      | 02.81 (0.28)       | 35,587          | 11.75 x  |
| nanoq-fast      | **03.26** (0.06)  | **00.12** (0.01)   | 100,000         | 33.01 x  |

### `filter` + `zymo.fq.gz`

| command         | mb (sd)           | sec (sd)           |  reads / sec    | speedup  |
| ----------------|-------------------|--------------------|-----------------|----------|
| nanofilt        | **01.57** (0.07)  | 33.48 (0.35)       | 2,986           | 01.00 x  |
| filtlong        | 274.2 (0.04)      | 16.45 (0.09)       | 6,079           | 02.04 x  |
| nanoq           | 03.68 (0.06)      | 05.77 (0.04)       | 17,331          | 05.80 x  |
| nanoq-fast      | 03.45 (0.07)      | **03.20** (0.02)   | 31,250          | 10.47 x  |


# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through:

* Cargo: `cargo install nanoq`
* Conda: `conda install -c bioconda nanoq`
* Docker: `docker pull esteinig/nanoq`

`Nanoq` is integrated with [pipelines servicing research projects](https://github.com/np-core) at [Queensland Genomics](https://queenslandgenomics.org/clinical-projects-3/north-queensland/) using nanopore sequencing to detect infectious agents in septic patients, reconstruct transmission dynamics of bacterial pathogens, and conduct outbreak sequencing.

# Acknowledgements

We would like to thank the `OneCodex` team for developing [`needletail`](htps://github.com/onecodex/needletail) and Michael Hall for code adoption from [Rasusa](https://github.com/mbhall88/rasusa). ES was funded by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932).

# References

