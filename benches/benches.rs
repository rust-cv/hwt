mod indices;
mod insert;

use criterion::*;

criterion_main! {
    indices::benches,
    insert::benches,
}
