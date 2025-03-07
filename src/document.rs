use std::{cell::RefCell, path::PathBuf, sync::Arc};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;
use crate::{curve::{Curve, StrongCurve}, layer::{Layer, StrongRenderTexture2D}, style::{StrongStyle, Style}};

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
    pub rtextures: Vec<StrongRenderTexture2D>,
    pub styles: Vec<StrongStyle>,
    pub curves: Vec<StrongCurve>,

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

    pub fn create_style(&mut self, style: Style) -> &StrongStyle {
        self.styles.push(Arc::new(ReentrantMutex::new(RefCell::new(style))));
        self.styles.last().expect("should have at least one element after push")
    }

    pub fn create_render_texture(&mut self, rtex: RenderTexture2D) -> &StrongRenderTexture2D {
        self.rtextures.push(Arc::new(ReentrantMutex::new(RefCell::new(rtex))));
        self.rtextures.last().expect("should have at least one element after push")
    }

    pub fn create_curve(&mut self, curve: Curve) -> &StrongCurve {
        self.curves.push(Arc::new(ReentrantMutex::new(RefCell::new(curve))));
        self.curves.last().expect("should have at least one element after push")
    }
}
