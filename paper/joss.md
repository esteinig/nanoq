---
title: 'Nanoq: fast summary and quality control for nanopore reads'
tags:
  - nanopore
  - filter 
  - summary
  - statistics
  - reads
  - length
  - quality
  - ont
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

Nanopore sequencing is now routinely integrated in a variety of genomics applications, including whole genome assembly [@human_genome] and real-time infectious disease surveillance [@covid]. As a consequence, the amount of nanopore sequence data in the public domain has increased rapidly in the last few years. One of the first steps in any workflow is to assess the quality of reads and obtain basic summary statistics after basecalling raw nanopore signal, and to filter low quality reads. [`NanoPack`](https://github.com/wdecoster/nanopack) (backend: `biopython` ) [@nanopack] [@biopython], [`Filtlong`](https://github.com/rrwick/Filtlong) (backend: [`Klib`](https://github.com/attractivechaos/klib)) and [`MinIONQC`](https://github.com/roblanf/minion_qc/blob/master/README.md) (backend: sequencing summary file) [@minionqc] are common tools used to filter and obtain summary statistics from nanopore reads. However, these tools can be relatively slow due to bottlenecks in read parsing (`NanoPack`, `Filtlong`) or not immediately usable due to reliance on a sequencing summary file (`MinIONQC`). We therefore implement `nanoq`, a fast summary and quality control tool for nanopore reads in Rust. 

# Statement of Need

A common practice for quality control and filtering of reads for length and quality is to use a sequencing summary file as index to accelerate iteration and computation of statistics over millions of individual reads and their precomputed summary metrics from basecalling, requiring access to the signal level data or shared summary files. With increasing throughput on scalable nanopore platforms like GridION or PromethION, fast quality control of sequence reads and the ability to generate summary statistics on-the-fly are required. `Nanoq` is highly competitive in processing speed (see benchmarks) and can be effectively applied to nanopore data from the public domain, where basecalling indices are unavailable, as part of automated pipelines processing, in streaming applications, or directly from the command line to check on the progress of active sequencing runs.

# Methodology

`Nanoq` is implemented in Rust using the `fast{a/q}` backends from [`needletail`](https://github.com/onecodex/needletail) and [`Rust-Bio`](https://github.com/rust-bio/rust-bio) [@rustbio]. Tests can be run within the repository:

```
cargo test
```

`Nanoq` accepts a stream of sequence reads (`fast{a/q}`, `.gz`) with ouput of summary statistics to `stderr`:

```bash
cat test.fq | nanoq
```

Output statistics are in order: 

* number of reads
* number of base pairs
* N50 read length
* longest and shorted reads
* mean and median read length
* mean and median read quality 

```bash
100000 400398234 5154 44888 5 4003 3256 8.90 9.49
```

Extended output analogous to `NanoStat` can be obtained using the `--detail` flag:

```bash
cat test.fq | nanoq -d
```


Reads filtered by minimum read length (`--length`) and mean read quality (`--quality`)  are output to `stdout`:

```bash
cat test.fq | nanoq -l 1000 -q 10 > reads.fq 
```

Advanced filtering analogous to `Filtlong` removes the worst 20% of bases by reads (`--keep_percent`) and the worst quality reads (`--keep_bases`) until 500 Mbp remain:

```bash
cat test.fq | nanoq -p 80 -b 500000000  > reads.fq 
```

# Other Applications

Live sequencing run checks:

```bash
RUN=/data/nanopore/run
```

Check total run statistics:

```bash
find $RUN -name *.fastq -print0 | xargs -0 cat | nanoq
```

Check per-barcode statistics:

```bash
for i in {01..12}; do
  find $RUN -name barcode${i}.fastq -print0 | xargs -0 cat | nanoq
done
```

# Benchmarks


Benchmarks evaluate processing speed of a simple read-length filter and computation of summary statistics on the first 100,000 reads of the [Zymo mock community](https://github.com/LomanLab/mockcommunity) [@zymo] running comparisons to [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong).

Summary statistic commands:


Filter commands:


While the `rust-bio` parser is slightly faster than `needletail` in these specific benchmarks, `needletail` is the default mode as it supports `gz` and `fasta` formats natively.

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through:

* Cargo: `cargo install nanoq`
* Docker: `docker pull esteinig/nanoq`
* BioConda: `conda install -c bioconda nanoq`
* Singularity: `singularity pull docker://esteinig/nanoq`

`Nanoq` is currently integrated into pipelines servicing [Queensland Genomics](https://github.com/np-core) projects using nanopore sequencing to detect infectious agents in sepsis patients, conduct regional surveillance of bacterial diseases, and reconstruct their transmission dynamics.

# Acknowledgements

We would like to thank the `Rust-Bio` and `OneCodex` teams for developing the read parsers and making them available to the bioinformatics community. ES was funded by Queensland Genomics and a joint grant by HOT NORTH and the Center for Policy Relevant Infectious Disease Simulation and Mathematical Modelling  (NHMRC: #1131932)


# References

