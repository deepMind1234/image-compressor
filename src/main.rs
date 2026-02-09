use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "compress")]
#[command(about = "Compress images to a target size constraint", long_about = None)]
struct Args {
    /// Input image file
    input: PathBuf,

    /// Output image file
    output: PathBuf,

    /// Maximum size (e.g., 500KB, 1MB)
    #[arg(long)]
    ms: String,

    /// Output format (optional, auto-detects from output extension if not provided)
    #[arg(long, value_enum, default_value_t = Format::Auto)]
    format: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Format {
    Auto,
    Jpeg,
    Png,
    Webp,
}

fn get_file_size(path: &Path) -> Result<u64> {
    Ok(std::fs::metadata(path)?.len())
}

fn compress_jpeg(img: &DynamicImage, target_size: u64) -> Result<Vec<u8>> {
    let mut quality = 90;
    let mut scale = 1.0;
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let mut cursor = Cursor::new(&mut buffer);
        
        // Use zune-jpeg for high performance encoding if possible or just image crate
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        
        let processed_img = if scale < 1.0 {
            let (w, h) = img.dimensions();
            img.resize(
                (w as f32 * scale) as u32,
                (h as f32 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img.clone()
        };

        encoder.encode_image(&processed_img)?;
        
        if buffer.len() as u64 <= target_size || (quality <= 10 && scale <= 0.1) {
            break;
        }

        if quality > 10 {
            quality -= 10;
        } else {
            scale -= 0.1;
        }
    }

    Ok(buffer)
}

fn compress_png(img: &DynamicImage, target_size: u64) -> Result<Vec<u8>> {
    let mut scale = 1.0;
    
    loop {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        let processed_img = if scale < 1.0 {
            let (w, h) = img.dimensions();
            img.resize(
                (w as f32 * scale) as u32,
                (h as f32 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img.clone()
        };

        processed_img.write_to(&mut cursor, ImageFormat::Png)?;

        // Now optimize with oxipng
        let options = oxipng::Options::from_preset(2); // Balanced preset
        let optimized = oxipng::optimize_from_memory(&buffer, &options)
            .map_err(|e| anyhow!("Oxipng error: {}", e))?;

        if optimized.len() as u64 <= target_size || scale <= 0.1 {
            return Ok(optimized);
        }

        scale -= 0.1;
    }
}

fn compress_webp(img: &DynamicImage, target_size: u64) -> Result<Vec<u8>> {
    let mut quality = 80.0;
    let mut scale = 1.0;
    
    loop {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        let processed_img = if scale < 1.0 {
            let (w, h) = img.dimensions();
            img.resize(
                (w as f32 * scale) as u32,
                (h as f32 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img.clone()
        };

        // WebP encoding via image crate
        processed_img.write_to(&mut cursor, ImageFormat::WebP)?;
        // Note: image crate's webp doesn't expose quality easily in a generic write_to. 
        // We might need a specific encoder if we want iterative quality reduction for WebP.
        // For now, let's focus on scale for WebP if quality isn't easily reachable via image crate.
        
        if buffer.len() as u64 <= target_size || scale <= 0.1 {
            return Ok(buffer);
        }

        scale -= 0.1;
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let target_size = parse_size::parse_size(&args.ms)
        .map_err(|_| anyhow!("Invalid size constraint: {}", args.ms))?;

    println!("Target size: {} bytes", target_size);

    // Detect format
    let img = image::open(&args.input)?;
    let format = if args.format == Format::Auto {
        match args.output.extension().and_then(|s| s.to_str()) {
            Some("jpg") | Some("jpeg") => Format::Jpeg,
            Some("png") => Format::Png,
            Some("webp") => Format::Webp,
            _ => Format::Jpeg, // Default to JPEG if unknown
        }
    } else {
        args.format
    };

    println!("Identified format: {:?}", format);

    let compressed_data = match format {
        Format::Jpeg | Format::Auto => compress_jpeg(&img, target_size)?,
        Format::Png => compress_png(&img, target_size)?,
        Format::Webp => compress_webp(&img, target_size)?,
    };

    let mut out_file = File::create(&args.output)?;
    out_file.write_all(&compressed_data)?;

    let final_size = get_file_size(&args.output)?;
    println!("Final size: {} bytes", final_size);
    if final_size > target_size {
        println!("Warning: Could not meet target size constraint within quality limits.");
    } else {
        println!("Success: Image compressed within target size.");
    }

    Ok(())
}
