# hwt

Hamming Weight Tree from the paper Online Nearest Neighbor Search in Hamming Space

To understand how the data structure works, please see [the docs](https://docs.rs/hwt/).

## Benchmarks

Most recent benchmark for 1-NN:

![1-NN Benchmark](http://vadixidav.github.io/hwt/983f0dcf0b7cc8f237d5771817414df2a22e1239/neighbors/report/lines.svg)

You can find benchmark output [here](http://vadixidav.github.io/hwt/).

If you would like to run the benchmarks yourself, just run `cargo bench` at the command line. I recommend using `RUSTFLAGS='-C target-cpu=native' cargo bench` instead since both linear search and this tree are both significantly faster when using modern instructions.
