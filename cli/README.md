# tesseract-rs-cli

Command-line OCR tool built on [tesseract-rs](https://crates.io/crates/tesseract-rs).

## Usage

```sh
tesseract-rs image.png
tesseract-rs -l eng+tur --psm 6 document.png
tesseract-rs -o hocr scanned.png > result.hocr
```

## Options

| Option | Description |
| ------ | ----------- |
| `-l, --lang <LANG>` | Language(s) to use (default: `eng`) |
| `-p, --psm <MODE>` | Page segmentation mode 0-13 (default: 3 = auto) |
| `-t, --tessdata <DIR>` | Tessdata directory (default: `TESSDATA_PREFIX`, then `tesseract --print-tessdata-dir`, then the compiled-in default) |
| `-o, --output <FMT>` | Output format: `txt`, `hocr`, `tsv` (default: `txt`) |
| `-v, --version` | Print version |
| `-h, --help` | Print help |

## Install

```sh
# Bundled Tesseract (compiled from source, no system dependency)
cargo install tesseract-rs-cli

# Or link against a system Tesseract (Homebrew)
brew install tesseract
cargo install tesseract-rs-cli --no-default-features --features use-system-tesseract
```

On macOS you can also use the Homebrew tap:

```sh
brew tap cafercangundogdu/tesseract-rs
brew install tesseract-rs
```
