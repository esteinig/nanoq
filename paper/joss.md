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
   orcid: 0000-0001-8735-9144
   affiliation: 1
affiliations:
 - name: Australian Institute of Tropical Health and Medicine, Townsville, Queensland, Australia
   index: 1
date: 11 July 2020
bibliography: paper.bib
---

# Summary

Nanopore sequencing is now routinely applied to a variety of genomic problems ranging from telomer-to-telomer chromosome sequencing to real-time, genome-informed infectious disease surveillance. as a consequence, sequence data in the public domain has increased rapidly in the last few years. One of the first steps during and after sequencing is to assess the quality of sequence reads and obtain basic summary statistics after basecalling the raw signal. A common practice for quality control and filtering of reads for length and quality is to use the basecalling summary file as index of reads to accelerate iteration over hundreds of thousands to millions of sequence reads. However, large volumes of basecalled reads are in the public domain where a sequencing summary is not available and with increasing throughput on nanopore devices, including larger devices like the PromethION, faster processing of sequence reads is desired. We implement a fast and minimal summary and quality control tool for fastq reads from nanopore devices in `Rust`: `Nanoq` can be efficiently applied on sequence read data from public domain where basecalling summaries are unavailable, and as part of automated pipelines processign nay nanopore read data.

# Methodology

`Nanoq` is implemented in `Rust` using the fastq reader from `rust-bio`. `Nanoq` by default accepts a streadm of fastq reads

# Availability


# Acknowledgments


# References
