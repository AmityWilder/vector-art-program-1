use std::path::PathBuf;
use raylib::prelude::*;

pub struct VectorPath {

}

pub struct Raster {

}

pub enum LayerContent {
    Path(VectorPath),
    Raster(Raster),
}

pub struct Layer {
    pub name: String,
    pub content: LayerContent,
}

pub struct Document {
    pub file_path: Option<PathBuf>,
    pub title: String,
    pub paper_color: Color,
    pub layers: Vec<Box<Layer>>,
}

pub enum Selection {
    Path,
    Raster,
}

pub struct Editor {
    pub document: Document,
    pub selection: Vec<Selection>,
}

pub struct Engine {
    pub documents: Vec<Box<Document>>,
}

fn main() {
    let (mut rl, thread) = init()
        .title("Amity Vector Art")
        .size(1280, 720)
        .resizable()
        .build();

    rl.set_window_state(WindowState::set_window_maximized(rl.get_window_state(), true));

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
    }
}
