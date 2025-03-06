#![feature(let_chains, if_let_guard)]

use std::{cell::RefCell, collections::VecDeque, ffi::CString, path::PathBuf, sync::{Arc, RwLock}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;

#[derive(Debug, Clone)]
pub enum Pattern {
    Solid(Color),
    Texture(Arc<ReentrantMutex<RefCell<RenderTexture2D>>>),
}

impl Default for Pattern {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern {
    pub const fn new() -> Self {
        Self::Solid(Color::BLANK)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WidthProfileVertex {
    pub speed_in: f32,
    pub thick: f32,
    pub speed_out: f32,
}

impl WidthProfileVertex {
    pub const fn new() -> Self {
        Self {
            speed_in: 0.0,
            thick: 0.0,
            speed_out: 0.0,
        }
    }

    pub const fn flat(thick: f32) -> Self {
        Self {
            speed_in: 0.0,
            thick,
            speed_out: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WidthProfileControl {
    pub t: f32,
    pub inner: WidthProfileVertex,
    pub outer: WidthProfileVertex,
}

impl WidthProfileControl {
    pub const fn new() -> Self {
        Self {
            t: 0.0,
            inner: WidthProfileVertex::new(),
            outer: WidthProfileVertex::new(),
        }
    }

    pub fn new_even(t: f32, vert: WidthProfileVertex) -> Self {
        Self {
            t,
            inner: vert,
            outer: vert,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WidthProfile {
    Constant { inner: f32, outer: f32 },
    Variable(Vec<WidthProfileControl>),
}

impl Default for WidthProfile {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl WidthProfile {
    pub const fn new() -> Self {
        Self::Constant { inner: 0.0, outer: 0.0 }
    }

    pub const fn new_flat(thick: f32) -> Self {
        Self::Constant { inner: thick, outer: thick }
    }

    pub const fn new_variable() -> Self {
        Self::Variable(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct Stroke {
    pub pattern: Pattern,
    pub width: WidthProfile,
}

#[derive(Debug, Clone)]
pub enum Modifier {
    Fill(Pattern),
    Stroke(Stroke),
}

impl Modifier {
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Fill(_) => "Fill",
            Self::Stroke(_) => "Stroke",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyleItem {
    /// If [`None`], defaults to the modifier variant
    pub name: Option<String>,
    pub modifier: Modifier,
}

impl StyleItem {
    pub const fn new(modifier: Modifier) -> Self {
        Self {
            name: None,
            modifier,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        if let Some(name) = &self.name {
            name
        } else {
            self.modifier.name()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Option<Pattern>,
    pub stroke: Option<Stroke>,
    pub items: Vec<StyleItem>,
}

impl Style {
    pub const fn default_style() -> Self {
        Self {
            fill: Some(Pattern::Solid(Color::SLATEBLUE)),
            stroke: Some(Stroke {
                pattern: Pattern::Solid(Color::BLACK),
                width: WidthProfile::Constant { inner: 5.0, outer: 5.0 },
            }),
            items: Vec::new(),
        }
    }

    pub const fn new() -> Self {
        Self {
            fill: None,
            stroke: None,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorPath {
    pub points: VecDeque<Vector2>,
    pub is_closed: bool,
}

#[derive(Debug)]
pub struct Group {
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
pub enum LayerContent {
    Path(VectorPath),
    Group(Group),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub content: VectorPath,
    pub style: Arc<RwLock<Style>>,
}

#[derive(Debug)]
pub struct Artboard {
    pub name: String,
    pub rect: Rectangle,
}

impl Artboard {
    pub const fn new(name: String, rect: Rectangle) -> Self {
        Self { name, rect }
    }
}

#[derive(Debug)]
pub struct Document {
    pub file_path: Option<PathBuf>,
    pub title: String,
    pub paper_color: Color,
    pub layers: Vec<Box<Layer>>,
    pub artboards: Vec<Box<Artboard>>,
}

impl Document {
    pub const fn new(title: String) -> Self {
        Self {
            file_path: None,
            title,
            paper_color: Color::GRAY,
            layers: Vec::new(),
            artboards: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Selection {
    Path(()),
    Points(Vec<()>),
    Raster(()),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    PointSelect,
    VectorBrush,
    VectorPen,
    RasterBrush,
}

#[derive(Debug)]
pub struct Editor {
    pub document: Document,
    pub selection: Vec<Selection>,
    pub current_tool: Tool,
    pub camera: Camera2D,
    pub current_style: Style,
}

impl Editor {
    pub const fn new(document: Document) -> Self {
        Self {
            document,
            selection: Vec::new(),
            current_tool: Tool::PointSelect,
            camera: Camera2D {
                offset: Vector2::zero(),
                target: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            },
            current_style: Style::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EngineTheme {
    pub color_background: Color,
    pub color_foreground: Color,
    pub color_panel: Color,
    pub color_accent: Color,
    pub font_size: i32,
}

impl EngineTheme {
    pub const fn default_theme() -> Self {
        Self {
            color_background: Color::new(24, 24, 24, 255),
            color_foreground: Color::new(200, 200, 200, 255),
            color_panel: Color::new(48, 48, 48, 255),
            color_accent: Color::BLUEVIOLET,
            font_size: 10,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub editors: Vec<Box<Editor>>,
    pub theme: EngineTheme,
    pub focused_editor: Option<u32>,
}

impl Engine {
    pub const fn new() -> Self {
        Self {
            editors: Vec::new(),
            theme: EngineTheme::default_theme(),
            focused_editor: None,
        }
    }

    /// Pushes the editor and focuses it
    pub fn create_editor(&mut self, editor: Box<Editor>) {
        self.editors.push(editor);
        self.focused_editor = Some(self.editors.len() as u32 - 1);
    }
}

#[allow(clippy::cognitive_complexity, reason = "you always overcomplicate everything when you listen to this about the main function, Amy.")]
fn main() {
    let (mut rl, thread) = init()
        .title("Amity Vector Art")
        .size(1280, 720)
        .resizable()
        .build();

    rl.set_target_fps(60);
    rl.set_window_state(WindowState::set_window_maximized(rl.get_window_state(), true));

    let mut engine = Engine::new();
    {
        let mut document = Document::new("untitled".to_owned());
        let artboard = Box::new(Artboard::new("artboard 1".to_owned(), Rectangle::new(0.0, 0.0, 512.0, 512.0)));
        document.artboards.push(artboard);
        let editor = Box::new(Editor::new(document));
        engine.create_editor(editor);
    }

    while !rl.window_should_close() {
        if let Some(focused_editor) = &engine.focused_editor {
            let editor = engine.editors[*focused_editor as usize].as_mut();
            let scroll: Vector2 = rl.get_mouse_wheel_move_v().into();
            editor.camera.target -= scroll;
            editor.camera.offset += scroll;
            match editor.current_tool {
                Tool::PointSelect => {

                }

                Tool::VectorBrush => {

                }

                Tool::VectorPen => {

                }

                Tool::RasterBrush => {

                }
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(engine.theme.color_background);

        for (i, editor) in engine.editors.iter().map(Box::as_ref).enumerate() {
            {
                let mut d = d.begin_mode2D(editor.camera);
                for artboard in &editor.document.artboards {
                    d.draw_rectangle_rec(artboard.rect, editor.document.paper_color);
                }
            }
            if let Some(focused_editor) = &engine.focused_editor && i as u32 == *focused_editor {
                match editor.current_tool {
                    Tool::PointSelect => {

                    }

                    Tool::VectorBrush => {

                    }

                    Tool::VectorPen => {

                    }

                    Tool::RasterBrush => {

                    }
                }
            }
            for artboard in &editor.document.artboards {
                let corner = d.get_world_to_screen2D(Vector2::new(artboard.rect.x, artboard.rect.y), editor.camera);
                d.draw_text(&artboard.name, corner.x as i32, corner.y as i32 - engine.theme.font_size, engine.theme.font_size, engine.theme.color_foreground);
            }
        }

        const TAB_PADDING_H: f32 = 5.0;
        const TAB_PADDING_V: f32 = 3.0;
        const TAB_MAX_WIDTH: f32 = 100.0;
        let mut tab_rect = Rectangle::new(0.0, 0.0, 0.0, engine.theme.font_size as f32 + TAB_PADDING_V * 2.0);
        for (i, editor) in engine.editors.iter().map(Box::as_ref).enumerate() {
            let tab_name = editor.document.title.as_str();
            let name_width = d.measure_text(tab_name, engine.theme.font_size) as f32 + TAB_PADDING_H * 2.0;
            tab_rect.width = name_width.min(TAB_MAX_WIDTH);
            let tab_color = if engine.focused_editor.is_some_and(|focused| focused == i as u32)  {
                engine.theme.color_accent
            } else {
                engine.theme.color_panel
            };
            d.draw_rectangle_rec(tab_rect, tab_color);
            d.draw_text(tab_name, (tab_rect.x + TAB_PADDING_H) as i32, (tab_rect.y + TAB_PADDING_V) as i32, engine.theme.font_size, engine.theme.color_foreground);

            tab_rect.x += tab_rect.width;
        }
    }
}
