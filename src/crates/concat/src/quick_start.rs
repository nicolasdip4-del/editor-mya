// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Editor MyA: the start screen's quick ways in.
//!
//! A seller starts from a MyA design (a template's card) or from "Video
//! rápido" (the quick template): either way the editor asks for the videos,
//! makes a project from the template with them in its slots, and opens it.
//! A design with one slot is fitted to the seller's video - the slot, the
//! stickers and titles over it, and the closing after it all move with the
//! video's own length - so a 30 second video is not cut to the design's 12.

use std::path::{Path, PathBuf};

use concat_host::templates::{self, SlotFill, TemplateInfo};
use concat_host::{AppDirs, projects};
use serde_json::Value;

use crate::i18n::{t, tf};
use crate::studio::Studio;

/// The quick template's name: "Video rápido" on the start screen, not a
/// card among the designs.
const QUICK: &str = "MyA · Video rápido";
/// What every MyA template's name starts with.
const PREFIX: &str = "MyA · ";
/// The shortest and longest a fitted slot becomes: a shorter video holds
/// its last frame, a longer one is cut, as the engine does past a slot.
const SHORTEST: f64 = 4.0;
const LONGEST: f64 = 90.0;
/// What the pickers offer.
const VIDEOS: [&str; 6] = ["mp4", "mov", "m4v", "mkv", "webm", "avi"];

/// The MyA designs the start screen shows as cards, by name.
pub fn designs(dirs: &AppDirs) -> Vec<TemplateInfo> {
    templates::list(&dirs.config)
        .into_iter()
        .filter(|info| info.name.starts_with(PREFIX) && info.name != QUICK)
        .collect()
}

/// A design's card: the template, by its folder.
pub fn start_design(path: String) {
    choose_videos(path);
}

/// "Video rápido": the quick template, when it is installed.
pub fn start_quick() {
    crate::host::on_ui(|studio, _, _| {
        let quick = templates::list(&studio.host.dirs.config)
            .into_iter()
            .find(|info| info.name == QUICK);
        match quick {
            Some(info) => choose_videos(info.path),
            None => studio.notify(&t("start.quickMissing"), true),
        }
    });
}

fn choose_videos(template: String) {
    crate::host::pick_files_then(
        t("start.chooseVideos"),
        Some((t("start.videos"), &VIDEOS)),
        move |studio, _, _, paths| begin(studio, template, paths),
    );
}

/// Probes the videos off the window's thread, then makes the project.
fn begin(studio: &mut Studio, template: String, paths: Vec<PathBuf>) {
    if paths.is_empty() {
        return;
    }
    let Some(info) = templates::list(&studio.host.dirs.config)
        .into_iter()
        .find(|info| info.path == template)
    else {
        studio.notify(&t("start.quickMissing"), true);
        return;
    };
    studio.start.busy = true;
    studio.notify(&t("start.makingVideo"), false);
    let location = crate::panes::start::default_location();
    crate::host::spawn(
        move || make(&info, &location, &paths),
        |studio, _, _, made: Result<projects::ProjectInfo, String>| {
            studio.start.busy = false;
            if let Err(error) = made.and_then(|info| studio.open_project(info)) {
                log::warn!("quick start: {error}");
                studio.notify(&tf("start.couldNotMakeVideo", &[&error]), true);
            }
        },
    );
}

/// The project: every slot filled in turn from `videos` (the first video
/// again when there are more slots than videos), named after the design
/// and the moment, and fitted when the design has one slot.
fn make(
    info: &TemplateInfo,
    location: &str,
    videos: &[PathBuf],
) -> Result<projects::ProjectInfo, String> {
    let mut probed = Vec::new();
    for video in videos {
        let summary = concat_host::media::probe(&video.to_string_lossy())?;
        probed.push(summary.to_new_media());
    }
    let fills: Vec<SlotFill> = info
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| SlotFill {
            media_id: slot.media_id.clone(),
            item: probed[index % probed.len()].clone(),
        })
        .collect();
    let name = project_name(&info.name);
    let project = templates::instantiate(&info.path, location, &name, fills)?;
    if let ([slot], Some(seconds)) = (info.slots.as_slice(), probed[0].duration) {
        fit(Path::new(&project.path), &slot.media_id, seconds)?;
        return projects::open(&project.path);
    }
    Ok(project)
}

