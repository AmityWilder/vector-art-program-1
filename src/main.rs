#![feature(let_chains, if_let_guard, arbitrary_self_types)]

use std::{cell::RefCell, path::PathBuf, sync::{Arc, RwLock, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::{*, KeyboardKey::*, MouseButton::*};

pub type ArcRTex = Arc<ReentrantMutex<RefCell<RenderTexture2D>>>;
pub type WeakRTex = Weak<ReentrantMutex<RefCell<RenderTexture2D>>>;

#[derive(Debug, Clone)]
pub enum Pattern {
    Solid(Color),
    Texture(WeakRTex),
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

pub type ArcStyle = Arc<ReentrantMutex<RefCell<Style>>>;
pub type WeakStyle = Weak<ReentrantMutex<RefCell<Style>>>;

#[derive(Debug, Clone, Copy, Default)]
pub struct CurvePoint {
    pub c_in: Option<Vector2>,
    pub p: Vector2,
    pub c_out: Option<Vector2>,
}

#[derive(Debug, Clone, Default)]
pub struct Curve {
    pub points: Vec<CurvePoint>,
    pub is_closed: bool,
}

pub type ArcCurve = Arc<ReentrantMutex<RefCell<Curve>>>;
pub type WeakCurve = Weak<ReentrantMutex<RefCell<Curve>>>;

impl From<Rectangle> for Curve {
    fn from(Rectangle { x, y, width, height }: Rectangle) -> Self {
        let left = x;
        let top = y;
        let right = x + width;
        let bottom = y + height;
        Self {
            points: vec![
                CurvePoint { c_in: None, p: Vector2::new(left, top), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(right, top), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(right, bottom), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(left, bottom), c_out: None },
            ],
            is_closed: true,
        }
    }
}

impl Curve {
    pub const fn new() -> Self {
        Self {
            points: Vec::new(),
            is_closed: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Group {
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
pub enum LayerContent {
    Curve(Curve),
    Group(Group),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub content: LayerContent,
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
    pub rtextures: Vec<ArcRTex>,
    pub styles: Vec<ArcStyle>,
    pub curves: Vec<ArcCurve>,

    pub file_path: Option<PathBuf>,
    pub title: String,
    pub paper_color: Color,
    pub layers: Vec<Layer>,
    pub artboards: Vec<Artboard>,
}

impl Document {
    pub const fn new(title: String) -> Self {
        Self {
            rtextures: Vec::new(),
            styles: Vec::new(),
            curves: Vec::new(),

            file_path: None,
            title,
            paper_color: Color::GRAY,
            layers: Vec::new(),
            artboards: Vec::new(),
        }
    }

    pub fn create_style(&mut self, style: Style) -> &ArcStyle {
        self.styles.push(Arc::new(ReentrantMutex::new(RefCell::new(style))));
        self.styles.last().expect("should have at least one element after push")
    }

    pub fn create_render_texture(&mut self, rtex: RenderTexture2D) -> &ArcRTex {
        self.rtextures.push(Arc::new(ReentrantMutex::new(RefCell::new(rtex))));
        self.rtextures.last().expect("should have at least one element after push")
    }

    pub fn create_curve(&mut self, curve: Curve) -> &ArcCurve {
        self.curves.push(Arc::new(ReentrantMutex::new(RefCell::new(curve))));
        self.curves.last().expect("should have at least one element after push")
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
    pub current_style: Weak<ReentrantMutex<RefCell<Style>>>,
}

impl Editor {
    pub const fn new(document: Document, current_style: Weak<ReentrantMutex<RefCell<Style>>>) -> Self {
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
            current_style,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineTheme {
    pub color_background: Color,
    pub color_foreground: Color,
    pub color_panel: Color,
    pub color_panel_edge: Color,
    pub color_accent: Color,
    pub font_size: i32,
}

impl EngineTheme {
    pub const fn default_theme() -> Self {
        Self {
            color_background: Color::new(24, 24, 24, 255),
            color_foreground: Color::new(200, 200, 200, 255),
            color_panel: Color::new(48, 48, 48, 255),
            color_panel_edge: Color::new(32, 32, 32, 255),
            color_accent: Color::BLUEVIOLET,
            font_size: 10,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub editors: Vec<Editor>,
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
    pub fn create_editor(&mut self, editor: Editor) {
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
        let artboard = Artboard::new("artboard 1".to_owned(), Rectangle::new(0.0, 0.0, 512.0, 512.0));
        let style0 = Arc::downgrade(document.create_style(Style::default_style()));
        document.artboards.push(artboard);
        let editor = Editor::new(document, style0);
        engine.create_editor(editor);
    }

    while !rl.window_should_close() {
        if let Some(focused_editor) = &engine.focused_editor {
            let editor = &mut engine.editors[*focused_editor as usize];

            if rl.is_key_pressed(KEY_P) {
                editor.current_tool = Tool::PointSelect;
            } else if rl.is_key_pressed(KEY_B) {
                if rl.is_key_down(KEY_LEFT_SHIFT) {
                    editor.current_tool = Tool::VectorBrush;
                } else {
                    editor.current_tool = Tool::RasterBrush;
                }
            } else if rl.is_key_pressed(KEY_V) {
                editor.current_tool = Tool::PointSelect;
            }

            // zoom and pan
            {
                let mut pan = Vector2::zero();

                let mut scroll = Vector2::from(rl.get_mouse_wheel_move_v());
                if rl.is_key_down(KEY_LEFT_ALT) {
                    const ZOOM_SPEED: f32 = 1.5;
                    const MIN_ZOOM: f32 = 0.125;
                    const MAX_ZOOM: f32 = 64.0;
                    let zoom = if scroll.x.abs() < scroll.y.abs() { scroll.y } else { scroll.x };
                    if zoom > 0.0 && editor.camera.zoom < MAX_ZOOM {
                        editor.camera.zoom *= ZOOM_SPEED;
                    } else if zoom < 0.0 && editor.camera.zoom > MIN_ZOOM {
                        editor.camera.zoom /= ZOOM_SPEED;
                    }
                } else {
                    if rl.is_key_down(KEY_LEFT_SHIFT) {
                        std::mem::swap(&mut scroll.x, &mut scroll.y);
                    }
                    pan += scroll * 20.0;
                }
                if rl.is_mouse_button_down(MOUSE_BUTTON_MIDDLE) {
                    let drag = rl.get_mouse_delta();
                    pan += drag;
                }

                editor.camera.target += (rl.get_mouse_delta() - pan) / editor.camera.zoom;
                editor.camera.offset = rl.get_mouse_position();
            }

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

        for (i, editor) in engine.editors.iter().enumerate() {
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
        let mut tab_rect = Rectangle::new(0.0, 0.0, d.get_render_width() as f32, engine.theme.font_size as f32 + TAB_PADDING_V * 2.0);
        d.draw_rectangle_rec(tab_rect, engine.theme.color_panel_edge);
        for (i, editor) in engine.editors.iter().enumerate() {
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
