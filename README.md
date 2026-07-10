# io_unity_next

`io_unity_next` is a Rust library for reading Unity assets, including UnityFS bundles and serialized files.

[![Crates.io](https://img.shields.io/crates/v/io_unity_next)](https://crates.io/crates/io_unity_next)

## Type tree data

The `type-tree-json` feature supports reading assets without embedded type trees by loading a zstd-compressed tar archive of JSON type tree data. Compatible data can be generated from [TypeTreeDumper](https://github.com/DaZombieKiller/TypeTreeDumper) or [TypeTreeDumps](https://github.com/AssetRipper/TypeTreeDumps).

Create an archive from the `InfoJson` directory with either command:

```sh
tar -caf InfoJson.tar.zst InfoJson
```

```sh
tar -c InfoJson | zstd --ultra -22 -o InfoJson.tar.zst
```

The archive should contain paths such as `InfoJson/x.x.x.json`.

## Examples

Build and run the Live2D extractor with all optional formats enabled:

```sh
cargo build --example live2dextractor --release --all-features
cargo run --example live2dextractor --release --all-features -- help
```
