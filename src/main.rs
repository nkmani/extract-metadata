use clap::Parser;
use std::path::PathBuf;

use std::fs::{File, metadata};
use std::io::{BufReader, Write};
use walkdir::WalkDir;

/// Extract metadata from SWF files
#[derive(Parser, Debug)]
#[command(name = "extract-metadata")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SWF file or directory to extract metadata from
    #[arg(short, long, value_name = "PATH")]
    input: PathBuf,

    /// Output format (json, yaml, or text)
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(serde::Serialize, Debug, PartialEq)]
struct Metadata {
    file_name: String,
    stage_size: (u32, u32),
    no_of_frames: u32,
    frame_rate: u32,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        eprintln!("Input path: {:?}", args.input);
        eprintln!("Format: {}", args.format);
    }

    // Check if input is a file or directory
    let meta = metadata(&args.input).expect("Failed to read input path");

    if meta.is_file() {
        // Process single file
        if args.verbose {
            eprintln!("Processing single file...");
        }
        let output = extract_metadata(args.input.to_str().unwrap(), &args.format);
        let output_path = args.input.with_extension(format!("swf.{}", args.format));
        save_metadata(&output_path, &output, args.verbose);
    } else if meta.is_dir() {
        // Process directory
        if args.verbose {
            eprintln!("Processing directory recursively...");
        }
        process_directory(&args.input, &args.format, args.verbose);
    } else {
        eprintln!("Error: Input path is neither a file nor a directory");
        std::process::exit(1);
    }
}

fn extract_metadata(file_name: &str, format: &str) -> String {
    println!("Extracting metadata from: {:?}", file_name);
    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    let swf_buf = swf::decompress_swf(reader).unwrap();
    let swf = swf::parse_swf(&swf_buf).unwrap();

    let stage_size = swf.header.stage_size();

    let metadata = Metadata {
        file_name: file_name.to_string(),
        stage_size: (
            (stage_size.x_max - stage_size.x_min).to_pixels() as u32,
            (stage_size.y_max - stage_size.y_min).to_pixels() as u32,
        ),
        no_of_frames: swf.header.num_frames() as u32,
        frame_rate: swf.header.frame_rate().to_f32() as u32,
    };

    format_metadata(&metadata, format)
}

fn format_metadata(metadata: &Metadata, format: &str) -> String {
    if format == "json" {
        serde_json::to_string(metadata).unwrap()
    } else if format == "yaml" {
        serde_yaml::to_string(metadata).unwrap()
    } else {
        format!(
            "File: {}\nStage Size: {:?}\nNumber of Frames: {}\nFrame Rate: {}",
            metadata.file_name, metadata.stage_size, metadata.no_of_frames, metadata.frame_rate
        )
    }
}

fn process_directory(dir_path: &PathBuf, format: &str, verbose: bool) {
    let mut swf_count = 0;

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Check if file has .swf extension (case-insensitive)
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str().unwrap_or("").to_lowercase() == "swf" {
                    swf_count += 1;

                    if verbose {
                        eprintln!("Found SWF file: {:?}", path);
                    }

                    // Extract metadata
                    match std::panic::catch_unwind(|| {
                        extract_metadata(path.to_str().unwrap(), format)
                    }) {
                        Ok(output) => {
                            // Generate output filename
                            let output_path = path.with_extension(format!("swf.{}", format));
                            save_metadata(&output_path, &output, verbose);
                        }
                        Err(_) => {
                            eprintln!("Error: Failed to extract metadata from {:?}", path);
                        }
                    }
                }
            }
        }
    }

    if verbose {
        eprintln!("Processed {} SWF file(s)", swf_count);
    }
}

