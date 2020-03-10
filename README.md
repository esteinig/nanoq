# nanoq <a href='https://github.com/esteinig'><img src='docs/logo.png' align="right" height="210" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.1.0-purple.svg)

Minimal but speedy quality control for nanopore reads.

## Overview

**`v0.1.0: it's working`**

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
- [Benchmarks](#benchmarks)
- [Etymology](#etymology)
- [Citing](#citing)

## Motivation

Basic read filters and computation of summary statistics can be a bit slow when a `sequencing_summary` file is not available. `Nanoq` attempts to perform these operations on `fastq` files faster.

Quality scores are computed for basecalls from nanopore sequencing data as outlined in the [technical documentation](https://community.nanoporetech.com/technical_documents/data-analysis/) and [this issue](https://github.com/esteinig/nanoq/issues/2).

## Usage

### Command line

Summary statistics:

```
nanoq -f test.fq
```

File mode:

```
nanoq -f test.fq -l 1000 -q 10 -o filt.fq 
```

Streaming mode:

```
cat test.fq | nanoq -l 1000 -q 10 > /dev/null
```

### Parameters

```
nanoq 0.0.1

Minimal quality control for nanopore reads

USAGE:
    nanoq [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --fastq <FILE>     Input fastq file [-]    
    -o, --output <FILE>    Output fastq file [-]
    -l, --length <INT>     Minimum read length [0]
    -q, --quality <INT>    Minimum read quality [0]
```

### Output

`Nanoq` writes reads passing filters to `/dev/stdout` or a `fastq` file. When filters are switched off (default) only summary statistics are computed and written to `/dev/stderr`:

```
5000 29082396 62483 120 5816 2898 11.87 12.02
```

These correspond to:

```
reads bp longest shortest mean_length median_length mean_qscore median_qscore
```

## Etymology

Coincidentally `nanoq` [nanɔq] means 'polar bear' in Native American (Eskimo-Aleut, Greenlandic). If you find `nanoq` useful for your research consider a small donation to the [Polar Bear Fund](https://www.polarbearfund.ca/) or [Polar Bears International](https://polarbearsinternational.org/).

## Citing

Nothing yet.
