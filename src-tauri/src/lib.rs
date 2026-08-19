use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};
use std::io::{Cursor, Read, Write};
use watchcompare_render::{sample_reference_frame, FrameState, ReferenceProfile, REFERENCE_FRAME_COUNT};
use watchcompare_scene::sample_reference_scene;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[tauri::command]
fn reference_profile() -> ReferenceProfile {
    ReferenceProfile::default()
}

#[tauri::command]
fn sample_reference(frame: u64) -> FrameState {
    sample_reference_frame(frame)
}

#[tauri::command]
fn reference_scene(frame: u64) -> Value {
    let scene = sample_reference_scene(frame);
    json!({
        "frame_state": sample_reference_frame(scene.frame),
        "first_card_reveal_width_px": scene.first_card_reveal_width_px,
        "credits_left_x_px": scene.credits_left_x_px,
        "second_badge_transform": scene.second_badge.transform,
        "second_badge_text_reveal_level": scene.second_badge.text_reveal_level,
        "second_badge_shine": scene.second_badge.shine,
        "mid_video_cta": scene.mid_video_cta,
        "outro_wipe_bottom_y": scene.outro.wipe_bottom_y,
        "outro_group": scene.outro.group,
        "outro_cta_bbox": scene.cta.outer_bbox,
        "outro_fade_level": scene.outro.fade_level,
        "cta_like_blue_level": scene.cta.like_blue_level,
        "cta_subscribed": scene.cta.subscribed,
        "cta_subscribed_bbox": scene.cta.subscribed_bbox,
        "cta_bell_filled": scene.cta.bell_filled,
        "cta_bell_fill_level": scene.cta.bell_fill_level,
        "cta_cursor_visible": scene.cta.cursor_visible,
        "cta_cursor_x_px": scene.cta.cursor_x_px,
        "cta_cursor_y_px": scene.cta.cursor_y_px
    })
}

#[tauri::command]
fn reference_track() -> Vec<FrameState> {
    (0..REFERENCE_FRAME_COUNT).map(sample_reference_frame).collect()
}

fn normalize_zip_path(value: &str) -> String {
    value.trim_start_matches("./").replace('\\', "/")
}

fn read_zip_bytes(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    requested: &str,
) -> Option<Vec<u8>> {
    let requested = normalize_zip_path(requested);
    let candidates = [
        requested.clone(),
        format!("assets/{requested}"),
        format!("images/{requested}"),
        format!("audio/{requested}"),
    ];
    for candidate in candidates {
        if let Ok(mut file) = archive.by_name(&candidate) {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() {
                return Some(bytes);
            }
        }
    }
    None
}

fn mime_for_path(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".png") { "image/png" }
    else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "image/jpeg" }
    else if lower.ends_with(".webp") { "image/webp" }
    else if lower.ends_with(".gif") { "image/gif" }
    else if lower.ends_with(".mp3") { "audio/mpeg" }
    else if lower.ends_with(".m4a") || lower.ends_with(".mp4") { "audio/mp4" }
    else if lower.ends_with(".wav") { "audio/wav" }
    else if lower.ends_with(".ogg") || lower.ends_with(".oga") { "audio/ogg" }
    else if lower.ends_with(".flac") { "audio/flac" }
    else { "application/octet-stream" }
}

fn data_url(path: &str, bytes: &[u8]) -> String {
    format!("data:{};base64,{}", mime_for_path(path), BASE64.encode(bytes))
}

