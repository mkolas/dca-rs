# dca-rs

Rust implementation of the [DCA1](https://github.com/bwmarrin/dca/wiki/DCA1-specification-draft) format.

For more information, see the reference implementation: https://github.com/bwmarrin/dca

The goal of this project is to match the reference implementation as closely as possible.

## Differences

Although dca-rs tries to be a seamless drop-in replacement of the reference implementation, there are some differences, as explained below.

### Command line flags

In order to follow the conventions for command line arguments and flags, dca-rs does not allow multi-character flags to be used with a single dash, rather, they must be used with two dashes.

For example, `-raw` becomes `--raw`, `-aa` becomes `--aa`, `-vol` becomes `--vol` and so on.

Note: `-i` is an exception to this, it can be used with both a single dash (because it is only one letter) and two dashes, to avoid ambiguity.

For more usage information, run dca-rs with the `--help` flag.

## Installation

Download the latest binaries for your platform [from the releases section](https://github.com/nstafie/dca-rs/releases)!

## Usage

Run `./dca-rs --help` for usage information.
