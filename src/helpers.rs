use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use image::imageops::blur;
use image::{DynamicImage, EncodableLayout, GenericImage, ImageBuffer, Rgba};
use rusttype::{point, Font, PositionedGlyph, Scale, VMetrics};
use speedy2d::color::Color;
use speedy2d::dimen::Vector2;
use speedy2d::image::{ImageDataType, ImageHandle, ImageSmoothingMode};
use speedy2d::shape::Rectangle;
use speedy2d::window::{KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};

pub(crate) const SCREEN_WIDTH: u32 = 810;
pub(crate) const SCREEN_HEIGHT: u32 = 1440;
pub(crate) const FONT_RATIO: f32 = 427.0 / 1000.0;

const RUST_IMAGE_SIZE: f32 = 405.0;
pub(crate) const TEXT_MARGIN: u32 = 30;

#[derive(Debug)]
pub(crate) struct GlyphSize {
    height: u32,
    width: u32,
}

type GlyphsCache = HashMap<(String, u32), (VMetrics, GlyphSize, Vec<PositionedGlyph<'static>>)>;
type RawImageCache = HashMap<(String, u32, TextType), ImageBuffer<Rgba<u8>, Vec<u8>>>;

pub(crate) struct TextManager {
    pub(crate) font: Font<'static>,
    pub(crate) glyphs: GlyphsCache,
    pub(crate) images: HashMap<(String, u32, TextType), ImageHandle>,
    pub(crate) raw_images: RawImageCache,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GlowColor {
    White,
    Gold,
    Red,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TextType {
    Glow(GlowColor),
    Gray,
}

impl TextManager {
    pub(crate) fn get_glyph_info<'a>(
        glyphs_cache: &'a mut GlyphsCache,
        font: &Font<'static>,
        text: String,
        size: u32,
    ) -> &'a mut (VMetrics, GlyphSize, Vec<PositionedGlyph<'static>>) {
        glyphs_cache.entry((text.clone(), size)).or_insert_with(|| {
            let metrics = font.v_metrics(Scale::uniform(size as f32));
            let glyphs: Vec<_> = font
                .layout(
                    &text,
                    Scale::uniform(size as f32),
                    point(TEXT_MARGIN as f32, TEXT_MARGIN as f32 + metrics.ascent),
                )
                .collect();

            let glyphs_height = (metrics.ascent - metrics.descent).ceil() as u32;
            let glyphs_width = {
                let min_x = glyphs
                    .first()
                    .map(|g| g.pixel_bounding_box().unwrap().min.x)
                    .unwrap();
                let max_x = glyphs
                    .last()
                    .map(|g| g.pixel_bounding_box().unwrap().max.x)
                    .unwrap();
                (max_x - min_x) as u32
            };

            (
                metrics,
                GlyphSize {
                    height: glyphs_height,
                    width: glyphs_width,
                },
                glyphs,
            )
        })
    }

    pub(crate) fn get_raw_image<'a>(
        raw_image_cache: &'a mut RawImageCache,
        glyphs_cache: &mut GlyphsCache,
        font: &Font<'static>,
        text: String,
        size: u32,
        text_type: TextType,
    ) -> &'a ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (_metrics, glyph_size, glyphs) =
            Self::get_glyph_info(glyphs_cache, font, text.clone(), size);

        raw_image_cache
            .entry((text, size, text_type))
            .or_insert_with(|| {
                let mut image = DynamicImage::new_rgba8(
                    glyph_size.width + TEXT_MARGIN * 2,
                    glyph_size.height + TEXT_MARGIN * 2,
                )
                .to_rgba8();
                // Loop through the glyphs in the text, positing each one on a line
                for glyph in glyphs.iter() {
                    if let Some(bounding_box) = glyph.pixel_bounding_box() {
                        // Draw the glyph into the image per-pixel by using the draw closure
                        glyph.draw(|x, y, v| {
                            image.put_pixel(
                                // Offset the position by the glyph bounding box
                                x + bounding_box.min.x as u32,
                                y + bounding_box.min.y as u32,
                                // Turn the coverage into an alpha value
                                match text_type {
                                    TextType::Gray => Rgba([204, 204, 204, (v * 255.0) as u8]),
                                    TextType::Glow(GlowColor::White) => {
                                        Rgba([255, 255, 255, (v * 255.0) as u8])
                                    }
                                    TextType::Glow(GlowColor::Gold) => {
                                        Rgba([255, 255, 102, (v * 255.0) as u8])
                                    }
                                    TextType::Glow(GlowColor::Red) => {
                                        Rgba([239, 68, 68, (v * 255.0) as u8])
                                    }
                                },
                            )
                        });
                    }
                }

                match text_type {
                    TextType::Gray => image,
                    TextType::Glow(color) => {
                        let mut image_blur = blur(&image, 8.0);

                        for glyph in glyphs {
                            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                                // Draw the glyph into the image per-pixel by using the draw closure
                                glyph.draw(|x, y, v| {
                                    let pixel = image_blur.get_pixel_mut(
                                        x + bounding_box.min.x as u32,
                                        y + bounding_box.min.y as u32,
                                    );

                                    let a_a = pixel.0[3] as f32;
                                    let a_b = v * 255.0;

                                    let r_a = pixel.0[0] as f32;
                                    let g_a = pixel.0[1] as f32;
                                    let b_a = pixel.0[2] as f32;

                                    let r_b = match color {
                                        GlowColor::Red => 239.0,
                                        _ => 255.0,
                                    };
                                    let g_b = match color {
                                        GlowColor::Red => 68.0,
                                        _ => 255.0,
                                    };
                                    let b_b = match color {
                                        GlowColor::White => 255.0,
                                        GlowColor::Gold => 102.0,
                                        GlowColor::Red => 68.0,
                                    };

                                    let a_out = a_a + (a_b * (255.0 - a_a) / 255.0);
                                    let r_out =
                                        (r_a * a_a + r_b * a_b * (255.0 - a_a) / 255.0) / a_out;
                                    let g_out =
                                        (g_a * a_a + g_b * a_b * (255.0 - a_a) / 255.0) / a_out;
                                    let b_out =
                                        (b_a * a_a + b_b * a_b * (255.0 - a_a) / 255.0) / a_out;

                                    pixel.0[0] = r_out as u8;
                                    pixel.0[1] = g_out as u8;
                                    pixel.0[2] = b_out as u8;
                                    pixel.0[3] = a_out as u8;
                                });
                            }
                        }

                        image_blur
                    }
                }
            })
    }

    pub(crate) fn draw_text(
        &mut self,
        graphics: &mut Graphics2D,
        size: u32,
        text_type: TextType,
        position: (f32, f32),
        text: String,
    ) {
        self.draw_text_align(graphics, size, text_type, position, text, Align::Center)
    }

    pub(crate) fn draw_text_align(
        &mut self,
        graphics: &mut Graphics2D,
        size: u32,
        text_type: TextType,
        position: (f32, f32),
        text: String,
        align: Align,
    ) {
        if text.is_empty() {
            return;
        }

        let image = Self::get_raw_image(
            &mut self.raw_images,
            &mut self.glyphs,
            &self.font,
            text.clone(),
            size,
            text_type,
        );

        let image_handle = self
            .images
            .entry((text.clone(), size, text_type))
            .or_insert_with(|| {
                let image_handle = graphics
                    .create_image_from_raw_pixels(
                        ImageDataType::RGBA,
                        ImageSmoothingMode::NearestNeighbor,
                        image.dimensions(),
                        image.as_bytes(),
                    )
                    .unwrap();

                image_handle
            });

        graphics.draw_image(
            (
                match align {
                    Align::Right => position.0 - image_handle.size().x as f32,
                    Align::Center => position.0 - image_handle.size().x as f32 / 2.0,
                    Align::Left => position.0,
                },
                position.1 - image_handle.size().y as f32 / 2.0,
            ),
            image_handle,
        );
    }
}

