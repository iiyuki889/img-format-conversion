use exiftool_rs::ExifTool;
use std::{
    collections::HashSet,
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct RenameEntry {
    pub path: PathBuf,
    pub taken_at: String,
    pub date_for_filename: String,
    pub camera: String,
}

#[derive(Debug, Clone)]
pub struct RenamePlan {
    pub source: PathBuf,
    pub target: PathBuf,
}

pub fn collect_rename_entries(
    paths: &[PathBuf],
) -> Result<Vec<RenameEntry>, Box<dyn std::error::Error>> {
    let exiftool = ExifTool::new();
    let mut entries = Vec::with_capacity(paths.len());

    for path in paths {
        let tags = exiftool.extract_info(path)?;

        let taken_at = tags
            .iter()
            .find(|tag| tag.name == "DateTimeOriginal")
            .or_else(|| tags.iter().find(|tag| tag.name == "CreateDate"))
            .map(|tag| tag.print_value.trim().to_string())
            .ok_or_else(|| {
                invalid_data_error(format!(
                    "撮影日時を取得できませんでした: {}",
                    path.display()
                ))
            })?;

        validate_exif_datetime(&taken_at, path)?;

        let camera = tags
            .iter()
            .find(|tag| tag.name == "Model")
            .map(|tag| sanitize_filename_part(&tag.print_value))
            .filter(|camera| !camera.is_empty())
            .ok_or_else(|| {
                invalid_data_error(format!(
                    "撮影カメラを取得できませんでした: {}",
                    path.display()
                ))
            })?;

        let date_for_filename = taken_at[..10].replace(':', "");

        entries.push(RenameEntry {
            path: path.clone(),
            taken_at,
            date_for_filename,
            camera,
        });
    }

    entries.sort_by(|left, right| {
        left.taken_at
            .cmp(&right.taken_at)
            .then_with(|| left.path.cmp(&right.path))
    });

    Ok(entries)
}

pub fn build_exif_rename_plan(
    paths: &[PathBuf],
    location: Option<&str>,
) -> Result<Vec<RenamePlan>, Box<dyn std::error::Error>> {
    let entries = collect_rename_entries(paths)?;

    let location = location
        .map(sanitize_filename_part)
        .filter(|value| !value.is_empty());

    let mut plans = Vec::with_capacity(entries.len());

    for (index, entry) in entries.into_iter().enumerate() {
        let extension = get_extension(&entry.path)?;
        let sequence = format!("{:03}", index + 1);

        let mut file_stem = format!("{}_{}_{}", entry.date_for_filename, entry.camera, sequence);

        if let Some(location) = &location {
            file_stem.push('_');
            file_stem.push_str(location);
        }

        let target = make_target_path(&entry.path, &file_stem, extension)?;

        plans.push(RenamePlan {
            source: entry.path,
            target,
        });
    }

    validate_rename_plan(&plans)?;

    Ok(plans)
}

pub fn build_custom_rename_plan(
    paths: &[PathBuf],
    custom_name: &str,
) -> Result<Vec<RenamePlan>, Box<dyn std::error::Error>> {
    let custom_name = sanitize_filename_part(custom_name);

    if custom_name.is_empty() {
        return Err(invalid_data_error(
            "任意のファイル名を入力してください".to_string(),
        ));
    }

    let mut sorted_paths = paths.to_vec();
    sorted_paths.sort();

    let mut plans = Vec::with_capacity(sorted_paths.len());

    for (index, source) in sorted_paths.into_iter().enumerate() {
        let extension = get_extension(&source)?;
        let sequence = format!("{:03}", index + 1);
        let file_stem = format!("{custom_name}_{sequence}");

        let target = make_target_path(&source, &file_stem, extension)?;

        plans.push(RenamePlan { source, target });
    }

    validate_rename_plan(&plans)?;

    Ok(plans)
}

pub fn execute_rename_plan(plans: &[RenamePlan]) -> Result<usize, Box<dyn std::error::Error>> {
    validate_rename_plan(plans)?;

    let process_id = std::process::id();
    let mut temporary_paths = Vec::with_capacity(plans.len());

    for (index, plan) in plans.iter().enumerate() {
        let parent = plan.source.parent().ok_or_else(|| {
            invalid_data_error(format!(
                "親フォルダーを取得できませんでした: {}",
                plan.source.display()
            ))
        })?;

        let mut attempt = 0_u32;
        let temporary_path = loop {
            let candidate = parent.join(format!(
                ".image_converter_rename_{process_id}_{index}_{attempt}.tmp"
            ));

            if !candidate.exists() {
                break candidate;
            }

            attempt += 1;
        };

        temporary_paths.push(temporary_path);
    }

    for index in 0..plans.len() {
        if let Err(error) = fs::rename(&plans[index].source, &temporary_paths[index]) {
            for rollback_index in (0..index).rev() {
                let _ = fs::rename(
                    &temporary_paths[rollback_index],
                    &plans[rollback_index].source,
                );
            }

            return Err(Box::new(Error::new(
                error.kind(),
                format!(
                    "一時ファイル名への変更に失敗しました: {} ({error})",
                    plans[index].source.display()
                ),
            )));
        }
    }

    for index in 0..plans.len() {
        if let Err(error) = fs::rename(&temporary_paths[index], &plans[index].target) {
            let mut rollback_errors = Vec::new();

            for rollback_index in (0..index).rev() {
                if let Err(rollback_error) =
                    fs::rename(&plans[rollback_index].target, &plans[rollback_index].source)
                {
                    rollback_errors.push(rollback_error.to_string());
                }
            }

            for rollback_index in index..plans.len() {
                if let Err(rollback_error) = fs::rename(
                    &temporary_paths[rollback_index],
                    &plans[rollback_index].source,
                ) {
                    rollback_errors.push(rollback_error.to_string());
                }
            }

            let rollback_message = if rollback_errors.is_empty() {
                "元のファイル名へ戻しました".to_string()
            } else {
                format!(
                    "元に戻せなかったファイルがあります: {}",
                    rollback_errors.join(" / ")
                )
            };

            return Err(Box::new(Error::new(
                error.kind(),
                format!(
                    "変更後のファイル名への移動に失敗しました: {} ({error})。{rollback_message}",
                    plans[index].target.display()
                ),
            )));
        }
    }

    Ok(plans.len())
}

fn get_extension(path: &Path) -> Result<&str, Box<dyn std::error::Error>> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .ok_or_else(|| {
            invalid_data_error(format!("拡張子を取得できませんでした: {}", path.display()))
        })
}

