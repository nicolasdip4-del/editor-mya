// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Editor MyA: writes the built-in MyA templates through the engine, so
//! every document is one the editor reads back.
//!
//! ```sh
//! cargo run -p concat-project --example mya_templates -- <out> <stickers>
//! ```
//!
//! `<out>` gets one folder per template (`template.json` and `assets/`);
//! `<stickers>` is the concat crate's `stickers/` folder, where the pictures
//! the designs use are copied from. The seller's name and WhatsApp are left
//! as `{{NOMBRE}}` and `{{WHATSAPP}}`, which the editor fills in from
//! Settings when it installs the templates.

use std::path::{Path, PathBuf};

use concat_project::commands::{ClipPatch, NewMedia, TrimEdge};
use concat_project::model::{MediaKind, ShapeKind, ShapeStyle, TextStyle, VideoSettings};
use concat_project::{Command, DocumentSettings, Editor};

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 1920;
const BLUE: &str = "#004F9E";
const BLUE_DARK: &str = "#003B78";
/// The brand face; the editor stands Montserrat in where it is missing.
const BRAND_FONT: &str = "\"Gotham Pro\"";

struct Builder {
    editor: Editor,
    assets: Vec<(String, PathBuf)>,
    stickers: PathBuf,
    slots: u32,
}

impl Builder {
    fn new(stickers: &Path) -> Builder {
        let mut editor = Editor::new();
        let timeline = editor.project().active().id.clone();
        let video = editor.project().active().video;
        apply(
            &mut editor,
            Command::SetTimelineVideo {
                timeline_id: timeline,
                video: VideoSettings {
                    width: WIDTH,
                    height: HEIGHT,
                    rate_num: 30,
                    rate_den: 1,
                    ..video
                },
            },
        );
        Builder {
            editor,
            assets: Vec::new(),
            stickers: stickers.to_path_buf(),
            slots: 0,
        }
    }

    fn track(&mut self) -> String {
        apply(&mut self.editor, Command::AddTrack { bottom: false }).expect("track")
    }

    /// A slot the seller's clip fills: placeholder media `seconds` long.
    fn slot(&mut self, track: &str, start: f64, seconds: f64, name: &str) -> String {
        self.slots += 1;
        let media = apply(
            &mut self.editor,
            Command::AddMedia {
                item: NewMedia {
                    path: format!("slot{}.mp4", self.slots),
                    name: name.to_owned(),
                    duration: Some(seconds),
                    kind: MediaKind::Video,
                    width: Some(WIDTH),
                    height: Some(HEIGHT),
                    frame_rate: Some(30.0),
                    frame_rate_fraction: Some("30/1".to_owned()),
                    video_codec: Some("h264".to_owned()),
                    audio_codec: None,
                    has_audio: false,
                    audio_tracks: Vec::new(),
                    origin: None,
                    color_space: Default::default(),
                },
            },
        )
        .expect("slot media");
        apply(
            &mut self.editor,
            Command::SetMediaPlaceholder {
                media_id: media.clone(),
                placeholder: true,
            },
        );
        self.clip(&media, track, start)
    }

    /// A picture from the sticker shelves, `seconds` long, shrunk to
    /// `scale` of the frame it would fill and moved by `offset_y`.
    fn picture(
        &mut self,
        file: &str,
        track: &str,
        start: f64,
        seconds: f64,
        scale: f64,
        offset_y: f64,
    ) -> String {
        let source = self.stickers.join(file);
        let (width, height) = png_size(&source);
        let bundled = format!(
            "assets/{}",
            Path::new(file).file_name().unwrap().to_string_lossy()
        );
        if !self.assets.iter().any(|(name, _)| *name == bundled) {
            self.assets.push((bundled.clone(), source));
        }
        let media = apply(
            &mut self.editor,
            Command::AddMedia {
                item: NewMedia {
                    path: bundled.clone(),
                    name: Path::new(file)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    duration: None,
                    kind: MediaKind::Image,
                    width: Some(width),
                    height: Some(height),
                    frame_rate: None,
                    frame_rate_fraction: None,
                    video_codec: Some("png".to_owned()),
                    audio_codec: None,
                    has_audio: false,
                    audio_tracks: Vec::new(),
                    origin: None,
                    color_space: Default::default(),
                },
            },
        )
        .or_else(|| {
            self.editor
                .project()
                .media
                .iter()
                .find(|item| item.path == bundled)
                .map(|item| item.id.clone())
        })
        .expect("picture media");
        let clip = self.clip(&media, track, start);
        self.length(&clip, seconds);
        self.place(&clip, scale, offset_y);
        self.fade(&clip, 0.25, 0.0);
        clip
    }