/// "Producto en oferta 16-10 18.40": the design's own words and when.
fn project_name(template: &str) -> String {
    let design = template.strip_prefix(PREFIX).unwrap_or(template);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    let (day, month, hour, minute) = local_parts(now);
    format!("{design} {day:02}-{month:02} {hour:02}.{minute:02}")
}

/// Day, month, hour and minute of `seconds` since the epoch, in
/// Argentina's time (UTC-3, no daylight saving), where every seller is.
fn local_parts(seconds: u64) -> (u64, u64, u64, u64) {
    let local = seconds.saturating_sub(3 * 3600);
    let days = (local / 86_400) as i64;
    let (hour, minute) = ((local % 86_400) / 3600, (local % 3600) / 60);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u64;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u64;
    (day, month, hour, minute)
}

/// Stretches the one slot of a fresh project to `seconds` (held between
/// [`SHORTEST`] and [`LONGEST`]): what spans to the slot's end stretches
/// with it, and what comes after it moves by the same amount.
fn fit(project: &Path, slot_media: &str, seconds: f64) -> Result<(), String> {
    let manifest = project.join("concat.json");
    let bytes = std::fs::read(&manifest).map_err(|error| error.to_string())?;
    let mut document: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    fit_document(&mut document, slot_media, seconds.clamp(SHORTEST, LONGEST));
    let encoded = serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?;
    std::fs::write(&manifest, encoded).map_err(|error| error.to_string())
}

fn fit_document(document: &mut Value, slot_media: &str, seconds: f64) {
    if let Some(timelines) = document.get_mut("timelines").and_then(Value::as_array_mut) {
        for timeline in timelines {
            if let Some(clips) = timeline.get_mut("clips").and_then(Value::as_array_mut) {
                fit_clips(clips, slot_media, seconds);
            }
        }
    }
    if let Some(clips) = document.get_mut("clips").and_then(Value::as_array_mut) {
        fit_clips(clips, slot_media, seconds);
    }
}

fn fit_clips(clips: &mut [Value], slot_media: &str, seconds: f64) {
    const EDGE: f64 = 0.05;
    let number = |clip: &Value, key: &str| clip.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    let is_slot = |clip: &Value| clip.get("mediaId").and_then(Value::as_str) == Some(slot_media);
    let Some(slot) = clips.iter().find(|clip| is_slot(clip)) else {
        return;
    };
    let start = number(slot, "start");
    let end = start + number(slot, "duration");
    let delta = start + seconds - end;
    if delta.abs() < EDGE {
        return;
    }
    for clip in clips.iter_mut() {
        let (clip_start, duration) = (number(clip, "start"), number(clip, "duration"));
        if is_slot(clip) || (clip_start < end - EDGE && clip_start + duration >= end - EDGE) {
            clip["duration"] = Value::from((duration + delta).max(0.3));
        } else if clip_start >= end - EDGE {
            clip["start"] = Value::from(clip_start + delta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_one_slot_design_follows_the_video() {
        let mut document = serde_json::json!({
            "timelines": [{"clips": [
                {"id": "v", "mediaId": "m1", "start": 0.0, "duration": 12.0},
                {"id": "sticker", "mediaId": "m2", "start": 0.3, "duration": 11.7},
                {"id": "late", "mediaId": "m3", "start": 2.0, "duration": 4.0},
                {"id": "card", "start": 12.0, "duration": 3.5},
            ]}],
            "clips": [{"id": "v", "mediaId": "m1", "start": 0.0, "duration": 12.0}],
        });
        fit_document(&mut document, "m1", 30.0);
        let clips = &document["timelines"][0]["clips"];
        assert_eq!(clips[0]["duration"], 30.0);
        assert!((clips[1]["duration"].as_f64().unwrap() - 29.7).abs() < 1e-9);
        assert_eq!(clips[2]["duration"], 4.0);
        assert_eq!(clips[3]["start"], 30.0);
        assert_eq!(document["clips"][0]["duration"], 30.0);
    }

    #[test]
    fn dates_read_in_argentinas_time() {
        // 2026-10-15 21:40 UTC is 18:40 in Argentina.
        assert_eq!(local_parts(1_792_100_400), (15, 10, 18, 40));
    }
}