fn make_target_path(
    source: &Path,
    file_stem: &str,
    extension: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent = source.parent().ok_or_else(|| {
        invalid_data_error(format!(
            "親フォルダーを取得できませんでした: {}",
            source.display()
        ))
    })?;

    let file_name = format!("{file_stem}.{extension}");

    Ok(parent.join(file_name))
}

fn validate_rename_plan(plans: &[RenamePlan]) -> Result<(), Box<dyn std::error::Error>> {
    let source_paths: HashSet<PathBuf> = plans.iter().map(|plan| plan.source.clone()).collect();

    let mut target_names = HashSet::new();

    for plan in plans {
        let target_key = plan.target.to_string_lossy().to_lowercase();

        if !target_names.insert(target_key) {
            return Err(invalid_data_error(format!(
                "変更後のファイル名が重複しています: {}",
                plan.target.display()
            )));
        }

        if plan.target.exists() && !source_paths.contains(&plan.target) {
            return Err(Box::new(Error::new(
                ErrorKind::AlreadyExists,
                format!("同名ファイルがすでに存在します: {}", plan.target.display()),
            )));
        }
    }

    Ok(())
}

fn validate_exif_datetime(value: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = value.as_bytes();

    let valid_length = bytes.len() >= 19;

    let valid_separators = valid_length
        && bytes[4] == b':'
        && bytes[7] == b':'
        && bytes[10] == b' '
        && bytes[13] == b':'
        && bytes[16] == b':';

    let valid_digits = valid_length
        && bytes[..19]
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit());

    if !valid_separators || !valid_digits {
        return Err(invalid_data_error(format!(
            "撮影日時の形式が不正です: {} ({value})",
            path.display()
        )));
    }

    Ok(())
}

pub fn sanitize_filename_part(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_separator = false;

    for character in value.trim().chars() {
        let invalid_character = matches!(
            character,
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
        );

        if invalid_character || character.is_whitespace() {
            if !previous_was_separator && !result.is_empty() {
                result.push('_');
                previous_was_separator = true;
            }
        } else {
            result.push(character);
            previous_was_separator = false;
        }
    }

    result.trim_matches(['_', '.', ' ']).to_string()
}

fn invalid_data_error(message: String) -> Box<dyn std::error::Error> {
    Box::new(Error::new(ErrorKind::InvalidData, message))
}
