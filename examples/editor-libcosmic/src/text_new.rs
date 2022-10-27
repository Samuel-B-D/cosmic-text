// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmic::iced_native::{
    {Color, Element, Length, Point, Rectangle, Size, Theme},
    layout::{self, Layout},
    renderer,
    widget::{self, tree, Widget},
};
use cosmic_text::{
    Attrs,
    AttrsList,
    SwashCache,
    TextBufferLine,
    TextMetrics,
};
use std::{
    cmp,
    sync::Mutex,
    time::Instant,
};

pub struct Appearance {
    background_color: Option<Color>,
}

pub trait StyleSheet {
    fn appearance(&self) -> Appearance;
}

impl StyleSheet for Theme {
    fn appearance(&self) -> Appearance {
        Appearance {
            background_color: None,
        }
    }
}

pub struct Text {
    line: Mutex<TextBufferLine<'static>>,
    metrics: TextMetrics,
}

impl Text {
    pub fn new(string: &str) -> Self {
        Self {
            //TODO: make it possible to set attrs
            line: Mutex::new(TextBufferLine::new(
                string,
                AttrsList::new(Attrs::new())
            )),
            metrics: TextMetrics::new(14, 20),
        }
    }
}

pub fn text(string: &str) -> Text {
    Text::new(string)
}

impl<Message, Renderer> Widget<Message, Renderer> for Text
where
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        println!("layout");

        let mut line = self.line.lock().unwrap();

        let mut max_x = 0;
        let mut line_y = 0;
        for layout_line in line.layout(
            &crate::FONT_SYSTEM,
            self.metrics.font_size,
            limits.max().width as i32
        ) {
            for glyph in layout_line.glyphs.iter() {
                max_x = cmp::max(max_x, (glyph.x + glyph.w) as i32);
            }
            line_y += self.metrics.line_height;
        }

        println!("{}, {}", max_x, line_y);

        layout::Node::new(Size::new(max_x as f32, line_y as f32))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        if let Some(background_color) = theme.appearance().background_color {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                background_color
            );
        }

        let mut line = self.line.lock().unwrap();
        let mut cache = state.cache.lock().unwrap();

        let mut line_y = self.metrics.font_size;
        for layout_line in line.layout(
            &crate::FONT_SYSTEM,
            self.metrics.font_size,
            layout.bounds().width as i32
        ) {
            for glyph in layout_line.glyphs.iter() {
                let (cache_key, x_int, y_int) = (glyph.cache_key, glyph.x_int, glyph.y_int);

                let glyph_color = match glyph.color_opt {
                    Some(some) => some,
                    None => cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF), //TODO: get from theme
                };

                cache.with_pixels(cache_key, glyph_color, |x, y, color| {
                    let a = color.a();
                    if a > 0 {
                        let r = color.r();
                        let g = color.g();
                        let b = color.b();
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle::new(
                                    Point::new(
                                        layout.bounds().x + (x_int + x) as f32,
                                        layout.bounds().y + (line_y + y_int + y) as f32
                                    ),
                                    Size::new(1.0, 1.0)
                                ),
                                border_radius: 0.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            Color::from_rgba8(r, g, b, a as f32 / 255.0),
                        );
                    }
                });
            }
            line_y += self.metrics.line_height;
        }
    }
}

impl<'a, Message, Renderer> From<Text> for Element<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(text: Text) -> Self {
        Self::new(text)
    }
}

pub struct State {
    cache: Mutex<SwashCache<'static>>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        let instant = Instant::now();

        let state = State {
            cache: Mutex::new(SwashCache::new(&crate::FONT_SYSTEM)),
        };

        log::info!("Created state in {:?}", instant.elapsed());

        state
    }
}