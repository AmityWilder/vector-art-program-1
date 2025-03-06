use std::path::PathBuf;
use raylib::prelude::*;

#[derive(Debug)]
pub struct VectorPath {

}

#[derive(Debug)]
pub struct Raster {

}

#[derive(Debug)]
pub enum LayerContent {
    Path(VectorPath),
    Raster(Raster),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub content: LayerContent,
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
}

impl Engine {
    pub const fn new() -> Self {
        Self {
            editors: Vec::new(),
            theme: EngineTheme::default_theme(),
        }
    }
}

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
        let mut editor = Box::new(Editor::new(document));
        editor.camera.target.y -= 32.0;
        editor.camera.target.x -= 32.0;
        engine.editors.push(editor);
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(engine.theme.color_background);

        for editor in &engine.editors {
            {
                let mut d = d.begin_mode2D(editor.camera);
                for artboard in &editor.document.artboards {
                    d.draw_rectangle_rec(artboard.rect, editor.document.paper_color);
                }
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
            for artboard in &editor.document.artboards {
                let corner = d.get_world_to_screen2D(Vector2::new(artboard.rect.x, artboard.rect.y), editor.camera);
                d.draw_text(&artboard.name, corner.x as i32, corner.y as i32 - engine.theme.font_size, engine.theme.font_size, engine.theme.color_foreground);
            }
        }
    }
}
