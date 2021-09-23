# nanoq <a href='https://github.com/esteinig'><img src='docs/nanoq.png' align="right" height="270" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.7.0-purple.svg)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.3707754.svg)](https://doi.org/10.5281/zenodo.3707754)

Minimal but speedy quality control for nanopore reads.

## Overview

**`v0.7.0`**

- [Purpose](#purpose)
- [Install](#install)
- [Usage](#usage)
  - [Command line](#command-line)
  - [Parameters](#parameters)
  - [Output](#output)
- [Benchmarks](#benchmarks)
- [Dependencies](#dependencies)
- [Etymology](#etymology)
- [Citing](#citing)

## Purpose

`Nanoq` implements fast sequence read and quality filtering and produces simple summary reports. Quality scores are computed for basecalls from nanopore sequencing data, as outlined in the [technical documentation](https://community.nanoporetech.com/technical_documents/data-analysis/) or in this [blog post](https://gigabaseorgigabyte.wordpress.com/2017/06/26/averaging-basecall-quality-scores-the-right-way/).

## Install

#### `Cargo`

If you have [`Rust`](https://www.rust-lang.org/tools/install) and `Cargo` installed:

```
cargo install nanoq
```

#### `Conda`

Currently on this channel but will be in `BioConda`:

```
conda install -c conda-forge -c esteinig nanoq=0.7.0
```

#### `Docker`

`Docker` image is based on `Alpine OS` (~ 20 MB)

```
docker pull esteinig/nanoq:latest
```

## Usage

`Nanoq` accepts a file (`--input`) or stream (`stdin`) of reads in `fast{a,q}.{gz,bz2,xz}` format and outputs reads to file (`--output`) or stream (`stdout`).

```bash
nanoq -i test1.fq.gz -o test2.fq
cat test1.fq.gz | nanoq > test2.fq
```

Output compression is inferred from file extensions (`gz`, `bz2`, `lzma`).

```bash
nanoq -i test1.fq -o test2.fq.gz
```

Output compression can be specified manually with `--output-type` and `--compress-level`.

```bash
nanoq -i test1.fq -O g -c 9 -o test2.fq.cmp
```

Reads can be filtered by minimum read length (`--min-len`), maximum length (`--max-len`) or mean read quality (`--min-qual`).

```bash
nanoq -i test.fq -l 1000 -q 10 -m 10000 > reads.fq 
```

Read summaries without output can be obtained by directing to `/dev/null` or using the stats flag (`--stats`):

```bash
nanoq -i test.fq > /dev/null
nanoq -i test.fq -s
```

Read qualities may be excluded from filters and statistics to speed up read iteration in some cases (`--fast`).

```bash
nanoq -i test1.fq.gz -f -s
```

`Nanoq` can be used to check on active sequencing runs and barcoded samples.

```bash
find /data/nanopore/run -name *.fastq -print0 | xargs -0 cat | nanoq -s
```

```bash
for i in {01..12}; do
  find /data/nanopore/run -name barcode${i}.fastq -print0 | xargs -0 cat | nanoq -s
done
```

### Parameters

```
nanoq 0.7.0

Read filters and summary reports for nanopore data

USAGE:
    nanoq [FLAGS] [OPTIONS]

FLAGS:
    -f, --fast       Fast mode, do not consider quality values
    -h, --help       Prints help information
    -s, --stats      Statistics only, reads to /dev/null
    -V, --version    Prints version information
    -v, --verbose    Pretty print output statistics

OPTIONS:
    -c, --compress-level <1-9>     Compression level to use if compressing output [default: 6]
    -i, --input <input>            Fast{a,q}.{gz,xz,bz}, stdin if not present
    -m, --max-len <INT>            Maximum read length filter (bp) [default: 0]
    -l, --min-len <INT>            Minimum read length filter (bp) [default: 0]
    -m, --min-qual <FLOAT>         Minimum average read quality filter (Q) [default: 0]
    -o, --output <output>          Output filepath, stdout if not present
    -O, --output-type <u|b|g|l>    u: uncompressed; b: Bzip2; g: Gzip; l: Lzma
    -t, --top <INT>                Number of top reads in verbose summary [default: 5]
```

### Output

A basic read summary is output to `stderr`: 

```bash
100000 400398234 5154 44888 5 4003 3256 8.90 9.49
```

* number of reads
* number of base pairs
* read length N50
* longest and shorted reads
* mean and median read length
* mean and median read quality 

Extended summaries analogous to `NanoStat` can be obtained using multiple `--verbose` flags:

```bash
nanoq -i test.fq -f -s -vv
```

```
Nanoq Read Summary
====================

Number of reads:      100000
Number of bases:      400398234
N50 read length:      5154
Longest read:         44888 
Shortest read:        5
Mean read length:     4003
Median read length:   3256 
Mean read quality:    NaN 
Median read quality:  NaN


Read length thresholds (bp)

> 200       99104             99.1%
> 500       96406             96.4%
> 1000      90837             90.8%
> 2000      73579             73.6%
> 5000      25515             25.5%
> 10000     4987              05.0%
> 30000     47                00.0%
> 50000     0                 00.0%
> 100000    0                 00.0%
> 1000000   0                 00.0%


Top ranking read lengths (bp)

1. 44888       
2. 40044       
3. 37441       
4. 36543       
5. 35630
```

## Benchmarks

TBD


## Dependencies

`Nanoq` uses the [`needletail`](https://github.com/onecodex/needletail) library from `OneCodex`. 

## Etymology

Avoided name collision with `nanoqc` and dropped the `c` to arrive at `nanoq` [nanɔq] which coincidentally means 'polar bear' in Native American ([Eskimo-Aleut](https://en.wikipedia.org/wiki/Eskimo%E2%80%93Aleut_languages), Greenlandic). If you find `nanoq` useful for your research consider a small donation to the [Polar Bear Fund](https://www.polarbearfund.ca/) or [Polar Bears International](https://polarbearsinternational.org/)

## Contributions

We welcome any and all suggestions or pull requests. Please feel free to open an issue in the repository on `GitHub`.