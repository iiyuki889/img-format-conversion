use exiftool_rs::ExifTool;
use image::{
    DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat,
    codecs::{
        ico::{IcoEncoder, IcoFrame},
        jpeg::JpegEncoder,
    },
    imageops::FilterType,
};
use little_exif::{ifd::ExifTagGroup, metadata::Metadata};
use std::{fs::File, path::Path};
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "ico"];
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ConvertFormat {
    #[default]
    Jpeg,
    Png,
    WebP,
    Ico,
}

impl ConvertFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Ico => "ico",
        }
    }

    fn image_format(self) -> ImageFormat {
        match self {
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
            Self::WebP => ImageFormat::WebP,
            Self::Ico => ImageFormat::Ico,
        }
    }
}

pub fn save_image(
    image: &DynamicImage,
    output_path: &Path,
    format: ConvertFormat,
) -> image::ImageResult<()> {
    if format == ConvertFormat::Ico {
        save_ico(image, output_path)
    } else {
        image.save_with_format(output_path, format.image_format())
    }
}

pub fn save_jpeg_with_metadata(
    image: &DynamicImage,
    input_path: &Path,
    output_path: &Path,
    remove_gps: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let reader = image::ImageReader::open(input_path)?.with_guessed_format()?;
    let mut decoder = reader.into_decoder()?;
    let exif = decoder.exif_metadata()?;
    let icc_profile = decoder.icc_profile()?;
    let output_file = File::create(output_path)?;
    let mut encoder = JpegEncoder::new(output_file);

    if let Some(exif) = exif {
        encoder.set_exif_metadata(exif)?;
    }

    if let Some(icc_profile) = icc_profile {
        encoder.set_icc_profile(icc_profile)?;
    }
    let rgb_image = image.to_rgb8();

    encoder.encode(
        rgb_image.as_raw(),
        rgb_image.width(),
        rgb_image.height(),
        ExtendedColorType::Rgb8,
    )?;
    if remove_gps {
        remove_gps_metadata(output_path)?;
    }
    Ok(())
}

fn remove_gps_metadata(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = Metadata::new_from_path(output_path)?;

    for tag_id in 0x0000..=0x001f {
        metadata.remove_tag_by_hex_group(tag_id, ExifTagGroup::GPS);
    }
    metadata.remove_tag_by_hex_group(0x8825, ExifTagGroup::GENERIC);
    metadata.write_to_file(output_path)?;
    Ok(())
}

fn save_ico(image: &DynamicImage, output_path: &Path) -> image::ImageResult<()> {
    let mut frames = Vec::with_capacity(ICO_SIZES.len());
    for &size in ICO_SIZES {
        let resized_image = image
            .resize_exact(size, size, FilterType::Lanczos3)
            .to_rgba8();
        let frame = IcoFrame::as_png(resized_image.as_raw(), size, size, ExtendedColorType::Rgba8)?;
        frames.push(frame);
    }

    let output_file = File::create(output_path)?;
    IcoEncoder::new(output_file).encode_images(&frames)
}

pub fn read_metadata(
    input_path: &Path,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let exiftool = ExifTool::new();
    let tags = exiftool.extract_info(input_path)?;

    let metadata = tags
        .into_iter()
        .map(|tag| (tag.description, tag.print_value))
        .collect();

    Ok(metadata)
}
