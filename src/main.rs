use core::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::exit;

use clap::{Parser, ValueEnum};

// N64 header magic bytes
const BIG_ENDIAN: [u8; 4] = [0x80, 0x37, 0x12, 0x40];
const BYTE_SWAP: [u8; 4] = [0x37, 0x80, 0x40, 0x12];
const LITTLE_ENDIAN: [u8; 4] = [0x40, 0x12, 0x37, 0x80];

#[derive(Debug, PartialEq, Copy, Clone, ValueEnum)]
enum RomType {
    /// (commonly .z64)
    BigEndian,
    /// (commonly .v64)
    ByteSwap,
    /// (commonly .n64)
    LittleEndian,
}

impl fmt::Display for RomType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RomType::BigEndian => write!(f, "BigEndian (.z64)"),
            RomType::ByteSwap => write!(f, "ByteSwap (.v64)"),
            RomType::LittleEndian => write!(f, "LittleEndian (.n64)"),
        }
    }
}

impl RomType {
    fn get_file_ext(&self) -> &str {
        match *self {
            RomType::BigEndian => ".z64",
            RomType::ByteSwap => ".v64",
            RomType::LittleEndian => ".n64",
        }
    }

    fn get_header_bytes(&self) -> &[u8; 4] {
        match *self {
            RomType::BigEndian => &BIG_ENDIAN,
            RomType::ByteSwap => &BYTE_SWAP,
            RomType::LittleEndian => &LITTLE_ENDIAN,
        }
    }
}

fn guess_type(ext: &str) -> Option<RomType> {
    match ext.to_lowercase().as_str() {
        ".z64" => Some(RomType::BigEndian),
        ".v64" => Some(RomType::ByteSwap),
        ".n64" => Some(RomType::LittleEndian),
        _ => None,
    }
}

fn identify_header(bytes: &[u8; 4]) -> Option<RomType> {
    match *bytes {
        BIG_ENDIAN => Some(RomType::BigEndian),
        BYTE_SWAP => Some(RomType::ByteSwap),
        LITTLE_ENDIAN => Some(RomType::LittleEndian),
        _ => None,
    }
}

fn detect_ext(filename: &str) -> Option<&str> {
    if let Some(idx) = filename.rfind('.') {
        filename.get(idx..)
    } else {
        None
    }
}

fn swapper(bytes: &mut [u8; 4], src_type: RomType, dst_type: RomType) {
    match (src_type, dst_type) {
        (RomType::BigEndian, RomType::ByteSwap) | (RomType::ByteSwap, RomType::BigEndian) => {
            bytes.swap(0, 1);
            bytes.swap(2, 3);
        }
        (RomType::BigEndian, RomType::LittleEndian) | (RomType::LittleEndian, RomType::BigEndian) => {
            bytes.swap(0, 3);
            bytes.swap(1, 2);
        }
        (RomType::ByteSwap, RomType::LittleEndian) | (RomType::LittleEndian, RomType::ByteSwap) => {
            bytes.swap(0, 2);
            bytes.swap(1, 3);
        }
        _ => {}
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input Filename
    filename: String,

    /// Output filename
    destination_filename: Option<String>,

    /// Output type
    #[arg(short, long)]
    romtype: Option<RomType>,

    /// Identify rom type (and exit)
    #[arg(short, long, default_value_t = false)]
    identify: bool,

    /// Force overwrite output file
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

fn main() {
    let args = Args::parse();

    // Input file
    let Ok(file) = File::open(&args.filename) else {
        println!("Unable to open file: {}", &args.filename);
        exit(1)
    };
    let mut buf = BufReader::new(file);
    let mut bytes = [0; 4];

    // Let's read the header
    let Ok(_) = buf.read_exact(&mut bytes) else {
        println!("Error reading file: {}", &args.filename);
        exit(1);
    };

    let Some(filetype) = identify_header(&bytes) else {
        println!("File {} not recognized!", &args.filename);
        exit(1);
    };

    if args.identify {
        println!("File {} is {}", &args.filename, filetype);
        exit(0);
    }

    // Output file
    let outfiletype = args.romtype.unwrap_or_else(|| { // If specified, use that
        args.destination_filename
            .as_deref() // Otherwise borrow the destination filename
            .and_then(detect_ext) // Detect the extension
            .and_then(guess_type) // Identify the type based on extension
            .unwrap_or(RomType::BigEndian) // Or default to BigEndian
    });

    if filetype == outfiletype {
        println!("File is already {}!", outfiletype);
        exit(0);
    }

    let outfilename = args.destination_filename.unwrap_or_else(|| { // If specified, use that
        let mut name = args.filename.clone(); // Otherwise, copy the input filename
        let len = name.len(); // Get the filename length
        if name.chars().nth(len - 4) == Some('.') { // Check if there's a 3-letter extension
            name.truncate(len - 4); // Lop off the extension
        }
        name.push_str(outfiletype.get_file_ext()); // Add the standard extension for the output type
        name
    });

    if args.filename == outfilename {
        println!(
            "Input and Output filenames are identical {}, consider renaming input file",
            &outfilename
        );
        exit(1);
    }

    let outfile = match File::options()
        .write(true)
        .create_new(!args.force)
        .open(&outfilename)
    {
        Ok(file) => file,
        Err(error) => {
            println!(
                "Unable to open file {} for output. Error {}",
                &outfilename, error
            );
            exit(1);
        }
    };
    let mut outbuf = BufWriter::new(outfile);
    let Ok(_) = outbuf.write_all(outfiletype.get_header_bytes() ) else {
        println!("Unable to write to output file!");
        exit(1);
    };

    while buf.read_exact(&mut bytes).is_ok() {
        swapper(&mut bytes, filetype, outfiletype);

        let Ok(_) = outbuf.write_all(&bytes) else {
            println!("Error during output!");
            exit(1);
        };
    }
}
