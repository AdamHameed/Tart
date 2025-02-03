use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use clap::{Arg, Command};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive, Builder, Header};

fn compress_files(input_files: &[&str], output: &str) -> io::Result<()> {
    let tar_gz = File::create(output)?;
    let encoder = GzEncoder::new(BufWriter::new(tar_gz), Compression::default());
    let mut tar = Builder::new(encoder);

    for file in input_files {
        let file_path = Path::new(file);
        if file_path.exists() {
            tar.append_path(file_path)?;
        } else {
            eprintln!("⚠️ Skipping missing file: {}", file);
        }
    }

    tar.finish()?;
    println!("✅ Compressed {} files into {}", input_files.len(), output);
    Ok(())
}

fn decompress_files(input: &str, output_dir: &str) -> io::Result<()> {
    let tar_gz = File::open(input)?;
    let decoder = GzDecoder::new(BufReader::new(tar_gz));
    let mut archive = Archive::new(decoder);

    archive.unpack(output_dir)?; // Extract all files into the output directory
    println!("✅ Extracted contents of {} to {}", input, output_dir);
    Ok(())
}
fn add_file_to_tar_gz(tar_gz_path: &str, file_path: &str) -> io::Result<()> {
    let tar_gz_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(tar_gz_path)?;

    let gz_encoder = GzEncoder::new(BufWriter::new(tar_gz_file), Compression::default());
    let mut tar_builder = Builder::new(gz_encoder);

    let file = File::open(file_path)?;
    let mut header = Header::new_gnu();
    header.set_path(Path::new(file_path))?;
    header.set_size(file.metadata()?.len());
    header.set_mode(0o755);
    header.set_mtime(0);

    tar_builder.append(&header, file)?;

    println!("✅ Added {} to {}", file_path, tar_gz_path);
    Ok(())
}
fn display_help() {
    println!(
        r#"
NAME
    tart - Compress and decompress files using Gzip and Tar

SYNOPSIS
    tart [OPTIONS] -i <INPUT> -o <OUTPUT>

DESCRIPTION
    Tart is a command-line utility to compress multiple files into a 
    single .tar.gz archive, extract .tar.gz archives, or append a file to an 
    existing .tar.gz archive.

OPTIONS
    -c, --compress
        Compress multiple files into a single .tar.gz archive.
    
    -d, --decompress
        Extract files from a .tar.gz archive.
    
    -a, --add
        Add a file to an existing .tar.gz archive.
    
    -i, --input <INPUT>
        Input file(s) for compression, or archive file for decompression.
        Accepts multiple files when compressing.

    -o, --output <OUTPUT>
        Output archive file (.tar.gz) or extraction directory.

    -h, --help
        Display this help message.

EXAMPLES
    Compress files into an archive:
        tart -c -i file1.txt file2.txt -o archive.tar.gz

    Decompress an archive:
        tart -d -i archive.tar.gz -o extracted_dir/

    Add a file to an existing archive:
        tart -a -i newfile.txt -o archive.tar.gz"#,
    );
}
fn main() {
    let matches = Command::new("Rust Compressor")
        .version("1.0")
        .about("Compress and decompress multiple files into a single Gzip archive")
        .disable_help_flag(true) 
        .arg(Arg::new("compress")
            .short('c')
            .long("compress")
            .help("Compress multiple files into a single .tar.gz file")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("decompress")
            .short('d')
            .long("decompress")
            .help("Extract files from a .tar.gz archive")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("add")
            .short('a')
            .long("add")
            .help("Add a file to an existing .tar.gz archive")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("help")
            .short('h')
            .long("help")
            .help("Display the help page")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .help("Input files (for compression) or archive (for decompression)")
            .required(false)
            .num_args(1..))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .help("Output archive file (.tar.gz) or extraction directory")
            .required(false)
            .num_args(1))
        .get_matches();

        if matches.get_flag("help") {
            display_help();
            return;
        }
    let output = matches.get_one::<String>("output").unwrap().as_str();

    if matches.get_flag("compress") {
        let input_files: Vec<_> = matches.get_many::<String>("input").unwrap().map(|s| s.as_str()).collect();
        compress_files(&input_files, output).expect("Compression failed");
    } else if matches.get_flag("decompress") {
        let input = matches.get_one::<String>("input").unwrap().as_str();
        decompress_files(input, output).expect("Decompression failed");
    }
    else if matches.get_flag("add") {
        let input = matches.get_one::<String>("input").unwrap().as_str();
        let file = matches.get_one::<String>("output").unwrap().as_str();
        add_file_to_tar_gz(input, file).expect("Adding file failed");
    } else {
        eprintln!("❌ Please specify --compress or --decompress");
    }
}
