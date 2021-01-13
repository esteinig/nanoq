---
title: 'Nanoq: fast summary and quality control for nanopore reads'
tags:
  - Nanopore
  - Oxford Nanopore Technologies
  - filter 
  - fastq
  - reads
  - length
  - quality
authors:
 - name: Eike Steinig
   orcid: 
   affiliation: 
affiliations:
 - name: Australian Institute of Tropical Health and Medicine, Townsville, Queensland, Australia
   index: 1
date: 11 July 2020
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely applied to a variety of genomic applications, ranging from whole genome assembly to real-time, genome-informed infectious disease surveillance. As a consequence, the amount of nanopore sequence data in the public domain has increased rapidly in the last few years. One of the first steps during and after sequencing is to assess the quality of sequence reads and obtain basic summary statistics after basecalling raw signal, and if necessary filter spuriously low quality reads. A common practice for quality control and filtering of reads for length and quality is to use the basecalling summary file as index to accelerate iteration over millions of individual reads and their quality scores. With increasing throughput on nanopore platforms like the PromethION, fast processing of sequence reads and the ability to quickly generate summary statistics of read files are required. `NanoStat`, `NanoFilt`, and `Filtlong` are classic Python tools (wrapping C libraries `kseq`) commonly used to filter and obtain summary statistics from nanopore data. We implement `nanoq`, a fast but minimal summary and quality control tool for nanopore reads in `Rust`. `Nanoq` can be efficiently applied on sequence read data from public domain where basecalling summaries are unavailable, as part of automated pipelines processing nay nanopore read data, or directly from the command line to check on statistics of currently active sequencing runs.

# Methodology

`Nanoq` is implemented in `Rust` using the fastq parsers from [`rust-bio`](https://github.com/rust-bio/rust-bio) and [`needletail`](https://github.com/onecodex/needletail). `Nanoq` by accepts a stream of sequence reads (`fast{a/q}`, `.gz`) and outputs summary statistics to `stderr`:

```
cat test.fq | nanoq
```

Filtered by read length and mean read quality, reads are outout to `stdout`:

```
cat test.fq | nanoq -l 1000 -q 10 > filtered_reads.fq 
```

Advanced filtering analogous to `Filtlong` with a two-pass filter:

```
cat test.fq | nanoq --keep_percent 80 --target_bases 500000 > filtered_reads.fq 
```

# Summary statistics

| output field    |  specification `v0.2.0`                                           |
| -------------   | ----------------------------------|
| reads           | number of sequence reads          | 
| bases           | number of bases (bp)              | 
| n50             | n50 if read lenghts               | 
| longest         | longest read in read set          | 
| shortest        | shortest read in read set         | 
| mean length     | average length of read set        | 
| median length   | median length of read set         | 
| mean q          | mean nanopore quality score       | 
| median q        | median quality score              | 


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

Filter commands:



Summary statistic commands:



While the `rust-bio` parser is slightly faster in these benchmarks for these specific applications than `needletail`, the default mode for `nanoq` uses `needletail` due to its native capacity to parse gzipped and fasta type input formats.

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through Cargo (`cargo install nanoq`), as Docker (`docker pull esteinig/nanoq`) or Singularity container (`singularity pull docker://esteinig/nanoq`) or through BioConda (`conda install -c bioconda nanoq`).

# Acknowledgements

My backyard monitor lizard, Hanson, a cold-blooded reptilian scavenging for scraps.

# References