    fn clip(&mut self, media: &str, track: &str, start: f64) -> String {
        apply(
            &mut self.editor,
            Command::AddClip {
                media_id: media.to_owned(),
                track_id: track.to_owned(),
                start,
                ripple: false,
            },
        )
        .expect("clip")
    }

    /// A title: `style` over the default, `seconds` long at `offset_y`.
    fn title(&mut self, track: &str, start: f64, seconds: f64, offset_y: f64, style: TextStyle) {
        let clip = apply(
            &mut self.editor,
            Command::AddTextClip {
                track_id: Some(track.to_owned()),
                above: false,
                start,
                style: Some(style),
                duration: Some(seconds),
                offset_y: Some(offset_y),
            },
        )
        .expect("title");
        self.fade(&clip, 0.25, 0.0);
    }

    /// A full-frame card in the brand blue, behind the closing.
    fn card(&mut self, track: &str, start: f64, seconds: f64) {
        apply(
            &mut self.editor,
            Command::AddShapeClip {
                track_id: Some(track.to_owned()),
                above: false,
                start,
                style: Some(ShapeStyle {
                    size: 1.05,
                    fill: BLUE.to_owned(),
                    ..ShapeStyle::of(ShapeKind::Square)
                }),
                duration: Some(seconds),
                name: "Fondo MyA".to_owned(),
            },
        )
        .expect("card");
    }

    fn length(&mut self, clip: &str, seconds: f64) {
        let now = self
            .editor
            .project()
            .active()
            .clips
            .iter()
            .find(|held| held.id == clip)
            .map(|held| held.duration)
            .expect("clip held");
        apply(
            &mut self.editor,
            Command::TrimClip {
                clip_id: clip.to_owned(),
                edge: TrimEdge::End,
                delta: seconds - now,
                ripple: false,
            },
        );
    }

    fn place(&mut self, clip: &str, scale: f64, offset_y: f64) {
        apply(
            &mut self.editor,
            Command::SetClipTransform {
                clip_id: clip.to_owned(),
                scale: Some(scale),
                offset_x: None,
                offset_y: Some(offset_y),
                rotation: None,
                stretch_x: None,
                stretch_y: None,
            },
        );
    }

    fn fade(&mut self, clip: &str, fade_in: f64, fade_out: f64) {
        apply(
            &mut self.editor,
            Command::UpdateClip {
                clip_id: clip.to_owned(),
                patch: ClipPatch {
                    fade_in: Some(fade_in),
                    fade_out: Some(fade_out),
                    ..ClipPatch::default()
                },
            },
        );
    }

    /// The closing every template ends on: the blue card, the logo, the
    /// seller's name and WhatsApp, and the call to write.
    fn closing(&mut self, start: f64, seconds: f64) {
        let lanes: Vec<String> = (0..5).map(|_| self.track()).collect();
        self.card(&lanes[0], start, seconds);
        self.picture(
            "MyA/01 Logo M&A.png",
            &lanes[1],
            start,
            seconds,
            0.55,
            -0.22,
        );
        self.title(
            &lanes[2],
            start + 0.2,
            seconds - 0.2,
            0.05,
            TextStyle {
                content: "{{NOMBRE}}".to_owned(),
                font_size: 0.05,
                font_weight: 700.0,
                shadow: false,
                font_family: BRAND_FONT.to_owned(),
                ..TextStyle::default()
            },
        );
        self.title(
            &lanes[3],
            start + 0.35,
            seconds - 0.35,
            0.14,
            TextStyle {
                content: "WhatsApp {{WHATSAPP}}".to_owned(),
                font_size: 0.04,
                font_weight: 700.0,
                color: BLUE_DARK.to_owned(),
                background: "#ffffff".to_owned(),
                shadow: false,
                font_family: BRAND_FONT.to_owned(),
                ..TextStyle::default()
            },
        );
        self.picture(
            "MyA/25 Consultá por WhatsApp.png",
            &lanes[4],
            start + 0.5,
            seconds - 0.5,
            0.75,
            0.3,
        );
    }

    /// The lower-third title a seller rewrites: white on the brand blue.
    fn label(content: &str, size: f64) -> TextStyle {
        TextStyle {
            content: content.to_owned(),
            font_size: size,
            font_weight: 700.0,
            background: BLUE.to_owned(),
            shadow: false,
            font_family: BRAND_FONT.to_owned(),
            ..TextStyle::default()
        }
    }