fn hydrate_project_assets(
    project: &mut Value,
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    if let Some(cards) = project.get_mut("cards").and_then(Value::as_array_mut) {
        for card in cards {
            let path = card.get("artwork").and_then(Value::as_str)
                .or_else(|| card.get("image").and_then(Value::as_str))
                .map(str::to_owned);
            if let Some(path) = path {
                if !path.starts_with("data:") {
                    if let Some(bytes) = read_zip_bytes(archive, &path) {
                        card["artwork"] = Value::String(data_url(&path, &bytes));
                        if card.get("artworkName").is_none() {
                            card["artworkName"] = Value::String(path.rsplit('/').next().unwrap_or(&path).to_owned());
                        }
                    }
                }
            }
        }
    }

    let audio_key = if project.get("audioTracks").is_some() { "audioTracks" } else { "audio_tracks" };
    if let Some(tracks) = project.get_mut(audio_key).and_then(Value::as_array_mut) {
        for track in tracks {
            let path = track.get("data").and_then(Value::as_str)
                .or_else(|| track.get("path").and_then(Value::as_str))
                .map(str::to_owned);
            if let Some(path) = path {
                if !path.starts_with("data:") {
                    if let Some(bytes) = read_zip_bytes(archive, &path) {
                        track["data"] = Value::String(data_url(&path, &bytes));
                        if track.get("name").is_none() {
                            track["name"] = Value::String(path.rsplit('/').next().unwrap_or(&path).to_owned());
                        }
                    }
                }
            }
        }
    }
}

fn header_index(headers: &csv::StringRecord, aliases: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let normalized = header.trim().to_ascii_lowercase().replace([' ', '_', '-', '/'], "");
        aliases.iter().any(|alias| normalized == *alias)
    })
}

fn project_from_csv(
    csv_bytes: &[u8],
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
) -> Result<Value, String> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(csv_bytes);
    let headers = reader.headers().map_err(|e| format!("Could not read megapack CSV headers: {e}"))?.clone();
    let badge = header_index(&headers, &["badge", "badgevalue", "value", "year", "rank", "date"]);
    let badge_label = header_index(&headers, &["badgelabel", "badgesubtitle", "unit", "units", "label"]);
    let title = header_index(&headers, &["title", "name", "heading"]);
    let description = header_index(&headers, &["description", "desc", "details", "summary", "text"]);
    let image = header_index(&headers, &["artwork", "image", "imagepath", "photo", "picture", "thumbnail"]);

    let mut cards = Vec::new();
    for (index, row) in reader.records().enumerate() {
        let row = row.map_err(|e| format!("Could not read megapack CSV row {}: {e}", index + 2))?;
        let value = |slot: Option<usize>| slot.and_then(|i| row.get(i)).unwrap_or("").trim().to_owned();
        let image_path = value(image);
        let artwork = if image_path.is_empty() { String::new() }
            else if let Some(bytes) = read_zip_bytes(archive, &image_path) { data_url(&image_path, &bytes) }
            else { image_path.clone() };
        cards.push(json!({
            "id": format!("megapack-card-{}", index + 1),
            "badge": value(badge),
            "badgeSubtitle": value(badge_label),
            "title": value(title),
            "description": value(description),
            "artwork": if artwork.is_empty() { Value::Null } else { Value::String(artwork) },
            "artworkName": if image_path.is_empty() { Value::Null } else { Value::String(image_path.rsplit('/').next().unwrap_or(&image_path).to_owned()) },
            "accent": "#e00000",
            "background": "#138ddb"
        }));
    }
    Ok(json!({
        "version": 3,
        "name": "Imported megapack",
        "cards": cards,
        "settings": {
            "modelId": "reference_locked",
            "automaticTiming": true,
            "customDuration": null,
            "soundtrackMasterVolume": 1.0
        },
        "audioTracks": []
    }))
}

