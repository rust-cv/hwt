mod indices;

use criterion::*;

criterion_main! {
    indices::benches,
}
