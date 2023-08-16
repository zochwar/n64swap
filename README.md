# n64swap

A slightly over-engineered Nintendo 64 Byte Swapper

## Usage
n64swap \<filename\> [outputfile]

The simplest usage is `n64swap file.v64`, this will convert your file to a Big-Endian (.z64) rom.\
You can optionally add the output filename as the second argument.

There are also some option flags available
* -r, --romtype <ROMTYPE>
    * big-endian (commonly .z64)
    * byte-swap  (commonly .v64)
    * little-endian (commonly .n64)
* -i, --identify
    * Identify rom (and exit)
* -f, --force
    * Force overwrite output file
* -h, --help
    * Print help (see a summary with '-h')
* -V, --version
    * Print version

## Dependencies
This program is written in [Rust](https://www.rust-lang.org/)\
[Clap](https://github.com/clap-rs/clap) is used to parse the commandline, cargo will add this automatically

## Building
There's nothing fancy going on, `cargo build --release` should work
