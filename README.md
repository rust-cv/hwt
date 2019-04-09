# hwt

Hamming Weight Tree from the paper Online Nearest Neighbor Search in Hamming Space

To understand how the data structure works, please see [the docs](https://docs.rs/hwt/).

## Benchmarks

You can find benchmark output [here](http://vadixidav.github.io/hwt/).

If you would like to run the benchmarks yourself, just run `cargo bench` at the
command line.

## Implemented

- Search within a given hamming radius.

## Not Implemented

- Searching at a specific hamming radius or in a range of radii.
