>## Archive Note
>The `pessimistic-proof` library has been moved into https://github.com/agglayer/agglayer.
>Please refer to this new location for inquiries or contributions.

# Pessimistic Proof

Current guidance from Succinct for running in performance-optimized mode:

``RUST_LOG=info RUSTFLAGS='-C target-cpu=native -C target_feature=+avx512ifma,+avx512vl --cfg curve25519_dalek_backend="simd"' cargo run --release``

Note that this requires compiling and running on a avx512 enabled CPU.

## License
Copyright (c) 2024 PT Services DMCC

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 

The SPDX license identifier for this project is `MIT OR Apache-2.0`.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
