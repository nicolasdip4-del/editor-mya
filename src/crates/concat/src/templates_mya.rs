// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Editor MyA: the built-in MyA templates, written into the templates
//! folder with the seller's own name and WhatsApp on their closing card.
//!
//! The bundles are made by `concat-project`'s `mya_templates` example and
//! kept under `templates-mya/`; their closing titles say `{{NOMBRE}}` and
//! `{{WHATSAPP}}`. [`install`] writes them on every start and whenever those
//! two settings change, so a template always closes on the seller's data.

use std::path::Path;

use concat_host::AppDirs;
use concat_host::templates::templates_dir;

/// Every file of every built-in template, as `(folder/file, bytes)`.
const FILES: &[(&str, &[u8])] = &[
    (
        "MyA - Llegó mercadería/assets/01 Logo M&A.png",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/assets/01 Logo M&A.png"),
    ),
    (
        "MyA - Llegó mercadería/assets/22 Nuevo.png",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/assets/22 Nuevo.png"),
    ),
    (
        "MyA - Llegó mercadería/assets/24 Stock disponible.png",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/assets/24 Stock disponible.png"),
    ),
    (
        "MyA - Llegó mercadería/assets/25 Consultá por WhatsApp.png",
        include_bytes!(
            "../templates-mya/MyA - Llegó mercadería/assets/25 Consultá por WhatsApp.png"
        ),
    ),
    (
        "MyA - Llegó mercadería/assets/26 Entrega inmediata.png",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/assets/26 Entrega inmediata.png"),
    ),
    (
        "MyA - Llegó mercadería/poster.jpg",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/poster.jpg"),
    ),
    (
        "MyA - Llegó mercadería/template.json",
        include_bytes!("../templates-mya/MyA - Llegó mercadería/template.json"),
    ),
    (
        "MyA - Presentación de equipo/assets/01 Logo M&A.png",
        include_bytes!("../templates-mya/MyA - Presentación de equipo/assets/01 Logo M&A.png"),
    ),
    (
        "MyA - Presentación de equipo/assets/11 Tarjetas.png",
        include_bytes!("../templates-mya/MyA - Presentación de equipo/assets/11 Tarjetas.png"),
    ),
    (
        "MyA - Presentación de equipo/assets/25 Consultá por WhatsApp.png",
        include_bytes!(
            "../templates-mya/MyA - Presentación de equipo/assets/25 Consultá por WhatsApp.png"
        ),
    ),
    (
        "MyA - Presentación de equipo/poster.jpg",
        include_bytes!("../templates-mya/MyA - Presentación de equipo/poster.jpg"),
    ),
    (
        "MyA - Presentación de equipo/template.json",
        include_bytes!("../templates-mya/MyA - Presentación de equipo/template.json"),
    ),
    (
        "MyA - Producto en oferta/assets/01 Logo M&A.png",
        include_bytes!("../templates-mya/MyA - Producto en oferta/assets/01 Logo M&A.png"),
    ),
    (
        "MyA - Producto en oferta/assets/04 9 cuotas sin interés azul.png",
        include_bytes!(
            "../templates-mya/MyA - Producto en oferta/assets/04 9 cuotas sin interés azul.png"
        ),
    ),
    (
        "MyA - Producto en oferta/assets/21 Oferta.png",
        include_bytes!("../templates-mya/MyA - Producto en oferta/assets/21 Oferta.png"),
    ),
    (
        "MyA - Producto en oferta/assets/25 Consultá por WhatsApp.png",
        include_bytes!(
            "../templates-mya/MyA - Producto en oferta/assets/25 Consultá por WhatsApp.png"
        ),
    ),
    (
        "MyA - Producto en oferta/poster.jpg",
        include_bytes!("../templates-mya/MyA - Producto en oferta/poster.jpg"),
    ),
    (
        "MyA - Producto en oferta/template.json",
        include_bytes!("../templates-mya/MyA - Producto en oferta/template.json"),
    ),
    (
        "MyA - Video rápido/assets/01 Logo M&A.png",
        include_bytes!("../templates-mya/MyA - Video rápido/assets/01 Logo M&A.png"),
    ),
    (
        "MyA - Video rápido/assets/25 Consultá por WhatsApp.png",
        include_bytes!("../templates-mya/MyA - Video rápido/assets/25 Consultá por WhatsApp.png"),
    ),
    (
        "MyA - Video rápido/poster.jpg",
        include_bytes!("../templates-mya/MyA - Video rápido/poster.jpg"),
    ),
    (
        "MyA - Video rápido/template.json",
        include_bytes!("../templates-mya/MyA - Video rápido/template.json"),
    ),
];

/// What a closing card says while Settings has no name or number yet: a
/// prompt the seller sees in the preview and knows to fill in.
const NO_NAME: &str = "TU NOMBRE";
const NO_WHATSAPP: &str = "TU WHATSAPP";

/// Writes every built-in template, its closing card naming `name` and
/// `whatsapp`. A file that cannot be written is logged and skipped:
/// templates are never worth failing a start over.
pub fn install(dirs: &AppDirs, name: &str, whatsapp: &str) {
    let root = templates_dir(&dirs.config);
    let name = filled(name, NO_NAME);
    let whatsapp = filled(whatsapp, NO_WHATSAPP);
    for (file, bytes) in FILES {
        let path = root.join(file);
        let written = if file.ends_with("template.json") {
            personalised(bytes, &name, &whatsapp)
                .map_or(Ok(()), |text| write(&path, text.as_bytes()))
        } else {
            write(&path, bytes)
        };
        if let Err(error) = written {
            log::warn!("templates: could not write {}: {error}", path.display());
        }
    }
}

fn filled(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

/// The template's document with the two placeholders written in, escaped
/// for the JSON string they land inside.
fn personalised(bytes: &[u8], name: &str, whatsapp: &str) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    Some(
        text.replace("{{NOMBRE}}", &escaped(name))
            .replace("{{WHATSAPP}}", &escaped(whatsapp)),
    )
}

fn escaped(value: &str) -> String {
    let quoted = serde_json::to_string(value).unwrap_or_default();
    quoted
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or_default()
        .to_owned()
}

fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::read(path).is_ok_and(|held| held == bytes) {
        return Ok(());
    }
    std::fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_install_with_the_sellers_data() {
        let dir = std::env::temp_dir().join(format!("concat-mya-templates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let dirs = AppDirs::under(&dir);

        install(&dirs, "Ana \"La\" Pérez", "381 555-0000");
        let listed = concat_host::templates::list(&dirs.config);
        assert_eq!(listed.len(), 4);
        let offer = std::fs::read_to_string(
            templates_dir(&dirs.config).join("MyA - Producto en oferta/template.json"),
        )
        .unwrap();
        assert!(offer.contains("Ana \\\"La\\\" Pérez"));
        assert!(offer.contains("WhatsApp 381 555-0000"));
        assert!(!offer.contains("{{"));

        install(&dirs, "", " ");
        let offer = std::fs::read_to_string(
            templates_dir(&dirs.config).join("MyA - Producto en oferta/template.json"),
        )
        .unwrap();
        assert!(offer.contains(NO_NAME) && offer.contains(NO_WHATSAPP));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
