// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Picture stickers: images the Stickers page offers beside the shapes,
//! one shelf per folder.
//!
//! They live as plain files under `stickers/` in the app's data directory,
//! one folder per shelf: `stickers/MyA/21 Oferta.png`. The built-in ones are
//! embedded in the binary and written there on start when missing, so a
//! project that uses one points at a real file like any other import, and
//! anyone can add a shelf of their own by dropping a folder of PNGs beside
//! them. A file's name is its card's label; a leading number ("21 ") only
//! orders the shelf and is not shown.

use std::path::{Path, PathBuf};

use concat_host::AppDirs;
use concat_project::model::Timeline;

/// The built-in shelves, as `(shelf/file, bytes)`.
const BUILT_IN: &[(&str, &[u8])] = &[
    ("MyA/01 Logo M&A.png", include_bytes!("../stickers/MyA/01 Logo M&A.png")),
    ("MyA/02 Súper Crédito.png", include_bytes!("../stickers/MyA/02 Súper Crédito.png")),
    ("MyA/03 Hasta 9 cuotas sin interés.png", include_bytes!("../stickers/MyA/03 Hasta 9 cuotas sin interés.png")),
    ("MyA/04 9 cuotas sin interés azul.png", include_bytes!("../stickers/MyA/04 9 cuotas sin interés azul.png")),
    ("MyA/05 9 cuotas sin interés.png", include_bytes!("../stickers/MyA/05 9 cuotas sin interés.png")),
    ("MyA/06 Aprovechá 9 cuotas.png", include_bytes!("../stickers/MyA/06 Aprovechá 9 cuotas.png")),
    ("MyA/07 Sin interés.png", include_bytes!("../stickers/MyA/07 Sin interés.png")),
    ("MyA/08 Envíos a todo el país.png", include_bytes!("../stickers/MyA/08 Envíos a todo el país.png")),
    ("MyA/09 Atención.png", include_bytes!("../stickers/MyA/09 Atención.png")),
    ("MyA/10 Líder en equipamiento comercial.png", include_bytes!("../stickers/MyA/10 Líder en equipamiento comercial.png")),
    ("MyA/11 Tarjetas.png", include_bytes!("../stickers/MyA/11 Tarjetas.png")),
    ("MyA/12 Tarjetas 2.png", include_bytes!("../stickers/MyA/12 Tarjetas 2.png")),
    ("MyA/13 Naranja X.png", include_bytes!("../stickers/MyA/13 Naranja X.png")),
    ("MyA/14 Visa.png", include_bytes!("../stickers/MyA/14 Visa.png")),
    ("MyA/15 Mastercard.png", include_bytes!("../stickers/MyA/15 Mastercard.png")),
    ("MyA/16 WhatsApp.png", include_bytes!("../stickers/MyA/16 WhatsApp.png")),
    ("MyA/17 Instagram.png", include_bytes!("../stickers/MyA/17 Instagram.png")),
    ("MyA/18 Facebook.png", include_bytes!("../stickers/MyA/18 Facebook.png")),
    ("MyA/19 TikTok.png", include_bytes!("../stickers/MyA/19 TikTok.png")),
    ("MyA/20 YouTube.png", include_bytes!("../stickers/MyA/20 YouTube.png")),
    ("MyA/21 Oferta.png", include_bytes!("../stickers/MyA/21 Oferta.png")),
    ("MyA/22 Nuevo.png", include_bytes!("../stickers/MyA/22 Nuevo.png")),
    ("MyA/23 Últimas unidades.png", include_bytes!("../stickers/MyA/23 Últimas unidades.png")),
    ("MyA/24 Stock disponible.png", include_bytes!("../stickers/MyA/24 Stock disponible.png")),
    ("MyA/25 Consultá por WhatsApp.png", include_bytes!("../stickers/MyA/25 Consultá por WhatsApp.png")),
    ("MyA/26 Entrega inmediata.png", include_bytes!("../stickers/MyA/26 Entrega inmediata.png")),
    ("Marcas/Depaolo.png", include_bytes!("../stickers/Marcas/Depaolo.png")),
    ("Marcas/Frider.png", include_bytes!("../stickers/Marcas/Frider.png")),
    ("Marcas/INR.png", include_bytes!("../stickers/Marcas/INR.png")),
    ("Marcas/Inelro.png", include_bytes!("../stickers/Marcas/Inelro.png")),
    ("Marcas/Morelli.png", include_bytes!("../stickers/Marcas/Morelli.png")),
    ("Marcas/Sol Real.png", include_bytes!("../stickers/Marcas/Sol Real.png")),
    ("Marcas/Teora.png", include_bytes!("../stickers/Marcas/Teora.png")),
    ("Marcas/Usman.png", include_bytes!("../stickers/Marcas/Usman.png")),
];

/// The shelves the built-in stickers come in, first in this order; a
/// user's own folders follow by name.
const BUILT_IN_SHELVES: [&str; 2] = ["MyA", "Marcas"];

