# nanoq <a href='https://github.com/esteinig'><img src='docs/nanoq.png' align="right" height="270" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.1.1-purple.svg)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.3707754.svg)](https://doi.org/10.5281/zenodo.3707754)

Minimal but speedy quality control for nanopore reads.

## Overview

**`v0.1.1 no tests`**

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
- [Dependencies](#dependencies)
- [Etymology](#etymology)
- [Citing](#citing)

## Motivation

Basic sequence quality control and computation of summary statistics can be a bit slow due to bottlenecks in read parsing. `Nanoq` attempts to perform these operations on `fastx` files using the `needletail` and `rust-bio` libraries with either a single-pass operation for defaulty summary statistics and filtering, or a two-pass operation enabling advanced filtering methods similar to `Filtlong`.

Quality scores are computed for basecalls from nanopore sequencing data, as outlined in the [technical documentation](https://community.nanoporetech.com/technical_documents/data-analysis/).

## Install

#### `Cargo`

If you have [`Rust`](https://www.rust-lang.org/tools/install) and `Cargo` installed:

```
cargo install nanoq
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

```

#### `Singularity`

I prefer `Singularity` over `Docker` containers for integrated access to the host file system.

```
singularity pull docker://esteinig/nanoq
```


## Usage

### Command line

Summary statistics:

```
cat test.fq | nanoq
```

File mode:

```
nanoq -f test.fq -l 1000 -q 10 -o filt.fq 
```



### Parameters

```
nanoq 0.2.0

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

`Nanoq` outputs  reads to `/dev/stdout` or a `fastq` file. If filters are switched off (default) only the summary statistics are computed. `Nanoq` outputs a single row of summary statistics on the filtered read set to `/dev/stderr`:

```
5000 29082396 62483 120 5816 2898 11.87 12.02
```

These correspond to:

```
reads bp longest shortest mean_length median_length mean_qscore median_qscore
```

Extended output is enabled with up to 3 `--detail` (`-d`) flags:

```
nanoq -f test.fq -d -d
```

## Benchmarks

Benchmarks evaluate processing speed of a long-read filter and computation of summary statistics on the first 100,000 reads (`test.fq.gz` in Docker container) of the even [Zymo mock community](https://github.com/LomanLab/mockcommunity) (`GridION`) using the `nanoq:v0.2.0` [`Benchmark`](paper/Benchmarks) image with comparison to [`NanoFilt`](https://github.com/wdecoster/nanofilt), [`NanoStat`](https://github.com/wdecoster/nanostat) and [`Filtlong`](https://github.com/rrwick/Filtlong)

![nanoq benchmarks](paper/benchmarks.png?raw=true "Nanoq benchmarks")

Filter:

| program         |  example command                                   | mean time (+/- sd)  |  ~ reads / sec  | speedup |
| -------------   | ---------------------------------------------------|---------------------| ----------------|---------|
| nanofilt        | `cat test.fq | NanoFilt -l 5000 > /dev/null`       | 00:20:39            | 2,818           | 1.00 x  |
| filtlong        | `filtlong --min_length 5000 test.fq > /dev/null`   | 00:13:20            | 4,364           | 1.55 x  |
| nanoq           | `cat test.fq | nanoq -l 5000 > /dev/null`          | 00:02:44            | 21,289          | 7.55 x  |

Summary statistics:

| program         |  example command                | threads  | mean time (+/- sd) |  reads / sec    | speedup |
| -------------   | --------------------------------|----------|--------------------| ----------------|---------|
| nanostat        | `NanoStat --fastq test.fq -t 4` | 4        | 00:18:47           | 3,097           | 1.00 x  |
| nanoq           | `cat test.fq | nanoq`           | 1        | 00:02:44           | 21,289          | 6.87 x  |


## Dependencies

`Nanoq` uses [`rust-bio`](https://rust-bio.github.io/) which has a ton of great contributors and the [`needletail`](https://github.com/onecodex/needletail) library from OneCodex. 

## Etymology

Avoiding name collision with `nanoqc` and dropping the `c` to arrive at `nanoq` [nanɔq] which coincidentally means 'polar bear' in Native American (Eskimo-Aleut, Greenlandic). If you find `nanoq` useful for your research consider a small donation to the [Polar Bear Fund](https://www.polarbearfund.ca/) or [Polar Bears International](https://polarbearsinternational.org/) :bear:

## Contributions

We welcome any and all suggestions or pull requests. Please feel free to open an issue in the repositorty on `GitHub`.