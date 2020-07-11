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

# Availability


# Acknowledgments


# References
