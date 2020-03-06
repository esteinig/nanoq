# nanoq <a href='https://github.com/esteinig'><img src='docs/logo.png' align="right" height="210" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.0.1-purple.svg)

Speedy but minimal quality control for `fastq` nanopore reads.

## Overview

**`v0.0.1: basically nothing`**


- [Motivation](#motivation)
- [Install](#install)
  - [:new_moon: `singularity`](#singularity)
  - [:rocket: `cargo`](#cargo)
  - [:whale: `docker`](#docker)
  - [:snake: `conda`](#conda)
- [Usage](#usage)
  - [Command line](#command-line)
- [Citing](#citing)

## Motivation

I often need to filter nanopore read files by minimum length and average quality but current tools are slowish without a `sequencing_summary`.