pub(crate) enum Align {
    Center,
    Left,
    Right,
}

pub(crate) fn ease_in_cube_ease_out_quad(x: f32) -> f32 {
    x * x
}

pub(crate) struct MyWindowHandler<S: State> {
    state: S,
    prog_name: (String, String),
    text_manager: TextManager,
    frame: u64,
    timings: Timings,
    pause: bool,
    splashscreen: bool,
    on_start_called: bool,
}

pub(crate) struct Timings {
    pub(crate) start: Instant,
    pub(crate) last_frame: Instant,
}

impl<S: State> MyWindowHandler<S> {}

impl<S: State> WindowHandler for MyWindowHandler<S> {
    fn on_key_up(
        &mut self,
        _helper: &mut WindowHelper<()>,
        virtual_key_code: Option<VirtualKeyCode>,
        _scancode: KeyScancode,
    ) {
        if let Some(VirtualKeyCode::Space) = virtual_key_code {
            if self.splashscreen {
                self.splashscreen = false;
                self.timings.start = Instant::now();
                self.timings.last_frame = Instant::now();
            } else {
                self.pause = !self.pause;
            }
        }
    }

    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        if !self.on_start_called {
            self.state.on_start(graphics);
            self.on_start_called = true;
        }

        self.frame += 1;

