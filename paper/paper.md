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
date: 18 October 2021
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely used in a variety of genomics applications, including whole genome assembly [@human_genome] and real-time infectious disease surveillance [@covid]. One of the first steps in many workflows is to assess the quality of reads and to obtain summary statistics, as well as to filter fragmented or low quality reads. With increasing throughput on scalable nanopore platforms like `GridION` or `PromethION`, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. Benchmarks indicate that `nanoq` is about as fast as `seqtk fqchk` for small uncompressed data and ~1.5x as fast as `seqtk fqchk` for large uncompressed datasets. Without computing quality scores, `nanoq` is around ~2-3x faster than `rust-bio-tools` for summary statistics and up to ~450x faster than `NanoStats` (> 1.2 million reads per second). In read filtering applications, `nanoq` is faster than other commonly tools (`NanoFilt`, `Filtlong`). Memory consumption is consistent and tends to be lower than alternatives (~3-5x). `Nanoq` offers nanopore-specific quality scores, read filtering options and output compression. it can be applied to data from the public domain, as part of automated pipelines, in streaming applications, or to rapidly check progress of active sequencing runs.

# Statement of need

 [`NanoPack`](https://github.com/wdecoster/nanopack) (`biopython`) [@nanopack], [`Filtlong`](https://github.com/rrwick/Filtlong) ([`Klib`](https://github.com/attractivechaos/klib)) and [`MinIONQC`](https://github.com/roblanf/minion_qc/blob/master/README.md) (basecalling summary) [@minionqc] are common tools used to filter and obtain summary statistics from nanopore reads. However, their performance may be bottlenecked at read iteration (`NanoPack`, `Filtlong`), they may not immediately applicable due to reliance on basecalling summary files (`MinIONQC`) or implement more complex filters and data visualization for research applications (`NanoFilt`, `Filtlong`). We wrote `nanoq` to accelerate quality control and summary statistics for large nanopore data sets.

# Benchmarks

Benchmarks evaluate processing speed and memory consumption of a basic read length filter and summary statistics on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) [@zymo] (`GridION`) with comparisons to [`seqtk fqchk`](https://github.com/lh3/seqtk), [`rust-bio-tools`](https://github.com/rust-bio/rust-bio-tools), [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong). Time to completion and maximum memory consumption were measured using `/usr/bin/time -f "%e %M"`, speedup is relative to the slowest command in the set. We note that summary statistics from `rust-bio-tools` do not compute read quality score and are therefore comparable to `nanoq-fast`.

Tasks:

  * `stats`: basic read set summaries
  * `filter`: minimum read length filter (into `/dev/null`)

Tools:

* `rust-bio-tools=0.28.0`
* `nanostat=1.5.0` 
* `nanofilt=2.8.0`
* `filtlong=0.2.1`
* `seqtk=1.3-r126`
* `nanoq=0.8.2`

Commands used for `stats` task:

  * `nanostat` (fq + fq.gz)  --> `NanoStat --fastq test.fq --threads 1` 
  * `nanostat-t8` (fq + fq.gz) --> `NanoStat --fastq test.fq --threads 8` 
  * `seqtk-fqchk` (fq + fq.gz) --> `seqtk fqchk`
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
/usr/bin/time -f "%e %M"
```

Task and command execution:

Commands were run in replicates of 10 with a mounted benchmark data volume in the provided `Docker` container. An additional cold start iteration for each command was not considered in the final benchmarks. 

```bash
for i in {1..11}; do
  for f in /data/*.fq; do 
    /usr/bin/time -f "%e %M" nanoq -f- s -i $f 2> benchmark
    tail -1 benchmark >> nanoq_stat_fq
  done
done
```

## Benchmark results

![Nanoq benchmarks on 3.5 million reads of the Zymo mock community (10 replicates)](benchmarks_zymo_full.png?raw=true "Nanoq benchmarks full")

**Figure 1**: Nanoq benchmarks on 3.5 million reads of the Zymo mock community (10 replicates)

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

**Figure 2**: Nanoq benchmarks on 100,000 reads of the Zymo mock community (10 replicates)

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

`Nanoq` is integrated with [pipelines servicing research projects](https://github.com/np-core) at [Queensland Genomics](https://queenslandgenomics.org/clinical-projects-3/north-queensland/) using nanopore sequencing to detect infectious agents in septic patients, reconstruct transmission dynamics of bacterial pathogens, and conduct outbreak sequencing.

# Acknowledgements

<<<<<<< HEAD
We would like to thank the `OneCodex` team for developing [`needletail`](htps://github.com/onecodex/needletail), Luiz Irber and Pierre Marijon for developing [`niffler`](https://github.com/luizirber/niffler) and Michael Hall for code adoption from [Rasusa](https://github.com/mbhall88/rasusa). ES was funded by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932).
=======
We would like to thank the `OneCodex` team for developing [`needletail`](htps://github.com/onecodex/needletail), Luiz Irber for developing [`niffler`](https://github.com/luizirber/niffler) and Michael Hall for code adoption from [Rasusa](https://github.com/mbhall88/rasusa). ES was funded by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932).
>>>>>>> 1651769730ea3a253182338ebe950624f6577d7b

# References

