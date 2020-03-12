# nanoq <a href='https://github.com/esteinig'><img src='docs/logo.png' align="right" height="210" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.1.0-purple.svg)

Minimal but speedy quality control for nanopore reads.

## Overview

**`v0.1.0: it's working`**

- [Motivation](#motivation)
- [Install](#install)
  - [:rocket: `cargo`](#cargo)
  - [:new_moon: `singularity`](#singularity)
  - [:snake: `conda`](#conda)
  - [:whale: `docker`](#docker)
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

## Install

#### `Cargo`

If you have [`Rust`](https://www.rust-lang.org/tools/install) and `Cargo` installed:

```
cargo install nanoq
nanoq --help
```

#### `Singularity`

I prefer `Singularity` over `Docker` containers for integrated access to the host file system.

```
singularity pull docker://esteinig/nanoq
./nanoq_latest.sif --help
```

#### `Conda`

Currently on my channel but will be in `BioConda` soon:

```
conda install -c esteinig nanoq
nanoq --help
```

#### `Docker`

`Docker` containers need a user- and bindmount of the current host working directory containing the `fastq` (here: `test.fq`) - which links into the default container working directory `/data`:

```
docker run -it \
  -v $(pwd):/data \
  -u $(id -u):$(id -g) \
  esteinig/nanoq \
  --fastq /data/test.fq \
  --output /data/filt.fq
```

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
nanoq 0.1.0

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

`Nanoq` outputs  reads to `/dev/stdout` or a `fastq` file - except when filters are switched off (default) so that only the summary statistics are computed. `Nanoq` outputs a single row of summary statistics on the filtered read set to `/dev/stderr`:

```
5000 29082396 62483 120 5816 2898 11.87 12.02
```

These correspond to:

```
reads bp longest shortest mean_length median_length mean_qscore median_qscore
```

## Benchmarks

Benchmarking evaluates a simple long-read filter on the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) (3,491,390  reads, 14.38 Gbp, `GridION`) using `Singularity` image `nanoq:v0.1.0` and compares it to [`NanoFilt`](https://github.com/wdecoster/nanofilt) via `PyPI` and [`Filtlong`](https://github.com/rrwick/Filtlong) via `BioConda`.

| program         |  command                                           |  real time |  reads / sec    |
| -------------   | ---------------------------------------------------|------------| ----------------|
| nanofilt        | `NanoFilt -f test.fq -l 5000 > /dev/null`          | 00:20:39   | 2,818           |
| filtlong        | `filtlong --min_length 5000 test.fq > /dev/null`   | 00:13:20   | 4,364           |
| nanoq           | `nanoq -f test.fq -l 5000 > /dev/null`             | 00:02:54   | 20,065          |

## Etymology

Coincidentally `nanoq` [nanɔq] means 'polar bear' in Native American (Eskimo-Aleut, Greenlandic). If you find `nanoq` useful for your research consider a small donation to the [Polar Bear Fund](https://www.polarbearfund.ca/) or [Polar Bears International](https://polarbearsinternational.org/).

## Citing

Nothing yet.
