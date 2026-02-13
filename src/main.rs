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

#[derive(serde::Serialize)]
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

    if format == "json" {
        return serde_json::to_string(&metadata).unwrap();
    } else if format == "yaml" {
        return serde_yaml::to_string(&metadata).unwrap();
    } else {
        return format!(
            "File: {}\nStage Size: {:?}\nNumber of Frames: {}\nFrame Rate: {}",
            metadata.file_name, metadata.stage_size, metadata.no_of_frames, metadata.frame_rate
        );
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
