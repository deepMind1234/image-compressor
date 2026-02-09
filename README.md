# Image Compressor

A no-frills command-line tool to compress images to a target size (JPEG, PNG, WebP). I build this because I was tired of using online image compressors and all the photos from my Pixel would exceed the file requirement for every use case. I leveraged AI for this and chose Rust as renowned for it's performance and safety.

## Download

```bash
git clone https://github.com/yourusername/image-compressor.git
cd image-compressor
```

## Install

To install the binary globally (to `~/.cargo/bin`):

```bash
cargo install --path .
```

To build locally:

```bash
cargo build --release
# Binary location: target/release/compress
```

## Dependencies

You can check the dependencies in two ways:

1. **Direct dependencies**: View the `[dependencies]` section in [Cargo.toml](./Cargo.toml).
2. **Full dependency tree**:
   ```bash
   cargo tree
   ```

## Usage

```bash
compress <INPUT_PATH> <OUTPUT_PATH> --ms <MAX_SIZE>
```

**Example:**

```bash
compress photo.jpg optimized.jpg --ms 500KB
compress large.png small.webp --ms 1MB
```

## Maintenance

To remove build artifacts and temporary files:

```bash
cargo clean
```
