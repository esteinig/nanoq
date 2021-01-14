---
title: 'Nanoq: fast summary and quality control for nanopore reads'
tags:
  - nanopore
  - filter 
  - fastq
  - reads
  - length
  - quality
  - ont
authors:
 - name: Eike Steinig
   orcid: 
   affiliation: 
affiliations:
 - name: The Peter Doherty Institute for Infection and Immunity, Melbourne University, Australia
   index: 1
date: 11 July 2020
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely applied to a variety of genomic applications, ranging from whole genome assembly [`Human ONT CITATION`] to real-time, genome-informed infectious disease surveillance [`Loman Gardy CITATION`, `Estee CITATION`]. As a consequence, the amount of nanopore sequence data in the public domain has increased rapidly in the last few years. One of the first steps during and after sequencing is to assess the quality of sequence reads and obtain basic summary statistics after basecalling raw signal, and to filter low quality reads. A common practice for quality control and filtering of reads for length and quality is to use the basecalling summary file as index to accelerate iteration over millions of individual reads and their quality scores (requiring access to the signal level data or shared files). With increasing throughput on scalable nanopore platforms, fast processing of sequence reads and the ability to generate summary statistics of sequence read files without access to their indices are required. [`NanoStat`](https://github.com/wdecoster/nanostat), [`NanoFilt`](https://github.com/wdecoster/nanofilt) (using the `biopython` backend) [`NANOPACK CITATION`, `BIOPYTHON CITATION`] and [`Filtlong`](https://github.com/rrwick/Filtlong) (using the [`Klib`](https://github.com/attractivechaos/klib/) backend in C) are classic tools used to filter and obtain summary statistics from nanopore data. Here, we implement `nanoq`, a minimal but fast summary and quality control tool for nanopore reads in Rust. `Nanoq` is highly competitive in iteration over sequence reads and can be effectively applied to nanopore data from the public domain, where basecalling indices are unavailable, as part of automated pipelines processing, or directly from the command line to check on statistics of active sequencing runs.

# Methodology

`Nanoq` is implemented in Rust using the `fast{a/q}` parsers from the [`Rust-Bio`](https://github.com/rust-bio/rust-bio) [`RUSTBIO CITATION`] and [`needletail`](https://github.com/onecodex/needletail) libraries, which is used by `nanoq` by default and accepts a stream of sequence reads (`fast{a/q}`, `.gz`) with ouput of summary statistics to `stderr`:

### Summary statistics

```
cat test.fq | nanoq
```

```

```


Detailed summary output analogous to `NanoStat`. Top summary block describes order of fields in `stderr` output above:

```
cat test.fq | nanoq -d
```

```
Reads:
Bases:
N50:
Longest:
Shortest:
Mean length:
Median length:
Mean Q score:
Median Q score:

Read thresholds:
  1. 
  2.
  3.
  4.
  5.

Longest reads:
  1. 
  2.
  3.
  4.
  5.

Quality reads:
  1.
  2.
  3.
  4.
  5.

```

### Read filters

Filtered by minimum read length (`-l`) and quality (`-q`), reads are output to `stdout`:

```
cat test.fq | nanoq -l 1000 -q 10 > filtered_reads.fq 
```

Advanced filtering analogous to `Filtlong` with a two-pass filter on 80% of the best quality bases, as well as removing the worst quality reads until 500 Mbp of bases remain:

```
cat test.fq | nanoq --percent_bases 80 --bases 500000000  > filtered_reads.fq 
```

# Other Applications

Live sequencing run checks:

```bash
RUN=/nanopore/data/run
```

Check total run statistics:

```bash
find $RUN -name *.fastq -print0 | xargs -0 cat | nanoq
```

Check per-barcode statistics during a live run:

```bash
for i in {01..12}; do
  find $RUN -name *barcode${i}*.fastq -print0 | xargs -0 cat | nanoq
done
```

# Benchmarks


Benchmarks evaluate processing speed of a long-read filter and computation of summary statistics on 100,000 reads of the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) ( `GridION`) using the `Benchmark` Docker image running comparisons to [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong).

Summary statistic commands:

Filter commands:


While the `rust-bio` parser is slightly faster in these benchmarks for these specific applications than `needletail`, the default mode for `nanoq` uses `needletail` due to its native capacity to parse gzipped and fasta type input formats.

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through Cargo (`cargo install nanoq`), as Docker (`docker pull esteinig/nanoq`) or Singularity container (`singularity pull docker://esteinig/nanoq`) or through BioConda (`conda install -c bioconda nanoq`).

# Acknowledgements

Nameless server #1

# References

