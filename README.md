# nanoq <a href='https://github.com/esteinig'><img src='docs/logo.png' align="right" height="210" /></a>

![](https://img.shields.io/badge/lang-rust-black.svg)
![](https://img.shields.io/badge/version-0.0.1-purple.svg)

Minimal quality control for nanopore reads in `Rust`.

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
- [Citing](#citing)

## Motivation

Basic read filters and summary statistics can be a bit slow when a `sequencing_summary` file is not available. `Nanoq` attempts to perform basic quality control a little faster.
