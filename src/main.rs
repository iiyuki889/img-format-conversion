#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use exiftool_rs::ExifTool;
use image::ImageFormat;
use std::path::PathBuf;
use std::result::Result::Ok;

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "ico"];
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Image format conversion",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}

fn load_icon() -> eframe::egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/app_icon.png"))
        .expect("Failed to load icon")
        .into_rgba8();

    let width = image.width();
    let height = image.height();

    eframe::egui::IconData {
        rgba: image.into_raw(),
        height,
        width,
    }
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
        .push("japanese".to_owned());

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
    //image_texture: Option<egui::TextureHandle>,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ConvertFormat {
    #[default]
    Jpeg,
    Png,
    WebP,
    Gif,
    Ico,
}

impl ConvertFormat {
    fn extension(self) -> &'static str {
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

impl MyApp {
    fn open_image(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Image", IMAGE_EXTENSIONS)
            .pick_file()
        else {
            return;
        };

        let reader = match image::ImageReader::open(&path) {
            Ok(reader) => reader,
            Err(error) => {
                self.status_message = format!("ファイルを開けませんでした: {error}");
                return;
            }
        };

        let reader = match reader.with_guessed_format() {
            Ok(reader) => reader,
            Err(error) => {
                self.status_message = format!("画像を判定できませんでした。: {error}");
                return;
            }
        };

        let image_format = reader.format();

        match reader.decode() {
            Ok(image) => {
                self.texture = Some(load_texture(ctx, &image));
                self.selected_file = Some(path.clone());
                self.selected_img = Some(image);
                self.selected_img_format = image_format;
                self.status_message = format!("画像を開きました: {}", path.display());
            }
            Err(error) => {
                self.texture = None;
                self.selected_file = None;
                self.selected_img = None;
                self.selected_img_format = None;
                self.status_message = format!("画像を読み込めませんでした: {error}");
            }
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui|{
            ui.heading("Image format converter tool");

            // 画像のプレビュー領域
            ui.group(|ui| {
                ui.add_space(10.0);

                let preview_width = (ui.available_width() -20.0).clamp(100.0, 500.0);
                let preview_height = (preview_width * 0.75).clamp(100.0, 500.0);
                let preview_size = egui::vec2(preview_width, preview_height);
                let (rect, _response) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    8.0,
                    egui::Color32::from_gray(30),
                );
                if let Some(texture)= &self.texture{
                    let original_size = texture.size_vec2();
                    let width_scale = rect.width() / original_size.x;
                    let height_scale = rect.height() / original_size.y;
                    let scale = width_scale.min(height_scale).min(1.0);
                    let display_size = original_size * scale;
                    let  image_rect = egui::Rect::from_center_size(rect.center(), display_size,);

                ui.painter().image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(
                        egui::pos2(0.0, 0.0),
                        egui::pos2(1.0,1.0),
                        ), 
                        egui::Color32::WHITE,);
                }else {
                    ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "画像をドロップしてください",
                    egui::FontId::proportional(18.0),
                    egui::Color32::GRAY,
            );
                }
                
        });
            
            // 画像を開く
            if ui.button("Open file").clicked() {self.open_image(ui.ctx());}

            if let Some(format) = self.selected_img_format {
                ui.label(format!("画像フォーマット: {format:?}"));
            }

            // selcet image format
            egui::ComboBox::from_label("変換形式")
                .selected_text(self.selected_format.extension())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Jpeg, "jpg");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Png, "png");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::WebP, "webp");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Gif, "gif");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Ico, "ico");
                });

            ui.checkbox(&mut self.remove_metadata_enabled,"変換後にGPS関連のメタデータを消去する",);

            let convert_enabled = self.selected_img.is_some();

            let convert_button = ui.add_enabled(convert_enabled, egui::Button::new("変換開始"));

            if convert_button.clicked()
                && let Some(image) = &self.selected_img {
                    let extension = self.selected_format.extension();
                    let image_format = self.selected_format.image_format();

                    let default_name = format!("converted.{extension}");

                    if let Some(output_path) = rfd::FileDialog::new()
                        .add_filter("変換後の画像", &[extension])
                        .set_file_name(&default_name)
                        .save_file()
                    {
                        let save_result = if self.selected_format == ConvertFormat::Ico{
                        let resized_image_ico = image.thumbnail(256,256).to_rgba8();
                        resized_image_ico.save_with_format(&output_path,image_format)
                    } else {
                        image.save_with_format(&output_path, image_format)
                    };

                        match save_result {
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
    });
    }
}
