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
  - name: The Peter Doherty Institute for Infection and Immunity, Melbourne University, Australia
    index: 1
date: 11 January 2021
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely integrated in a variety of genomics applications, including whole genome assembly[@human_genome] and real-time infectious disease surveillance [@covid]. As a consequence, the amount of nanopore sequence data in the public domain has increased rapidly in the last few years. One of the first steps in any workflow is to assess the quality of reads and obtain basic summary statistics after basecalling raw nanopore signal, and to filter low quality reads. [`NanoPack`](https://github.com/wdecoster/nanopack) (`biopython` parser) [@nanopack; @biopython], [`Filtlong`](https://github.com/rrwick/Filtlong) ([`Klib`](https://github.com/attractivechaos/klib) parser) and [`MinIONQC`](https://github.com/roblanf/minion_qc/blob/master/README.md) (summary file parser) [@minionqc] are common tools used to filter and obtain summary statistics from nanopore reads. However, these tools can be relatively slow due to bottlenecks in read parsing (`NanoPack`, `Filtlong`), i.e. iteration speed over the reads while extracting relevant information, are not immediately usable due to reliance on summary files (`MinIONQC`), or focus on data exploration and visualization. We therefore implement `nanoq`, a command line tool to accelerate summary and quality control for nanopore reads in *Rust*. 

# Statement of Need

A common practice for quality control and filtering of reads for length and quality is to use a sequencing summary file as index to speed up iteration and computation over millions of individual reads and their precomputed metrics from the basecalling process (e.g. the main acess mode for `MinIONQC`), which requires access to signal level data or shared summary files. With increasing throughput on scalable nanopore platforms like GridION or PromethION, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. `Nanoq` is highly competitive in processing speed (see benchmarks) and can be effectively applied to nanopore data from the public domain, where sequencing summaries are unavailable, as part of automated pipelines, in streaming applications, or directly from the command line to check on the progress of active sequencing runs.

# Applications

`Nanoq` is implemented in *Rust* using the read parsers from [`needletail`](https://github.com/onecodex/needletail) and [`Rust-Bio`](https://github.com/rust-bio/rust-bio) [@rustbio].

Tests can be run within the `nanoq` repository:

```
cargo test
```

`Nanoq` accepts a file or stream of sequence reads in `fast{a/q}` and compressed formats on `stdin`:

```bash
cat test.fq | nanoq
```

Basic summary statistics are output to `stderr`: 

```bash
100000 400398234 5154 44888 5 4003 3256 8.90 9.49
```

* number of reads
* number of base pairs
* N50 read length
* longest and shorted reads
* mean and median read length
* mean and median read quality 

Extended output analogous to `NanoStat` can be obtained using multiple `--detail` flags:

```bash
cat test.fq | nanoq -d -d -d
```

Reads filtered by minimum read length (`--length`) and mean read quality (`--quality`) are output to `stdout`:

```bash
cat test.fq | nanoq -l 1000 -q 10 > reads.fq 
```

Advanced two-pass filtering analogous to `Filtlong` removes the worst 20% of bases using sorted reads by quality (`--keep_percent`) or the worst quality reads until approximately 500 Mbp remain (`--keep_bases`): 

```bash
nanoq -f test.fq -p 80 -b 500000000  > reads.fq 
```

Live sequencing run data directory:

```bash
RUN=/data/nanopore/run
```

Check total run statistics of active run:

```bash
find $RUN -name *.fastq -print0 | xargs -0 cat | nanoq
```

Check per-barcode statistics of active run:

```bash
for i in {01..12}; do
  find $RUN -name barcode${i}.fastq -print0 | xargs -0 cat | nanoq
done
```

# Benchmarks

Benchmarks evaluate processing speed of a long-read filter and computation of summary statistics on the first 100,000 reads (`test.fq.gz` in Docker container) of the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) (`GridION`) using the `nanoq:v0.2.0` [`Benchmark`](paper/Benchmarks) image with comparison to [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong)

![nanoq benchmarks](benchmarks.png?raw=true "Nanoq benchmarks" )

| program         |  task  | mean sec (+/- sd)   |  ~ reads / sec  | speedup |
| -------------   | -------|---------------------|-----------------|---------|
| nanofilt        | filter | 35.42 (0.396)       | 2,283           | 1.00 x  |
| filtlong        | filter | 20.28 (0.396)       | 4,930           | 2.15 x  |
| nanoq           | filter |  05.01 (1.442)      | 19,960          | 8.74 x  |
| nanostat        | stats  | 40.01 (2.649)       | 2,499           | 1.00 x  |
| nanoq           | stats  | 04.93 (1.441)       | 20,283          | 8.11 x  |

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through:

* Cargo: `cargo install nanoq`
* Docker: `docker pull esteinig/nanoq`
* BioConda: `conda install -c bioconda nanoq`
* Singularity: `singularity pull docker://esteinig/nanoq`

`Nanoq` is integrated with [pipelines servicing research projects](https://github.com/np-core) at [Queensland Genomics](https://queenslandgenomics.org/clinical-projects-3/north-queensland/) using nanopore sequencing to detect infectious agents in septic patients, reconstruct transmission dynamics of bacterial pathogens, and conduct outbreak sequencing at the Townsville University Hospital (QLD, Australia).

# Acknowledgements

We would like to thank the `Rust-Bio` community, the [`seq_io`](https://github.com/markschl/seq_io) project and the `OneCodex` team for developing the *Rust* read parsing crates. ES was funded by the Queensland Genomics Far North Queensland project and a joint grant by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling (NHMRC: #1131932).

# References

