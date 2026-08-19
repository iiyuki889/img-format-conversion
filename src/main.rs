use eframe::egui;
use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::path::PathBuf;
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

fn load_texture(ctx: &egui::Context, image: &image::DynamicImage) -> egui::TextureHandle {
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());

    ctx.load_texture("sekected-img", color_image, egui::TextureOptions::default())
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
            if ui.button("Open faile").clicked()
                && let Some(path) = rfd::FileDialog::new()
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

                    match image::open(&path) {
                        Ok(image) => {
                            //画像の表示
                            self.texture = Some(load_texture(ui.ctx(), &image));
                            self.selected_file = Some(path.clone());
                            self.selected_img = Some(image);
                            // 画像フォーマットの文字列の保存
                            self.selected_img_format = reader.format();
                            self.status_message = format!("画像を開きました: {}", path.display());
                        }
                        Err(error) => {
                            self.texture = None;
                            self.selected_img = None;
                            self.status_message = format!("画像を開けませんでした: {error}");
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

            if convert_button.clicked()
                && let Some(image) = &self.selected_img {
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
                                                }Err(error) => {
                                                    let message = format!("画像は変換しましたが、メタデータの消去に失敗しました: {error}");
                                                    eprintln!("{message}");self.status_message = message;
                                                }
                                            }
                                        } else {
                                            let message ="保存先のパスを文字列へ変換できませんでした".to_string();
                                            eprintln!("{message}");self.status_message = message;
                                        }
                                    } else {
                                        let message = format!("画像の変換が完了しました: {}",output_path.display());
                                        println!("{message}");self.status_message = message;
                                    }
                                }
                            Err(error) => {
                                let message =format!("画像の変換に失敗しました: {error}");
                                eprintln!("{message}");self.status_message = message;
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
