# array-plus-extra

[![Crates.io](https://img.shields.io/crates/v/array-plus-extra.svg)](https://crates.io/crates/array-plus-extra)
[![Documentation](https://docs.rs/array-plus-extra/badge.svg)](https://docs.rs/array-plus-extra)
[![License](https://img.shields.io/crates/l/array-plus-extra.svg)](https://github.com/korken89/array-plus-extra#license)

An array type that holds N+EXTRA elements using const generic parameters, providing safe slice access to contiguous memory.

This allows the creation of arrays that would require more powerful const-generics, e.g. `[T; N+3]`.

## Features

- Specify both base size (N) and extra elements (EXTRA) at compile time
- Deref to `&{mut} [T]` provides safe access to all N+EXTRA elements
- `as_slice()` and `as_mut_slice()` work in const contexts
- No runtime overhead compared to raw arrays
- Works in embedded and bare-metal environments
- All unsafe code verified with **Miri** for undefined behavior
- Optional `serde` support for serialization/deserialization

## Examples

### Basic usage

```rust
use array_plus_extra::ArrayPlusExtra;

// Create an array with 5 base elements + 3 extra = 8 total elements.
let arr: ArrayPlusExtra<i32, 5, 3> = ArrayPlusExtra::new(42);

// Access via deref to slice.
assert_eq!(arr.len(), 8);
assert_eq!(arr[0], 42);
assert_eq!(arr[7], 42);

// Use slice methods.
let sum: i32 = arr.iter().sum();
assert_eq!(sum, 336); // 42 * 8
```

### Mutable access

```rust
use array_plus_extra::ArrayPlusExtra;

let mut arr: ArrayPlusExtra<i32, 2, 2> = ArrayPlusExtra::new(0);

// Modify through deref_mut.
arr[0] = 10;
arr[1] = 20;
arr[2] = 30;
arr[3] = 40;

assert_eq!(arr[0], 10);
assert_eq!(arr[3], 40);
```

### Const contexts

```rust
use array_plus_extra::ArrayPlusExtra;

const ARR: ArrayPlusExtra<u8, 3, 2> = ArrayPlusExtra::new(255);
const SLICE: &[u8] = ARR.as_slice();
const LEN: usize = SLICE.len();

assert_eq!(LEN, 5);
assert_eq!(SLICE[0], 255);
```

## Testing

Run tests:
```bash
cargo test
```

Run tests with Miri (requires nightly):
```bash
cargo +nightly miri test
```

## Minimum Supported Rust Version (MSRV)

This crate requires Rust 1.85 or later due to the use of Edition 2024 and const fn features.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
