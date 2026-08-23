use exiftool_rs::ExifTool;
use image::{DynamicImage, ImageFormat};
use std::path::Path;

pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "ico"];

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
    let image_format = format.image_format();

    if format == ConvertFormat::Ico {
        let resized_image = image.thumbnail(256, 256).to_rgba8();

        resized_image.save_with_format(output_path, image_format)
    } else {
        image.save_with_format(output_path, image_format)
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
