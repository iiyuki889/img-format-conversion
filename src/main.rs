use eframe::egui;
use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Image Conversion",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    );
    Ok(())
}

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
    let _ = tags_check(&output_path);
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
fn tags_check(output_img: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// -------- GUI --------
// GUI font setting
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "japanese".to_owned(),
        egui::FontData::from_static(include_bytes!(
            r"../fonts/NotoSerifJP-VariableFont_wght.ttf"
        ))
        .into(),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "japanese".to_owned());

    ctx.set_fonts(fonts);
}

fn load_image(ctx: &egui::Context, path: &PathBuf) -> Option<egui::TextureHandle> {
    let image = image::open(path).ok()?;
    let image = image.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];

    let pixels = image.as_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);

    Some(ctx.load_texture("selected-img", color_image, egui::TextureOptions::default()))
}

#[derive(Default)]
struct MyApp {
    selected_file: Option<PathBuf>,
    texture: Option<egui::TextureHandle>,
    selected_format: ConvertFormat,
}

#[derive(Default, PartialEq)]
enum ConvertFormat {
    #[default]
    Jpeg,
    Png,
    WebP,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("画像変換ツール");

            egui::ComboBox::from_label("に変換")
                .selected_text(match self.selected_format {
                    ConvertFormat::Jpeg => "jpeg",
                    ConvertFormat::Png => "png",
                    ConvertFormat::WebP => "webp",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Jpeg, "jpg");
                    ui.selectable_value(&mut self.selected_format,ConvertFormat::Png, "png");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::WebP, "webp");
                });

            if ui.button("Dialog").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["jpg", "jpeg", "png", "webp"])
                    .pick_file()
                {
                    self.texture = load_image(ui.ctx(), &path);
                    self.selected_file = Some(path);
                }
            }
        });
    }
}