    fn write(self, out: &Path, folder: &str, name: &str) {
        let root = out.join(folder);
        std::fs::create_dir_all(root.join("assets")).expect("template folder");
        for (bundled, source) in &self.assets {
            std::fs::copy(source, root.join(bundled)).expect("asset");
        }
        let settings = DocumentSettings {
            name: name.to_owned(),
            width: WIDTH,
            height: HEIGHT,
            rate_num: 30,
            rate_den: 1,
        };
        let mut document = self.editor.to_document(&settings);
        // A slot carries no file of the creator's: the seller's fills it.
        if let Some(media) = document.get_mut("media").and_then(|m| m.as_array_mut()) {
            for item in media {
                if item.get("placeholder").and_then(|p| p.as_bool()) == Some(true) {
                    item["path"] = serde_json::Value::String(String::new());
                }
            }
        }
        // Read back as the editor will, so a document it cannot open never
        // ships.
        Editor::from_document(&document).expect("the template reads back");
        let encoded = serde_json::to_vec_pretty(&document).expect("encode");
        std::fs::write(root.join("template.json"), encoded).expect("write template");
        println!(
            "{folder}: {} slot(s), {} asset(s)",
            self.slots,
            self.assets.len()
        );
    }
}

fn apply(editor: &mut Editor, command: Command) -> Option<String> {
    editor
        .apply(command)
        .unwrap_or_else(|error| panic!("{error}"))
        .created_id
}

/// Width and height from a PNG's header.
fn png_size(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let read = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (read(16), read(20))
}

fn offer(stickers: &Path, out: &Path) {
    let mut b = Builder::new(stickers);
    let video = b.track();
    b.slot(&video, 0.0, 12.0, "Tu video del producto");
    let top = b.track();
    b.picture("MyA/21 Oferta.png", &top, 0.3, 11.7, 0.42, -0.36);
    let middle = b.track();
    b.title(
        &middle,
        0.6,
        11.4,
        0.22,
        Builder::label("NOMBRE DEL PRODUCTO", 0.045),
    );
    let bottom = b.track();
    b.picture(
        "MyA/04 9 cuotas sin interés azul.png",
        &bottom,
        2.0,
        10.0,
        0.55,
        0.36,
    );
    b.closing(12.0, 3.5);
    b.write(out, "MyA - Producto en oferta", "MyA · Producto en oferta");
}

fn showcase(stickers: &Path, out: &Path) {
    let mut b = Builder::new(stickers);
    let video = b.track();
    for (index, start) in [0.0, 4.0, 8.0].into_iter().enumerate() {
        b.slot(&video, start, 4.0, &format!("Toma {}", index + 1));
    }
    let words = b.track();
    b.title(
        &words,
        0.3,
        3.7,
        -0.34,
        Builder::label("NOMBRE DEL EQUIPO", 0.05),
    );
    for (index, start) in [4.0, 8.0].into_iter().enumerate() {
        b.title(
            &words,
            start + 0.2,
            3.8,
            0.3,
            Builder::label(&format!("Característica {}", index + 1), 0.04),
        );
    }
    let mark = b.track();
    b.picture("MyA/01 Logo M&A.png", &mark, 0.0, 12.0, 0.2, -0.42);
    let cards = b.track();
    b.picture("MyA/11 Tarjetas.png", &cards, 8.5, 3.5, 0.7, 0.42);
    b.closing(12.0, 3.5);
    b.write(
        out,
        "MyA - Presentación de equipo",
        "MyA · Presentación de equipo",
    );
}

fn arrivals(stickers: &Path, out: &Path) {
    let mut b = Builder::new(stickers);
    let video = b.track();
    b.slot(&video, 0.0, 10.0, "Tu video de la mercadería");
    let top = b.track();
    b.picture("MyA/22 Nuevo.png", &top, 0.3, 9.7, 0.4, -0.36);
    let middle = b.track();
    b.picture("MyA/24 Stock disponible.png", &middle, 1.5, 8.5, 0.65, 0.26);
    let bottom = b.track();
    b.picture(
        "MyA/26 Entrega inmediata.png",
        &bottom,
        3.0,
        7.0,
        0.65,
        0.35,
    );
    b.closing(10.0, 3.5);
    b.write(out, "MyA - Llegó mercadería", "MyA · Llegó mercadería");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(out), Some(stickers)) = (args.next(), args.next()) else {
        eprintln!("usage: mya_templates <out> <stickers>");
        std::process::exit(2);
    };
    let (out, stickers) = (PathBuf::from(out), PathBuf::from(stickers));
    offer(&stickers, &out);
    showcase(&stickers, &out);
    arrivals(&stickers, &out);
}