        if self.splashscreen {
            graphics.clear_screen(Color::from_hex_rgb(0x0f0f23));
            let rust_image = graphics
                .create_image_from_file_path(
                    None,
                    ImageSmoothingMode::NearestNeighbor,
                    "data/rust.png",
                )
                .unwrap();
            graphics.draw_image(
                (
                    SCREEN_WIDTH as f32 / 2.0 - (RUST_IMAGE_SIZE / 2.0),
                    SCREEN_HEIGHT as f32 / 5.0 - (RUST_IMAGE_SIZE / 2.0),
                ),
                &rust_image,
            );

            self.text_manager.draw_text(
                graphics,
                180,
                TextType::Glow(GlowColor::White),
                (
                    SCREEN_WIDTH as f32 / 2.0,
                    SCREEN_HEIGHT as f32 / 2.0 - 100.0 - 0.0,
                ),
                "Advent".to_string(),
            );
            self.text_manager.draw_text(
                graphics,
                180,
                TextType::Glow(GlowColor::White),
                (
                    SCREEN_WIDTH as f32 / 2.0,
                    SCREEN_HEIGHT as f32 / 2.0 - 100.0 + 150.0,
                ),
                "Of".to_string(),
            );
            self.text_manager.draw_text(
                graphics,
                180,
                TextType::Glow(GlowColor::White),
                (
                    SCREEN_WIDTH as f32 / 2.0,
                    SCREEN_HEIGHT as f32 / 2.0 - 100.0 + 300.0,
                ),
                "Code".to_string(),
            );
            self.text_manager.draw_text(
                graphics,
                130,
                TextType::Glow(GlowColor::White),
                (
                    SCREEN_WIDTH as f32 / 2.0,
                    SCREEN_HEIGHT as f32 / 2.0 - 100.0 + 450.0,
                ),
                "2023".to_string(),
            );
            self.text_manager.draw_text(
                graphics,
                100,
                TextType::Glow(GlowColor::Gold),
                (SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 - 100.0),
                format!("Day {} - Part {}", self.prog_name.0, self.prog_name.1),
            );

            let hat_image = graphics
                .create_image_from_file_path(
                    None,
                    ImageSmoothingMode::NearestNeighbor,
                    "data/hat.png",
                )
                .unwrap();
            graphics.draw_image((130.0, 530.0), &hat_image);
        } else if !self.pause {
            graphics.clear_screen(Color::from_hex_rgb(0x0f0f23));
            let last_frame = Instant::now();
            self.state
                .on_draw(&self.timings, &mut self.text_manager, graphics);
            self.timings.last_frame = last_frame;
        }

        // if self.start.elapsed().as_secs() != 0 {
        //     println!(
        //         "Frame n°{} - FPS {}",
        //         self.frame,
        //         self.frame / self.start.elapsed().as_secs()
        //     );
        // }

        // Request that we draw another frame once this one has finished
        helper.request_redraw();
    }
}

pub(crate) trait State: Sized {
    fn on_draw(
        &mut self,
        timings: &Timings,
        text_manager: &mut TextManager,
        graphics: &mut Graphics2D,
    );

    fn on_start(&mut self, _graphics: &mut Graphics2D) {}
}

