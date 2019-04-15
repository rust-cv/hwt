mod insert;
mod neighbors;

use criterion::*;

criterion_main! {
    insert::benches,
    neighbors::benches,
}
