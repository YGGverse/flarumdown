# flarumdown

![Linux](https://github.com/YGGverse/flarumdown/actions/workflows/linux.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/flarumdown/status.svg)](https://deps.rs/repo/github/YGGverse/flarumdown)
[![crates.io](https://img.shields.io/crates/v/flarumdown.svg)](https://crates.io/crates/flarumdown)

Flarum is down - read in Markdown

CLI tool for [Flarum](https://flarum.org) v2 that allows to export public DB entries into Markdown format.

> [!IMPORTANT]
> This extension was created by and for the UANA community forums. This means that it's adapted for our local build first:
> * SQLite driver is in use
> * FoF/upload plugin installed
> * Markdown (only) plugin is enabled
> * Flarum v2 (7 beta)

## Install

``` bash
cargo install flarumdown
```

## Usage

``` bash
RUST_LOG=warn flarumdown -s '/path/to/flarum.sqlite' \
                         -t '/path/to/target' \
                         -p '/path/to/public' \
                         -i index \
                         -r http://[202:68d0:f0d5:b88d:1d1a:555e:2f6b:3148] \
                         -r http://[505:6847:c778:61a1:5c6d:e802:d291:8191] \
                         -r http://hc3fycfadz7fkapp62fqi6llioe46fvis6wuswfobl5ghc2u7snq.b32.i2p \
                         -r http://w6vtcpbir5vvokwdqqbqlrdtnzwyfc4iyqn6owxuyjeppszuydutqwqd.onion
```