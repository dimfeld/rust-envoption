# EnvOption

[![crates.io](https://img.shields.io/crates/v/envoption.svg)](https://crates.io/crates/envoption)

Simple functions for parsing environment variables when reading config.

* `require` - Parse an environment variable or return an error if it doesn't exist.
* `optional` - Same as above, but return an `Option<T>` set to `None` if the variable doesn't exist.
* `with_default` - As above, but return a default value if the variable doesn't exist.

[Documentation](https://docs.rs/envoption/)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
