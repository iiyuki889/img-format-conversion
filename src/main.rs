use eframe::egui;
use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::result::Result::Ok;

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
    selected_img: Option<image::DynamicImage>,
    selected_img_format: Option<ImageFormat>,
    remove_metadata_enabled: bool,
    status_message: String,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ConvertFormat {
    #[default]
    Jpeg,
    Png,
    WebP,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Image converter tool");

            // open folder
            if ui.button("Open faile").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["jpg", "jpeg", "png", "webp"])
                    .pick_file()
                {
                    let reader = match image::ImageReader::open(&path) {
                        Ok(reader) => reader,
                        Err(error) => {
                            eprintln!("ファイルを開けません: {error}");
                            return;
                        }
                    };

                    let reader = match reader.with_guessed_format() {
                        Ok(reader) => reader,
                        Err(error) => {
                            eprint!("画像形式を判定できません: {error}");
                            return;
                        }
                    };

                    if let Some(format) = reader.format() {
                        println!("画像フォーマット: {format:?}");
                    } else {
                        println!("画像フォーマットを判定できません");
                    }

                    // 画像フォーマットの文字列の保存
                    self.selected_img_format = reader.format();

                    // 画像を表示
                    self.texture = load_image(ui.ctx(), &path);
                    self.selected_file = Some(path.clone());

                    match image::open(&path) {
                        Ok(image) => {
                            self.selected_img = Some(image);
                            self.status_message = format!("画像を開きました: {}", path.display());
                        }
                        Err(error) => {
                            self.selected_img = None;
                            self.status_message = format!("画像を開けませんでした: {error}");
                        }
                    }
                }
            }
            if let Some(texture) = &self.texture {
                let image = egui::Image::new(texture).shrink_to_fit();
                ui.add(image);
            }

            if let Some(format) = self.selected_img_format {
                ui.label(format!("画像フォーマット: {format:?}"));
            }

            // selcet image format
            egui::ComboBox::from_label("変換形式")
                .selected_text(match self.selected_format {
                    ConvertFormat::Jpeg => "jpeg",
                    ConvertFormat::Png => "png",
                    ConvertFormat::WebP => "webp",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Jpeg, "jpg");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Png, "png");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::WebP, "webp");
                });

            ui.checkbox(
                &mut self.remove_metadata_enabled,
                "変換後にメタデータを消去する",
            );

            let convert_enabled = self.selected_img.is_some();

            let convert_button = ui.add_enabled(convert_enabled, egui::Button::new("変換開始"));

            if convert_button.clicked() {
                if let Some(image) = &self.selected_img {
                    let (extension, image_format) = match self.selected_format {
                        ConvertFormat::Jpeg => ("jpg", ImageFormat::Jpeg),
                        ConvertFormat::Png => ("png", ImageFormat::Png),
                        ConvertFormat::WebP => ("webp", ImageFormat::WebP),
                    };

                    let default_name = format!("converted.{extension}");

                    if let Some(output_path) = rfd::FileDialog::new()
                        .add_filter("変換後の画像", &[extension])
                        .set_file_name(&default_name)
                        .save_file()
                    {
                        match image.save_with_format(&output_path, image_format) {
    Ok(()) => {
        if self.remove_metadata_enabled {
            if let Some(output_path_str) = output_path.to_str() {
                match remove_tags(output_path_str) {
                    Ok(()) => {
                        let message = format!(
                            "画像を変換し、メタデータを消去しました: {}",
                            output_path.display()
                        );

                        println!("{message}");
                        self.status_message = message;
                    }
                    Err(error) => {
                        let message = format!(
                            "画像は変換しましたが、メタデータの消去に失敗しました: {error}"
                        );

                        eprintln!("{message}");
                        self.status_message = message;
                    }
                }
            } else {
                let message =
                    "保存先のパスを文字列へ変換できませんでした".to_string();

                eprintln!("{message}");
                self.status_message = message;
            }
        } else {
            let message = format!(
                "画像の変換が完了しました: {}",
                output_path.display()
            );

            println!("{message}");
            self.status_message = message;
        }
    }
    Err(error) => {
        let message =
            format!("画像の変換に失敗しました: {error}");

        eprintln!("{message}");
        self.status_message = message;
    }
}
                    }
                }
            }

            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
            }
        });
    }
}
