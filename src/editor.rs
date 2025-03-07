use std::sync::Arc;
use raylib::prelude::*;
use crate::{document::Document, style::{Style, WeakStyle}};

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
