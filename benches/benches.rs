mod indices;
mod insert;
mod neighbors;

use criterion::*;

criterion_main! {
    indices::benches,
    insert::benches,
    neighbors::benches,
}
