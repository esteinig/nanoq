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

Nanopore sequencing is now routinely applied to a variety of genomic problems ranging from telomer-to-telomer chromosome sequencing to real-time, genome-informed infectious disease surveillance. as a consequence, sequence data in the public domain has increased rapidly in the last few years. One of the first steps during and after sequencing is to assess the quality of sequence reads and obtain basic summary statistics after basecalling the raw signal. A common practice for quality control and filtering of reads for length and quality is to use the basecalling summary file as index of reads to accelerate iteration over hundreds of thousands to millions of sequence reads. However, large volumes of basecalled reads are in the public domain where a sequencing summary is not available and with increasing throughput on nanopore devices, including larger devices like the PromethION, faster processing of sequence reads is desired. `NanoStat` and `NanoFilt` are classic toolkits to filter and obtain summary statistics from nanopore data, but tend to be slow to process read data if not presented with a summaru file. We therefore implement `nanoq` a fast but minimal summary and quality control tool for nanopore reads in `Rust`. `Nanoq` can be efficiently applied on sequence read data from public domain where basecalling summaries are unavailable,  as part of automated pipelines processing nay nanopore read data, or directly from the command line to check on statistics of currently active sequencing runs.

# Methodology

`Nanoq` is implemented in `Rust` using the fastq reader from `rust-bio`. `Nanoq` by default accepts a streadm of fastq reads

# Applications

Compute summary statistics:

```
nanoq -f test.fq
```

Filtered by read length and mean read quality:

```
nanoq -f test.fq -l 1000 -q 10 -o filt.fq 
```

Read stream:

```
cat test.fq | nanoq -l 1000 -q 10 > /dev/null
```

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


Benchmarks evaluate processing speed of a long-read filter and computation of summary statistics on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) (3,491,390  reads, 14.38 Gbp, `GridION`) using the `nanoq:v0.1.0` `Singularity` image in comparison to [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong)

Filter:

| program         |  command                                           |  real time |  reads / sec    | speedup |
| -------------   | ---------------------------------------------------|------------| ----------------|---------|
| nanofilt        | `NanoFilt -f test.fq -l 5000 > /dev/null`          | 00:20:39   | 2,818           | 1.00 x  |
| filtlong        | `filtlong --min_length 5000 test.fq > /dev/null`   | 00:13:20   | 4,364           | 1.55 x  |
| nanoq           | `nanoq -f test.fq -l 5000 > /dev/null`             | 00:02:44   | 21,289          | 7.55 x  |

Summary statistics:

| program         |  command                       | threads  | real time |  reads / sec    | speedup |
| -------------   | -------------------------------|----------|-----------| ----------------|---------|
| nanostat        | `NanoStat -f test.fq -t 1`     | 1        | 00:18:47  | 3,097           | 1.00 x  |
| nanostat        | `NanoStat -f test.fq -t 8`     | 8        | 00:18:29  | 3,148           | 1.01 x  |
| nanostat        | `NanoStat -f test.fq -t 16`    | 16       | 00:18:24  | 3,162           | 1.02 x  |
| nanoq           | `nanoq -f test.fq 2> stats.txt`| 1        | 00:02:44  | 21,289          | 6.87 x  |

Since we directed the reads to `/dev/null` in the filter benchmarks there is no difference to computing just the summary statistics for `nanoq`. Additional threads in `NanoStat` did not make a difference in processing the `fastq` which is likely limited by input capacity of the reader. 

# Availability

`Nanoq` is open-source on GitHub (https://github.com/esteinig/nanoq) and available through Cargo (`cargo install nanoq`), as Docker (`docker pull esteinig/nanoq`) or Singularity container (`singularity pull docker://esteinig/nanoq`) or through BioConda (`conda install -c bioconda nanoq`).

# Acknowledgements

My backyard monitor lizard, P. Hanson. Just like the namesake, just a cold brainless reptilian scavenging for scraps.

# References