/// How big a sticker lands, as a fraction of the picture it would fill.
pub const PLACED_SCALE: f64 = 0.4;

/// How long a still lasts on the timeline when placed (see the project's
/// `clip_length`), which is the span a lane must have free to take it.
pub const STILL_SECONDS: f64 = 5.0;

/// One card on the Stickers page.
#[derive(Clone, Debug, PartialEq)]
pub struct Sticker {
    /// The shelf: the folder it sits in.
    pub shelf: String,
    /// The name under its card.
    pub label: String,
    /// The file, which is what a placed sticker imports.
    pub path: PathBuf,
}

/// The folder every shelf sits in.
pub fn root(dirs: &AppDirs) -> PathBuf {
    dirs.data.join("stickers")
}

/// Every sticker, built-in shelves first: writes the built-in files when
/// missing, then reads the folders. A folder that cannot be read is
/// skipped; stickers are never worth failing a start over.
pub fn all(dirs: &AppDirs) -> Vec<Sticker> {
    let root = root(dirs);
    install(&root);
    let mut shelves: Vec<String> = std::fs::read_dir(&root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    shelves.sort_by_key(|shelf| {
        let built_in = BUILT_IN_SHELVES.iter().position(|known| known == shelf);
        (built_in.unwrap_or(usize::MAX), shelf.to_lowercase())
    });
    shelves
        .iter()
        .flat_map(|shelf| read_shelf(&root.join(shelf), shelf))
        .collect()
}

/// The shelves' names, in the order [`all`] gives their stickers.
pub fn shelves(stickers: &[Sticker]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for sticker in stickers {
        if names.last() != Some(&sticker.shelf) {
            names.push(sticker.shelf.clone());
        }
    }
    names
}

/// The lane a sticker placed at `start` goes on: the first one above the
/// highest lane occupied over its span, so it sits over the picture.
/// `None` when every lane above is taken - the caller adds one on top.
pub fn lane_for(timeline: &Timeline, start: f64) -> Option<String> {
    let end = start + STILL_SECONDS;
    let occupied = |track_id: &str| {
        timeline.clips.iter().any(|clip| {
            clip.track_id == track_id && clip.start < end && start < clip.start + clip.duration
        })
    };
    let floor = timeline
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| occupied(&track.id))
        .map(|(row, _)| row + 1)
        .max()
        .unwrap_or(0);
    timeline
        .tracks
        .iter()
        .skip(floor)
        .find(|track| !occupied(&track.id))
        .map(|track| track.id.clone())
}

fn install(root: &Path) {
    for (name, bytes) in BUILT_IN {
        let path = root.join(name);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            log::warn!("stickers: could not create {}: {error}", parent.display());
            return;
        }
        if let Err(error) = std::fs::write(&path, bytes) {
            log::warn!("stickers: could not write {}: {error}", path.display());
        }
    }
}

fn read_shelf(folder: &Path, shelf: &str) -> Vec<Sticker> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_picture(path))
        .collect();
    files.sort_by_key(|path| path.file_name().map(|name| name.to_string_lossy().to_lowercase()));
    files
        .into_iter()
        .map(|path| Sticker {
            shelf: shelf.to_owned(),
            label: label_of(&path),
            path,
        })
        .collect()
}

fn is_picture(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "png" | "webp" | "jpg" | "jpeg"))
}

/// "21 Oferta.png" reads "Oferta": the number only orders the shelf.
fn label_of(path: &Path) -> String {
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let trimmed = stem.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start();
    if trimmed.is_empty() || trimmed.len() == stem.len() {
        stem.replace(['_', '-'], " ")
    } else {
        trimmed.replace(['_', '-'], " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leading_number_orders_and_is_not_shown() {
        assert_eq!(label_of(Path::new("MyA/21 Oferta.png")), "Oferta");
        assert_eq!(
            label_of(Path::new("MyA/04 9 cuotas sin interés azul.png")),
            "9 cuotas sin interés azul"
        );
        assert_eq!(label_of(Path::new("Marcas/Sol Real.png")), "Sol Real");
        assert_eq!(label_of(Path::new("Marcas/stock_disponible.png")), "stock disponible");
    }

    #[test]
    fn built_in_stickers_install_and_read_in_shelf_order() {
        let dir = std::env::temp_dir().join(format!("concat-stickers-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let dirs = AppDirs::under(&dir);
        std::fs::create_dir_all(root(&dirs).join("Mios")).unwrap();
        std::fs::write(root(&dirs).join("Mios/Promo.png"), BUILT_IN[0].1).unwrap();

        let stickers = all(&dirs);
        assert_eq!(stickers.len(), BUILT_IN.len() + 1);
        assert_eq!(shelves(&stickers), ["MyA", "Marcas", "Mios"]);
        assert_eq!(stickers[0].label, "Logo M&A");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
