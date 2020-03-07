# nanoq <a href='https://github.com/esteinig'><img src='docs/logo.png' align="right" height="210" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.0.1-purple.svg)

Minimal quality control for nanopore reads in `Rust`

## Overview

**`v0.0.1: it's something`**

- [Motivation](#motivation)
- [Install](#install)
  - [:new_moon: `singularity`](#singularity)
  - [:rocket: `cargo`](#cargo)
  - [:whale: `docker`](#docker)
  - [:snake: `conda`](#conda)
- [Usage](#usage)
  - [Command line](#command-line)
  - [Parameters](#parameters)
  - [Output](#output)
- [Etymology](#etymology)
- [Citing](#citing)

## Motivation

Basic read filters and computation of summary statistics can be a bit slow when a `sequencing_summary` file is not available. `Nanoq` attempts to perform these operations on `fastq` files a little faster.

## Usage

```
nanoq --fastq test.fq --length 1000 --quality 10 --output filt.fq 
```

`Nanoq` outputs a single row of whitespace delimited numerics for the summary statistics of the `fastq` file to `stderr`:

```
29082396 5000 62483 120 5816 2898 11.87 12.02
```

These correspond to:

```
bp_total num_reads longest_reads shortest_read mean_length median_length mean_q median_q
```

Please note that `nanoq` is not a general `fastq` quality control tool because the quality scores are computed for basecalls from nanopore sequencing data as outlined in the [technical documentation](https://community.nanoporetech.com/technical_documents/data-analysis/) and [this issue](https://github.com/esteinig/nanoq/issues/2).

## Etymology

Since all the 'qc' variants of nanopore-themed names seemed to be taken the 'c' was rather lazily dropped. Coincidentally `nanoq` [nanɔq] also means 'polar bear' in Native American (Eskimo-Aleut, Greenlandic). If you find `nanoq` useful for your research consider a small donation to the [Polar Bear Fund](https://www.polarbearfund.ca/) or [Polar Bears International](https://polarbearsinternational.org/).

## Citing

Nothing yet.