#[tauri::command]
fn import_megapack(bytes: Vec<u8>) -> Result<Value, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Not a readable megapack ZIP: {e}"))?;
    let mut json_project = None;
    for name in ["project.json", "manifest.json", "watchcompare.json", "cts-project.json"] {
        if let Some(bytes) = read_zip_bytes(&mut archive, name) {
            let parsed: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{name} is not valid JSON: {e}"))?;
            json_project = Some(parsed.get("project").cloned().unwrap_or(parsed));
            break;
        }
    }

    let mut project = if let Some(project) = json_project {
        project
    } else {
        let names: Vec<String> = (0..archive.len()).filter_map(|index| {
            archive.by_index(index).ok().map(|file| file.name().to_owned())
        }).collect();
        let csv_name = names.iter().find(|name| {
            let lower = name.to_ascii_lowercase();
            lower.ends_with("data.csv") || lower.ends_with("cards.csv") || lower == &&"data.csv".to_string()
        }).cloned().or_else(|| names.iter().find(|name| name.to_ascii_lowercase().ends_with(".csv")).cloned())
            .ok_or_else(|| "Megapack needs project.json/manifest.json or a CSV data file.".to_string())?;
        let csv_bytes = read_zip_bytes(&mut archive, &csv_name).ok_or_else(|| "Could not read megapack CSV.".to_string())?;
        project_from_csv(&csv_bytes, &mut archive)?
    };

    hydrate_project_assets(&mut project, &mut archive);
    if project.get("version").is_none() { project["version"] = json!(3); }
    if project.get("settings").is_none() {
        project["settings"] = json!({
            "modelId": "reference_locked",
            "automaticTiming": true,
            "customDuration": null,
            "soundtrackMasterVolume": 1.0
        });
    }
    if project.get("audioTracks").is_none() && project.get("audio_tracks").is_none() {
        project["audioTracks"] = json!([]);
    }
    Ok(project)
}

fn decode_data_url(value: &str) -> Option<(String, Vec<u8>)> {
    let rest = value.strip_prefix("data:")?;
    let (meta, payload) = rest.split_once(',')?;
    if !meta.ends_with(";base64") { return None; }
    let mime = meta.trim_end_matches(";base64").to_owned();
    let bytes = BASE64.decode(payload).ok()?;
    Some((mime, bytes))
}

fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "audio/mpeg" => "mp3",
        "audio/mp4" => "m4a",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        _ => "bin",
    }
}

#[tauri::command]
fn export_megapack(project: Value) -> Result<Vec<u8>, String> {
    let mut packed = project.clone();
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    if let Some(cards) = packed.get_mut("cards").and_then(Value::as_array_mut) {
        for (index, card) in cards.iter_mut().enumerate() {
            if let Some(source) = card.get("artwork").and_then(Value::as_str).map(str::to_owned) {
                if let Some((mime, bytes)) = decode_data_url(&source) {
                    let path = format!("assets/card_{:04}.{}", index + 1, extension_for_mime(&mime));
                    writer.start_file(&path, options).map_err(|e| e.to_string())?;
                    writer.write_all(&bytes).map_err(|e| e.to_string())?;
                    card["artwork"] = Value::String(path);
                }
            }
        }
    }

    if let Some(tracks) = packed.get_mut("audioTracks").and_then(Value::as_array_mut) {
        for (index, track) in tracks.iter_mut().enumerate() {
            if let Some(source) = track.get("data").and_then(Value::as_str).map(str::to_owned) {
                if let Some((mime, bytes)) = decode_data_url(&source) {
                    let path = format!("audio/track_{:02}.{}", index + 1, extension_for_mime(&mime));
                    writer.start_file(&path, options).map_err(|e| e.to_string())?;
                    writer.write_all(&bytes).map_err(|e| e.to_string())?;
                    track["data"] = Value::String(path.clone());
                    track["path"] = Value::String(path);
                }
            }
        }
    }

    writer.start_file("project.json", options).map_err(|e| e.to_string())?;
    let json_bytes = serde_json::to_vec_pretty(&packed).map_err(|e| e.to_string())?;
    writer.write_all(&json_bytes).map_err(|e| e.to_string())?;

    let cursor = writer.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            reference_profile,
            sample_reference,
            reference_scene,
            reference_track,
            import_megapack,
            export_megapack
        ])
        .run(tauri::generate_context!())
        .expect("error while running WatchCompare");
}
