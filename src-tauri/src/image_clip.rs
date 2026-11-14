use image::{GenericImageView, ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};
use std::io::Cursor;

const MAX_SAVED_PNG_BYTES: usize = 20 * 1024 * 1024;
const MAX_PIXELS: u64 = 30_000_000;

pub struct StoredImage {
    pub png: Vec<u8>,
    pub thumbnail: Vec<u8>,
    pub fingerprint: String,
    pub width: u32,
    pub height: u32,
}

pub fn prepare(bytes: &[u8], format: &str) -> Result<StoredImage, String> {
    let image_format = match format {
        "png" => ImageFormat::Png,
        "tiff" => ImageFormat::Tiff,
        _ => return Err("未対応の画像形式です".into()),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), image_format);
    let mut limits = Limits::default();
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|error| error.to_string())?;
    let (width, height) = decoded.dimensions();
    if width == 0 || height == 0 || (width as u64 * height as u64) > MAX_PIXELS {
        return Err("画像の解像度が大きすぎます".into());
    }

    // Keep PNG byte-for-byte if it is already PNG; otherwise convert TIFF to PNG.
    let png = if format == "png" {
        bytes.to_vec()
    } else {
        let mut png = Vec::new();
        decoded
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .map_err(|error| error.to_string())?;
        png
    };
    if png.len() > MAX_SAVED_PNG_BYTES {
        return Err("保存できる画像サイズ（20MB）を超えています".into());
    }

    // Bound IPC traffic: only a small thumbnail is sent to React.
    let preview = decoded.thumbnail(180, 112);
    let mut thumbnail = Vec::new();
    preview
        .write_to(&mut Cursor::new(&mut thumbnail), ImageFormat::Png)
        .map_err(|error| error.to_string())?;

    let fingerprint = format!("{:x}", Sha256::digest(&png));
    Ok(StoredImage {
        png,
        thumbnail,
        fingerprint,
        width,
        height,
    })
}
