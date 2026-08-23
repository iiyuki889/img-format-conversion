use exiftool_rs::ExifTool;
use image::{
    DynamicImage, ExtendedColorType, ImageFormat,
    codecs::ico::{IcoEncoder, IcoFrame},
    imageops::FilterType,
};
use std::{fs::File, path::Path};
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "ico"];
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ConvertFormat {
    #[default]
    Jpeg,
    Png,
    WebP,
    Gif,
    Ico,
}

impl ConvertFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Gif => "gif",
            Self::Ico => "ico",
        }
    }

    fn image_format(self) -> ImageFormat {
        match self {
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
            Self::WebP => ImageFormat::WebP,
            Self::Gif => ImageFormat::Gif,
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

pub fn remove_tags(input_img: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut exiftool = ExifTool::new();

    let tags = match exiftool.extract_info(input_img) {
        Ok(tags) => tags,
        Err(error) => {
            eprintln!("メタデータの取得に失敗しました: {error}");
            return Ok(());
        }
    };

    for tag in &tags {
        if tag.name.starts_with("GPS") {
            println!("削除対象: {}", tag.name);
            exiftool.set_new_value(&tag.name, None);
        }
    }

    match exiftool.write_info(input_img, input_img) {
        Ok(_) => {
            println!("GPS関連のタグを削除しました");
        }
        Err(error) => {
            eprintln!("メタデータの書き込みに失敗しました: {error}");
        }
    }

    Ok(())
}

pub fn save_ico(image: &DynamicImage, output_path: &Path) -> image::ImageResult<()> {
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