fn save_metadata(output_path: &PathBuf, content: &str, verbose: bool) {
    match File::create(output_path) {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => {
                if verbose {
                    eprintln!("Saved metadata to: {:?}", output_path);
                }
            }
            Err(e) => {
                eprintln!("Error writing to {:?}: {}", output_path, e);
            }
        },
        Err(e) => {
            eprintln!("Error creating file {:?}: {}", output_path, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test metadata
    fn create_test_metadata() -> Metadata {
        Metadata {
            file_name: "test.swf".to_string(),
            stage_size: (800, 600),
            no_of_frames: 100,
            frame_rate: 30,
        }
    }

    #[test]
    fn test_metadata_to_json() {
        let metadata = create_test_metadata();
        let result = format_metadata(&metadata, "json");

        // Verify it's valid JSON
        assert!(result.contains("\"file_name\":\"test.swf\""));
        assert!(result.contains("\"stage_size\":[800,600]"));
        assert!(result.contains("\"no_of_frames\":100"));
        assert!(result.contains("\"frame_rate\":30"));

        // Verify it can be parsed back
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["file_name"], "test.swf");
        assert_eq!(parsed["stage_size"][0], 800);
        assert_eq!(parsed["stage_size"][1], 600);
    }

    #[test]
    fn test_metadata_to_yaml() {
        let metadata = create_test_metadata();
        let result = format_metadata(&metadata, "yaml");

        // Verify YAML format
        assert!(result.contains("file_name: test.swf"));
        assert!(result.contains("stage_size:"));
        assert!(result.contains("- 800"));
        assert!(result.contains("- 600"));
        assert!(result.contains("no_of_frames: 100"));
        assert!(result.contains("frame_rate: 30"));
    }

    #[test]
    fn test_metadata_to_text() {
        let metadata = create_test_metadata();
        let result = format_metadata(&metadata, "text");

        // Verify text format
        assert!(result.contains("File: test.swf"));
        assert!(result.contains("Stage Size: (800, 600)"));
        assert!(result.contains("Number of Frames: 100"));
        assert!(result.contains("Frame Rate: 30"));
    }

    #[test]
    fn test_output_path_generation_json() {
        let input = PathBuf::from("test.swf");
        let output = input.with_extension("swf.json");
        assert_eq!(output.to_str().unwrap(), "test.swf.json");
    }

    #[test]
    fn test_output_path_generation_yaml() {
        let input = PathBuf::from("test.swf");
        let output = input.with_extension("swf.yaml");
        assert_eq!(output.to_str().unwrap(), "test.swf.yaml");
    }

    #[test]
    fn test_output_path_generation_text() {
        let input = PathBuf::from("test.swf");
        let output = input.with_extension("swf.text");
        assert_eq!(output.to_str().unwrap(), "test.swf.text");
    }

    #[test]
    fn test_output_path_with_directory() {
        let input = PathBuf::from("/path/to/animation.swf");
        let output = input.with_extension("swf.json");
        assert_eq!(output.to_str().unwrap(), "/path/to/animation.swf.json");
    }

    #[test]
    fn test_metadata_equality() {
        let metadata1 = create_test_metadata();
        let metadata2 = Metadata {
            file_name: "test.swf".to_string(),
            stage_size: (800, 600),
            no_of_frames: 100,
            frame_rate: 30,
        };
        assert_eq!(metadata1, metadata2);
    }

    #[test]
    fn test_metadata_inequality() {
        let metadata1 = create_test_metadata();
        let metadata2 = Metadata {
            file_name: "different.swf".to_string(),
            stage_size: (800, 600),
            no_of_frames: 100,
            frame_rate: 30,
        };
        assert_ne!(metadata1, metadata2);
    }

    #[test]
    fn test_is_swf_extension_lowercase() {
        let path = PathBuf::from("test.swf");
        let ext = path.extension().unwrap().to_str().unwrap().to_lowercase();
        assert_eq!(ext, "swf");
    }

    #[test]
    fn test_is_swf_extension_uppercase() {
        let path = PathBuf::from("test.SWF");
        let ext = path.extension().unwrap().to_str().unwrap().to_lowercase();
        assert_eq!(ext, "swf");
    }

    #[test]
    fn test_is_swf_extension_mixed_case() {
        let path = PathBuf::from("test.SwF");
        let ext = path.extension().unwrap().to_str().unwrap().to_lowercase();
        assert_eq!(ext, "swf");
    }

    #[test]
    fn test_format_metadata_default_to_text() {
        let metadata = create_test_metadata();
        let result = format_metadata(&metadata, "unknown_format");

        // Should default to text format
        assert!(result.contains("File: test.swf"));
        assert!(result.contains("Stage Size: (800, 600)"));
    }
}
