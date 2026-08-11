use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::env;
use std::error::Error;
use std::path::Path;

// convert jpeg and png to webp
fn convert_image() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "画像ファイルと出力方式を指定してください(cargo run -- <img path> <output path> <format>"
        );
        return Ok(());
    }

    let input_img = &args[1];
    let format_str = &args[2].to_lowercase();

    let output_format = match format_str.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        _ => {
            eprintln!("対応していないフォーマットです: {format_str}");
            return Ok(());
        }
    };

    let output_path = make_output_path(input_img, format_str);

    let img = image::open(input_img)?;
    println!("{}", "-".repeat(30));
    println!("画像の読み込みに成功");
    img.save_with_format(&output_path, output_format)?;
    println!("変換完了");
    println!("{}", "-".repeat(30));
    let _ = remove_tags(&output_path);
    let _ = tags_cheack(&output_path);
    Ok(())
}

fn make_output_path(input_img: &str, extension: &str) -> String {
    let mut output = Path::new(input_img).to_path_buf();
    output.set_extension(extension);
    output.to_string_lossy().into_owned()
}

// remove photo metadata
fn remove_tags(input_img: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            println!("消去対象: {}", tag.name);
            exiftool.set_new_value(&tag.name, None);
        }
    }

    match exiftool.write_info(input_img, input_img) {
        Ok(_) => println!("GPS関連のタグを消去しました"),
        Err(error) => eprintln!("書き込みに失敗しました: {error}"),
    }

    Ok(())
}

// check deleted metadata
fn tags_cheack(output_img: &str) -> Result<(), Box<dyn std::error::Error>> {
    let exiftool = ExifTool::new();
    let tags = match exiftool.extract_info(output_img) {
        Ok(tags) => tags,
        Err(error) => {
            eprintln!("メタデータの取得に失敗しました: {error}");
            return Ok(());
        }
    };
    println!("{}", "-".repeat(30));
    println!("消去できたかチェック");
    for tag in &tags {
        if tag.name.starts_with("GPS") {
            println!("{}: {}", tag.name, tag.print_value);
        }
    }
    println!("{}", "-".repeat(30));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    convert_image()?;
    Ok(())
}
