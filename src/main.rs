#![feature(let_chains, if_let_guard, arbitrary_self_types)]
#![warn(arithmetic_overflow, clippy::arithmetic_side_effects)]

use std::{cell::RefCell, path::PathBuf, sync::{Arc, RwLock, Weak}};
use engine::{Engine, EngineTab, EngineTabData};
use parking_lot::ReentrantMutex;
use raylib::prelude::{KeyboardKey::*, MouseButton::*, *};

pub mod engine;

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

/// A style that may not be inside a document yet
#[derive(Debug)]
pub enum MaybeNewStyle {
    New(Style),
    Existing(WeakStyle),
}

impl Default for MaybeNewStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl MaybeNewStyle {
    pub const fn new_default() -> Self {
        Self::New(Style::default_style())
    }

    pub const fn new() -> Self {
        Self::New(Style::new())
    }
}

#[derive(Debug)]
pub struct Editor {
    pub document: Document,
    pub selection: Vec<Selection>,
    pub current_tool: Tool,
    pub camera: Camera2D,
    pub current_style: MaybeNewStyle,
}

impl Editor {
    pub const fn new(document: Document, current_style: MaybeNewStyle) -> Self {
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

    pub fn upgrade_current_style(&mut self) {
        if let MaybeNewStyle::New(style) = std::mem::take(&mut self.current_style) {
            self.current_style = MaybeNewStyle::Existing(Arc::downgrade(self.document.create_style(style)))
        }
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

    // initialize engine
    let mut engine = Engine::new();
    #[cfg(debug_assertions)]
    {
        engine.create_editor({
            Editor::new({
                let mut document = Document::new("untitled".to_owned());
                let artboard = Artboard::new("artboard 1".to_owned(), Rectangle::new(0.0, 0.0, 512.0, 512.0));
                document.artboards.push(artboard);
                document
            }, MaybeNewStyle::new_default())
        });
    }

    while !rl.window_should_close() {
        // editor tabs
        {
            if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
                let mouse_pos = rl.get_mouse_position();
                if let Some(EngineTab { data, .. }) = engine.tab_iter().find(|tab| tab.rect.check_collision_point_rec(mouse_pos)) {
                    match data {
                        EngineTabData::Editor { index, close_button_rect, .. } => {
                            if close_button_rect.check_collision_point_rec(mouse_pos) {
                                _ = engine.remove_editor(index);
                            } else {
                                engine.focus_editor(index).expect("tab_iter should only iterate over valid indices");
                            }
                        }

                        EngineTabData::New => {
                            engine.create_editor({
                                Editor::new({
                                    let document = Document::new("untitled".to_owned());
                                    document
                                }, MaybeNewStyle::new_default())
                            });
                        }
                    }
                }
            }
        }

        // tick editor
        if let Some(editor) = engine.focused_editor_mut() {
            // editor inputs
            {
                if rl.is_key_pressed(KEY_P) {
                    editor.current_tool = Tool::PointSelect;
                } else if rl.is_key_pressed(KEY_B) {
                    editor.current_tool =
                        if rl.is_key_down(KEY_LEFT_SHIFT) {
                            Tool::VectorBrush
                        } else {
                            Tool::RasterBrush
                        }
                } else if rl.is_key_pressed(KEY_V) {
                    editor.current_tool = Tool::PointSelect;
                }
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
                editor.camera.offset += rl.get_mouse_delta(); // equivalent to `rl.get_mouse_position()` when loading a file
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

        // draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(engine.theme.color_background);

        // draw focused editor
        if let Some(editor) = engine.focused_editor() {
            // draw artboard background
            {
                let mut d = d.begin_mode2D(editor.camera);
                for artboard in &editor.document.artboards {
                    d.draw_rectangle_rec(artboard.rect, editor.document.paper_color);
                }
            }

            // draw artwork
            {
                // TODO
            }

            // draw tool visuals
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

            // draw artboard name
            for artboard in &editor.document.artboards {
                let corner = d.get_world_to_screen2D(Vector2::new(artboard.rect.x, artboard.rect.y), editor.camera);
                d.draw_text(&artboard.name, corner.x as i32, corner.y as i32 - engine.theme.font_size, engine.theme.font_size, engine.theme.color_foreground);
            }
        }

        // draw editor tabs
        d.draw_rectangle_rec(engine.tab_well(d.get_render_width() as f32), engine.theme.color_panel_edge);
        for tab in engine.tab_iter() {
            let is_hovered = tab.rect.check_collision_point_rec(d.get_mouse_position());
            match tab.data {
                EngineTabData::Editor { index, editor, close_button_rect } => {
                    let is_close_button_hovered = is_hovered && close_button_rect.check_collision_point_rec(d.get_mouse_position());
                    let is_focused = engine.focused_editor_index_eq(index);

                    let tab_color = if is_focused {
                        engine.theme.color_accent
                    } else if is_hovered {
                        engine.theme.color_panel_edge
                    } else {
                        engine.theme.color_panel
                    };

                    let close_color = if is_close_button_hovered {
                        engine.theme.color_danger
                    } else if is_focused {
                        engine.theme.color_foreground
                    } else if is_hovered {
                        engine.theme.color_panel
                    } else {
                        engine.theme.color_panel_edge
                    };

                    d.draw_rectangle_rec(tab.rect, tab_color);
                    d.draw_rectangle_rec(close_button_rect, close_color);
                    d.draw_text(
                        &editor.document.title,
                        (tab.rect.x + Engine::TAB_PADDING_H) as i32,
                        (tab.rect.y + Engine::TAB_PADDING_V) as i32,
                        engine.theme.font_size,
                        engine.theme.color_foreground,
                    );
                }

                EngineTabData::New => {
                    let tab_color = if is_hovered {
                        engine.theme.color_accent
                    } else {
                        engine.theme.color_panel
                    };

                    d.draw_rectangle_rec(tab.rect, tab_color);
                    d.draw_text(
                        "+",
                        (tab.rect.x + Engine::TAB_PADDING_H) as i32,
                        (tab.rect.y + Engine::TAB_PADDING_V) as i32,
                        engine.theme.font_size,
                        engine.theme.color_foreground,
                    );
                }
            }
        }
    }
}
