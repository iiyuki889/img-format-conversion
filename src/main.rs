use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::env;
use std::error::Error;

// convert jpeg and png to webp
fn convert_image(
    input_img: &str,
    output_img: &str,
    format: ImageFormat,
) -> Result<(), Box<dyn Error>> {
    let img = match image::open(input_img) {
        Ok(image) => image,
        Err(error) => {
            eprintln!("画像を開けませんでした: {error}");
            return Ok(());
        }
    };

    println!("画像の読み込みに成功");
    img.save_with_format(output_img, format)?;
    Ok(())
}

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
        println!("{}: {}", tag.name, tag.print_value);
        if tag.name.starts_with("GPS") {
            println!("消去対象: {}", tag.name);
            exiftool.set_new_value(&tag.name, None);
        }
    }

    match exiftool.write_info(input_img, input_img) {
        Ok(_) => println!("GPS関連のタグを消去しました"),
        Err(error) => eprintln!("書き込みに失敗しました: {error}"),
    }

    println!("{}", "-".repeat(30));
    println!("消去できたかチェック");
    for tag in &tags {
        if tag.name.starts_with("GPS") {
            println!("{}: {}", tag.name, tag.print_value);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "画像ファイルと出力方式を指定してください(cargo run -- <img path> <output path> <format>"
        );
        return Ok(());
    }

    let input_img = &args[1];
    let output_img = &args[2];
    let format_str = &args[3].to_lowercase();

    let output_format = match format_str.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        _ => {
            eprintln!("対応していないフォーマットです: {format_str}");
            return Ok(());
        }
    };

    println!("入力ファイル: {}", input_img);
    println!("{:?}", output_format);
    println!("{}", "-".repeat(30));
    remove_tags(input_img)?;
    println!("{}", "-".repeat(30));
    convert_image(input_img, output_img, output_format)?;
    println!("変換完了");
    Ok(())
}