pub(crate) fn run<S: State + 'static>(state: S) {
    let prog_name = prog();
    Command::new("bspc")
        .args([
            "rule",
            "-a",
            &format!("{}_{}", prog_name.0, prog_name.1),
            "desktop='Term'",
            "state=floating",
        ])
        .output()
        .expect("failed to execute process");

    // Load the font
    let font_data = include_bytes!("../data/SourceCodePro-Regular.ttf");
    // This only succeeds if collection consists of one font
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    let window = Window::new_centered("AoE", (SCREEN_WIDTH, SCREEN_HEIGHT)).unwrap();

    let my_window = MyWindowHandler {
        pause: false,
        #[cfg(debug_assertions)]
        splashscreen: false,
        #[cfg(not(debug_assertions))]
        splashscreen: true,
        timings: Timings {
            start: Instant::now(),
            last_frame: Instant::now(),
        },
        frame: 0,
        prog_name,
        state,
        text_manager: TextManager {
            font,
            glyphs: HashMap::new(),
            images: HashMap::new(),
            raw_images: HashMap::new(),
        },
        on_start_called: false,
    };

    window.run_loop(my_window);
}

fn prog() -> (String, String) {
    let prog_name = env::args()
        .next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
        .unwrap();

    let (day, part) = prog_name.split_once('_').unwrap();

    (day.to_string(), part.to_string())
}

pub(crate) fn array_to_rectangle(array: [Vector2<f32>; 4]) -> Rectangle {
    Rectangle::new(array[0], array[2])
}

pub(crate) fn square_at_position(center: Vector2<f32>, size: f32) -> [Vector2<f32>; 4] {
    // Here size is the "radius" of the square (so times 2 to have the full width and height)
    rect_at_position(center, size * 2.0, size * 2.0)
}

pub(crate) fn rect_at_position(center: Vector2<f32>, width: f32, height: f32) -> [Vector2<f32>; 4] {
    [
        Vector2::new(center.x - width / 2.0, center.y - height / 2.0),
        Vector2::new(center.x + width / 2.0, center.y - height / 2.0),
        Vector2::new(center.x + width / 2.0, center.y + height / 2.0),
        Vector2::new(center.x - width / 2.0, center.y + height / 2.0),
    ]
}

pub(crate) fn rotate_rect(rect: &mut [Vector2<f32>; 4], center: Vector2<f32>, rotation_rad: f32) {
    rect[0] = rotate_vec(rect[0], center, rotation_rad);
    rect[1] = rotate_vec(rect[1], center, rotation_rad);
    rect[2] = rotate_vec(rect[2], center, rotation_rad);
    rect[3] = rotate_vec(rect[3], center, rotation_rad);
}

pub(crate) fn rotate_vec(
    vec: Vector2<f32>,
    center: Vector2<f32>,
    rotation_rad: f32,
) -> Vector2<f32> {
    // (x·cosθ−y·sinθ ,x·sinθ+y·cosθ)

    Vector2::new(
        center.x + (vec.x - center.x) * rotation_rad.cos()
            - (vec.y - center.y) * rotation_rad.sin(),
        center.y
            + (vec.x - center.x) * rotation_rad.sin()
            + (vec.y - center.y) * rotation_rad.cos(),
    )
}

pub(crate) fn draw_image_rotated(
    graphics: &mut Graphics2D,
    position: Vector2<f32>,
    image: &ImageHandle,
    rot_rad: f32,
) {
    let mut rect = rect_at_position(position, image.size().x as f32, image.size().y as f32);
    rotate_rect(&mut rect, position, rot_rad);

    let image_coords_normalized = Rectangle::new(Vector2::ZERO, Vector2::new(1.0, 1.0));
    let color = Color::WHITE;

    graphics.draw_quad_image_tinted_four_color(
        [rect[0], rect[1], rect[2], rect[3]],
        [color, color, color, color],
        [
            *image_coords_normalized.top_left(),
            image_coords_normalized.top_right(),
            *image_coords_normalized.bottom_right(),
            image_coords_normalized.bottom_left(),
        ],
        &image,
    );
}

pub(crate) fn translate_rect(rect: &mut [Vector2<f32>; 4], translation: Vector2<f32>) {
    rect[0].x += translation.x;
    rect[0].y += translation.y;

    rect[1].x += translation.x;
    rect[1].y += translation.y;

    rect[2].x += translation.x;
    rect[2].y += translation.y;

    rect[3].x += translation.x;
    rect[3].y += translation.y;
}

pub(crate) fn interp(start: f32, end: f32, percentage: f32) -> f32 {
    if start > end {
        start - (start - end) * percentage
    } else {
        start + (end - start) * percentage
    }
}
