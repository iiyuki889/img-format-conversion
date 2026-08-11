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
    convert_image(input_img, output_img, output_format)?;
    println!("変換完了");
    Ok(())
}
