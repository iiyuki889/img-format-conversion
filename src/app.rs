use crate::converter::{
    ConvertFormat, IMAGE_EXTENSIONS, read_metadata, save_image, save_jpeg_with_metadata,
};
use eframe::egui;
use image::ImageFormat;
use std::path::PathBuf;

fn load_texture(ctx: &egui::Context, image: &image::DynamicImage) -> egui::TextureHandle {
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());

    ctx.load_texture(
        "selected-image",
        color_image,
        egui::TextureOptions::default(),
    )
}

// -------- GUI --------
// GUI font setting
pub fn setup_fonts(ctx: &egui::Context) {
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
pub struct MyApp {
    active_tab: AppTab,
    selected_file: Option<PathBuf>,
    selected_files: Vec<PathBuf>,
    selected_index: usize,
    texture: Option<egui::TextureHandle>,
    selected_format: ConvertFormat,
    selected_img: Option<image::DynamicImage>,
    selected_img_format: Option<ImageFormat>,
    remove_metadata_enabled: bool,
    custom_filename_enabled: bool,
    output_filename: String,
    status_message: String,
    status_kind: StatusKind,
    metadata: Vec<(String, String)>,

    rename_files: Vec<PathBuf>,
    rename_mode: RenameMode,
    rename_location_enabled: bool,
    rename_location: String,
    rename_custom_name: String,
    rename_status_message: String,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum AppTab {
    #[default]
    Convert,
    Rename,
}

#[derive(Default, PartialEq)]
enum StatusKind {
    #[default]
    Idle,
    Info,
    Processing,
    Success,
    Error,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum RenameMode {
    #[default]
    ExifTemplate,
    Custom,
}

impl MyApp {
    fn open_image(&mut self, ctx: &egui::Context) {
        let Some(paths) = rfd::FileDialog::new()
            .add_filter("Image", IMAGE_EXTENSIONS)
            .pick_files()
        else {
            return;
        };

        if paths.is_empty() {
            return;
        }

        self.selected_files = paths;
        self.selected_index = 0;

        if let Some(first_path) = self.selected_files.first().cloned() {
            self.load_image_from_path(ctx, first_path);
        }

        self.status_kind = StatusKind::Info;
        self.status_message = format!("{}枚の画像を選択しました", self.selected_files.len());
    }

    fn selected_image(&mut self, ctx: &egui::Context, index: usize) {
        let Some(path) = self.selected_files.get(index).cloned() else {
            return;
        };
        self.selected_index = index;
        self.load_image_from_path(ctx, path);
    }

    fn load_image_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        let reader = match image::ImageReader::open(&path) {
            Ok(reader) => reader,
            Err(error) => {
                self.status_kind = StatusKind::Error;
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
                self.metadata = read_metadata(&path).unwrap_or_default();
                self.texture = Some(load_texture(ctx, &image));
                self.selected_file = Some(path.clone());
                self.selected_img = Some(image);
                self.selected_img_format = image_format;
                self.status_kind = StatusKind::Info;
                self.status_message = format!("画像を開きました: {}", path.display());
            }
            Err(error) => {
                self.texture = None;
                self.selected_file = None;
                self.selected_img = None;
                self.selected_img_format = None;
                self.metadata.clear();
                self.status_message = format!("画像を読み込めませんでした: {error}");
            }
        }
    }

    fn clear_image(&mut self) {
        self.selected_file = None;
        self.selected_files.clear();
        self.selected_index = 0;
        self.output_filename.clear();
        self.custom_filename_enabled = false;
        self.texture = None;
        self.selected_img = None;
        self.selected_img_format = None;
        self.metadata.clear();

        self.status_kind = StatusKind::Info;
        self.status_message = "現在の画像を解除しました".to_string();
    }

    fn open_rename_files(&mut self) {
        let Some(paths) = rfd::FileDialog::new()
            .add_filter("Image", IMAGE_EXTENSIONS)
            .pick_files()
        else {
            return;
        };

        if paths.is_empty() {
            return;
        }

        self.rename_files = paths;
        self.rename_status_message =
            format!("{}個のファイルを選択しました", self.rename_files.len());
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui|{
                // ドロップファイルの取得
                let dropped_path = ui.ctx().input(|input| {input.raw.dropped_files.first().map(|file| file.path().to_path_buf())});
                if let Some(path) = dropped_path{
                    self.load_image_from_path(ui.ctx(), path);
                }
                // title
                ui.heading("Image format converter tool");
                ui.add_space(12.0);

                ui.horizontal(|ui|{
                    ui.selectable_value(&mut self.active_tab, AppTab::Convert, "フォーマット変換");
                    ui.selectable_value(&mut self.active_tab, AppTab::Rename, "一括リネーム");
                });

                ui.separator();
                ui.add_space(8.0);

                // リネームのタブの切り替え
                if self.active_tab == AppTab::Rename {
                    ui.heading("一括リネーム");
                    ui.add_space(12.0);

                    ui.group(|ui| {
                        ui.set_min_width(500.0);
                        ui.heading("リネーム対象");
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button("ファイルを複数選択").clicked(){
                                self.open_rename_files();
                            }

                            let clear_enabled = !self.rename_files.is_empty();

                            if ui.add_enabled(clear_enabled, egui::Button::new("選択を解除")).clicked(){
                                self.rename_files.clear();
                                self.rename_status_message.clear();
                            }
                        });

                        ui.add_space(8.0);

                        if self.rename_files.is_empty(){
                            ui.label("ファイルはまた選択されていません");
                        } else {
                            ui.label(format!("選択数: {}個", self.rename_files.len()));

                            egui::ScrollArea::vertical()
                            .id_salt("rename_file_list")
                            .max_height(160.0)
                            .show(ui, |ui|{
                                for path in &self.rename_files{
                                    let file_name = path
                                    .file_name()
                                    .and_then(|name| name
                                        .to_str())
                                        .unwrap_or("不明");

                                    ui.label(file_name);
                                }
                            });
                        }
                    });

                    ui.add_space(12.0);

                    ui.group(|ui| {
                        ui.set_min_width(500.0);
                        ui.heading("リネーム設定");
                        ui.add_space(8.0);

                        ui.radio_value(&mut self.rename_mode, RenameMode::ExifTemplate, "exifから名前を作成");

                        ui.label("形式: YYYYMMDD_撮影カメラ_連番.format");

                        ui.add_space(8.0);

                        ui.add_enabled_ui(self.rename_mode == RenameMode::ExifTemplate, |ui|{
                            ui.horizontal(|ui|{
                                ui.checkbox(&mut self.rename_location_enabled, "場所を追加");
                            });

                            ui.add_enabled(self.rename_location_enabled, egui::TextEdit::singleline(&mut self.rename_location).hint_text("例: 東京"));
                            if self.rename_location_enabled{
                                ui.label("形式: YYYYMMDD_撮影カメラ_連番.format");
                            }
                        });
                        ui.add_space(12.0);
                        ui.radio_value(&mut self.rename_mode, RenameMode::Custom, "任意の名前を使用");
                        ui.add_enabled(self.rename_mode == RenameMode::Custom, egui::TextEdit::singleline(&mut self.rename_custom_name).hint_text("例: 旅行写真"));
                        if self.rename_mode == RenameMode::Custom {
                            ui.label("形式: 任意名_連番.format");
                        }
                    });

                    ui.add_space(12.0);

                    let setting_ready = match self.rename_mode {
                        RenameMode::ExifTemplate => {
                            !self.rename_files.is_empty() && (!self.rename_location_enabled || !self.rename_location.trim().is_empty())
                        },
                        RenameMode::Custom => {
                            !self.rename_files.is_empty() && !self.rename_custom_name.trim().is_empty()
                        }
                    };

                    ui.add_enabled(false && setting_ready, egui::Button::new("一括リネームを実行"));
                    ui.label("リネーム処理は次の段階で有効にします");

                    if !self.rename_status_message.is_empty(){
                        ui.separator();
                        ui.colored_label(egui::Color32::LIGHT_BLUE, &self.rename_status_message);
                    }
                    return;
                }

                let available_width = ui.available_width();
                let preview_width = (available_width * 0.62).clamp(300.0, 500.0);
                let preview_height = (preview_width * 0.75).clamp(100.0, 500.0);
                let preview_size = egui::vec2(preview_width, preview_height);

                // 画像のプレビュー領域
                ui.horizontal_top(|ui|{
                    ui.vertical(|ui|{
                        ui.group(|ui| {
                            ui.add_space(10.0);

                            let (rect, _response) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
                            ui.painter().rect_filled(rect,8.0,egui::Color32::from_gray(30),);
                            if let Some(texture)= &self.texture{
                                let original_size = texture.size_vec2();
                                let width_scale = rect.width() / original_size.x;
                                let height_scale = rect.height() / original_size.y;
                                let scale = width_scale.min(height_scale).min(1.0);
                                let display_size = original_size * scale;
                                let image_rect = egui::Rect::from_center_size(rect.center(), display_size,);
                                ui.painter().image(texture.id(),image_rect,egui::Rect::from_min_max(egui::pos2(0.0, 0.0),egui::pos2(1.0,1.0),),egui::Color32::WHITE,);
                            }else {
                                ui.painter().text(rect.center(),egui::Align2::CENTER_CENTER,"画像をドロップしてください",egui::FontId::proportional(18.0),egui::Color32::GRAY,);
                            }

                            ui.horizontal(|ui| {
                                let image_count = self.selected_files.len();
                                let previous_enabled = image_count > 1 && self.selected_index >0;

                                if ui.add_enabled(previous_enabled, egui::Button::new("前へ")).clicked() {
                                    self.selected_image(ui.ctx(), self.selected_index -1);
                                }

                                if image_count > 0 {
                                    ui.label(format!("{}/{}",self.selected_index + 1, image_count));
                                }

                                let next_enabled = image_count > 1 && self.selected_index + 1 < image_count;

                                if ui.add_enabled(next_enabled, egui::Button::new("次へ")).clicked(){
                                    self.selected_image(ui.ctx(), self.selected_index + 1);
                                }
                            });

                            // 画像を開く
                            ui.horizontal(|ui| {
                                if ui.button("画像を開く").clicked() {
                                    self.open_image(ui.ctx());
                                }
                                let clear_enabled =self.selected_img.is_some();
                                let clear_button = ui.add_enabled(clear_enabled,egui::Button::new("選択を解除"),);
                                if clear_button.clicked() {self.clear_image();
                                }
                            });
                        });
                    });

                        ui.add_space(16.0);
                        let info_width = (ui.available_width() - 16.0).max(120.0);

                        // 入力情報スペース
                        ui.vertical(|ui| {
                            let image_info_height = 130.0;
                            let group_spacing = 8.0;
                            let metadata_height = (preview_height - image_info_height - group_spacing).max(100.0);

                            ui.group(|ui|{
                                ui.set_width(info_width);
                                ui.set_height(image_info_height);
                                ui.vertical(|ui|{
                                    ui.heading("画像情報");
                                    ui.add_space(8.0);

                                    if !self.selected_files.is_empty(){
                                        ui.label(format!(
                                            "選択枚数: {}枚",self.selected_files.len())
                                        );
                                    }
                                    if let (Some(path),Some(image)) = (&self.selected_file, &self.selected_img) {
                                        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("不明");
                                        ui.label(format!("ファイル名: {file_name}"));
                                        ui.label(format!("画像サイズ: {} * {} px",image.width(),image.height()));
                                        if let Some(format) = self.selected_img_format {
                                            ui.label(format!("画像形式: {format:?}"));
                                        }
                                        match std::fs::metadata(path) {Ok(metadata) => {
                                            let file_size_kb = metadata.len() as f64 /1024.0;
                                            ui.label(format!("ファイル容量: {file_size_kb:.1} kB"));
                                        }Err(_) => {
                                            ui.label("画像が選択されていません");
                                        }}
                                    }
                                });
                            });

                                ui.add_space(group_spacing);

                                //metadata表示
                                ui.group(|ui|{
                                    ui.set_width(info_width);
                                    ui.set_height(metadata_height);

                                    ui.vertical(|ui|{
                                        ui.heading("メタデータ");
                                        ui.add_space(6.0);

                                        let scroll_height = (metadata_height - 45.0).max(50.0);

                                        egui::ScrollArea::vertical()
                                        .id_salt("metadata_scroll")
                                        .max_height(scroll_height)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            if self.metadata.is_empty(){
                                            ui.label("メタデータはありません");
                                            } else {
                                                for (name, value) in &self.metadata{
                                                    ui.label(egui::RichText::new(name).strong(),);
                                                    ui.add(egui::Label::new(value).wrap());
                                                    ui.separator();
                                                }
                                            }
                                        });
                                    });
                                });
                        });
                });

                // selcet image format
                egui::ComboBox::from_label("変換形式")
                .selected_text(self.selected_format.extension())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Jpeg, "jpg");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Png, "png");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::WebP, "webp");
                    ui.selectable_value(&mut self.selected_format, ConvertFormat::Ico, "ico");
                });

                ui.horizontal(|ui|{
                    let checkbox = ui.checkbox(&mut self.custom_filename_enabled, "保存名を指定");

                    if checkbox.changed()
                    && self.custom_filename_enabled
                    && self.output_filename.is_empty()
                    && let Some(path) = &self.selected_file
                    && let Some(file_stem) = path.file_stem()
                    && let Some(file_stem) = file_stem.to_str()
                    {
                        self.output_filename = file_stem.to_string();
                    }

                    ui.add_enabled(self.custom_filename_enabled, egui::TextEdit::singleline(&mut self.output_filename,).hint_text("保存名"));

                    if self.custom_filename_enabled {
                        ui.label(format!(".{}", self.selected_format.extension()));
                    }
                });

                    let metadata_copy_enabled = self.selected_format == ConvertFormat::Jpeg;
                    ui.add_enabled_ui(metadata_copy_enabled, |ui|{
                    ui.checkbox(&mut self.remove_metadata_enabled, "GPS関連のメタデータは引き継がない");
                    });
                    if !metadata_copy_enabled {
                        ui.label("メタデータの引継ぎはJPEGのみ対応しています");
                    }

                let convert_enabled = self.selected_img.is_some();
                let convert_button = ui.add_enabled(convert_enabled, egui::Button::new("変換開始"));
                if convert_button.clicked()

                && let Some(image) = &self.selected_img {

                    let extension = self.selected_format.extension();
                    let entered_name = self.output_filename.trim();

                    let default_name = if self.custom_filename_enabled && !entered_name.is_empty(){
                        format!("{entered_name}.{extension}")
                    }else {
                        format!("converted.{extension}")
                    };

                    if let Some(output_path) = rfd::FileDialog::new()
                    .add_filter("変換後の画像", &[extension])
                    .set_file_name(&default_name)
                    .save_file()
                    {
                        self.status_kind = StatusKind::Processing;
                        self.status_message = "画像を変換中".to_string();

                        let Some(input_path) = self.selected_file.as_deref() else {
                            self.status_kind = StatusKind::Error;
                            self.status_message = "変換元のパスを取得できませんでした".to_string();
                            return;
                        };

                        let save_result: Result<(), Box<dyn std::error::Error>> =
                            if self.selected_format == ConvertFormat::Jpeg {
                                save_jpeg_with_metadata(
                                    image,
                                    input_path,
                                    &output_path,
                                    self.remove_metadata_enabled
                                )
                            }else {
                                save_image(image, &output_path, self.selected_format)
                                .map_err(|error|{Box::new(error) as Box<dyn std::error::Error>})
                            };
                            match save_result {
                                Ok(_) => {
                                    self.status_kind = StatusKind::Success;
                                    if self.selected_format == ConvertFormat::Jpeg {
                                            if self.remove_metadata_enabled{
                                                self.status_message = format!(
                                                    "JPEG画像を変換し、メタデータを引き継ぎました(GPSを除外)。: {}",
                                                    output_path.display()
                                                );
                                            }else {
                                                self.status_message = format!(
                                                    "JPEG画像を変換し、メタデータを引き継ぎました: {}",
                                                    output_path.display()
                                                );
                                            }
                                        } else {
                                            self.status_message = format!("画像を変換しました（メタデータの引継ぎはJPEGのみ対応):{}", output_path.display());
                                        }
                                }
                                Err(error) => {
                                    self.status_kind = StatusKind::Error;
                                        self.status_message = format!("画像は保存しましたが、メタデータの引継ぎに失敗しました。: {error}");
                                    }
                            }
                        }
                }

                if !self.status_message.is_empty() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        match self.status_kind {
                            StatusKind::Idle => {}
                            StatusKind::Info => {
                                ui.colored_label(egui::Color32::LIGHT_BLUE, &self.status_message,);
                            }
                            StatusKind::Processing => {
                                ui.add(egui::Spinner::new());
                                ui.label(&self.status_message);
                            }
                            StatusKind::Success => {
                                ui.colored_label(egui::Color32::LIGHT_GREEN, &self.status_message,);
                            }
                            StatusKind::Error => {ui.colored_label(egui::Color32::LIGHT_RED, &self.status_message,);
                            }
                        }
                    });
                }
            });
        });
    }
}
