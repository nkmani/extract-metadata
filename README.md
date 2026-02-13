# Extract Metadata

A Rust command-line tool for extracting metadata from Adobe Flash SWF files. Supports both individual files and recursive directory traversal, with output in JSON, YAML, or plain text formats.

## Features

- 📁 **Dual Mode Operation**: Process individual SWF files or entire directories
- 🔄 **Recursive Traversal**: Automatically finds all SWF files in subdirectories
- 📊 **Multiple Output Formats**: JSON, YAML, or human-readable text
- 💾 **Auto-Save**: Metadata files are saved alongside each SWF file
- 🔍 **Case-Insensitive**: Finds `.swf`, `.SWF`, `.Swf`, etc.
- 🛡️ **Error Handling**: Continues processing even if individual files fail
- 📢 **Verbose Mode**: Detailed progress reporting

## Extracted Metadata

For each SWF file, the tool extracts:

- **File Name**: Full path to the SWF file
- **Stage Size**: Width and height in pixels (e.g., `[550, 400]`)
- **Number of Frames**: Total frame count in the animation
- **Frame Rate**: Frames per second

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build from Source

```bash
git clone https://github.com/nkmani/extract-metadata.git
cd extract-metadata
cargo build --release
```

The compiled binary will be available at `target/release/extract-metadata`.

## Usage

### Basic Syntax

```bash
extract-metadata -i <PATH> [OPTIONS]
```

### Options

- `-i, --input <PATH>`: Path to SWF file or directory (required)
- `-f, --format <FORMAT>`: Output format: `json`, `yaml`, or `text` (default: `json`)
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Examples

#### Process a Single File

```bash
# Extract metadata in JSON format (default)
cargo run -- -i ~/animation/movie.swf

# Extract metadata in YAML format
cargo run -- -i ~/animation/movie.swf -f yaml

# Extract metadata in text format with verbose output
cargo run -- -i ~/animation/movie.swf -f text -v
```

**Output file**: `movie.swf.json` (or `.yaml`/`.text` depending on format)

#### Process a Directory

```bash
# Process all SWF files in a directory and subdirectories
cargo run -- -i ~/animations -v

# Process with YAML output
cargo run -- -i ~/animations -f yaml
```

The tool will:
1. Recursively search for all `.swf` files
2. Extract metadata from each file
3. Save metadata files alongside each SWF file

### Output Examples

#### JSON Format
```json
{
  "file_name": "~/animations/3-1/3-1.swf",
  "stage_size": [550, 400],
  "no_of_frames": 321,
  "frame_rate": 24
}
```

#### YAML Format
```yaml
file_name: ~/animations/3-1/3-1.swf
stage_size:
- 550
- 400
no_of_frames: 321
frame_rate: 24
```

#### Text Format
```
File: ~/animations/3-1/3-1.swf
Stage Size: (550, 400)
Number of Frames: 321
Frame Rate: 24
```

## Development

### Project Structure

```
extract-metadata/
├── Cargo.toml          # Project dependencies and metadata
├── src/
│   └── main.rs         # Main application code
├── .gitignore          # Git ignore rules
└── README.md           # This file
```

### Dependencies

- **clap**: Command-line argument parsing
- **swf**: SWF file parsing and decompression
- **serde**: Serialization framework
- **serde_json**: JSON serialization
- **serde_yaml**: YAML serialization
- **walkdir**: Recursive directory traversal

### Running Tests

The project includes comprehensive unit tests covering:
- Metadata serialization (JSON, YAML, text formats)
- Output file path generation
- SWF file extension detection (case-insensitive)
- Metadata equality comparisons

Run all tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

Run a specific test:
```bash
cargo test test_metadata_to_json
```

**Test Coverage**: 13 unit tests ensuring code quality and reliability.

### Building for Release

```bash
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the MIT License.

## Acknowledgments

- Built with the [swf](https://crates.io/crates/swf) crate for SWF parsing
- Uses [clap](https://crates.io/crates/clap) for elegant command-line interfaces
