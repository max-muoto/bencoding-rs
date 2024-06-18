# bencoding-rs

Simple library for decoding bencoded data in Rust.

## Usage

```rust
use bencoding::decode;

fn main() {
    let data = b"i42e";
    let decoded = decode(data).unwrap();
    assert_eq!(decoded, 42);
}
```

## License

MIT
