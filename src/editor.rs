use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;
use crate::{document::Document, style::{Style, WeakStyle}};

/// A collection selected items
#[derive(Debug)]
#[non_exhaustive]
pub enum Selection {
    // Path(()),
    // Points(Vec<()>),
    // Raster(()),
}

/// Enumation of how user inputs should be interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    /// Tool for selecting individual points in one or more vector paths
    ///
    /// ### Selection
    /// // TODO
    #[default]
    PointSelect,

    /// Tool for painting or sculpting vector paths naturally with a stylus
    ///
    /// ### Selection
    /// // TODO
    VectorBrush,

    /// Tool for constructing or editing vector paths precisely with a mouse
    ///
    /// ### Selection
    /// // TODO
    VectorPen,

    /// Tool for painting pixels with a brush style
    ///
    /// ### Selection
    /// // TODO
    RasterBrush,
}

/// A reuseable that may not be inside a document yet
#[derive(Debug)]
pub enum MaybeNew<T> {
    New(T),
    Existing(Weak<ReentrantMutex<RefCell<T>>>),
}

impl Default for MaybeNew<Style> {
    fn default() -> Self {
        Self::new()
    }
}

impl MaybeNew<Style> {
    pub const fn new_default() -> Self {
        Self::New(Style::default_style())
    }

    pub const fn new() -> Self {
        Self::New(Style::new())
    }
}

#[derive(Debug)]
pub struct Editor {
    /// The document this editor is editing
    pub document: Document,

    /// The current selection
    ///
    /// Takes on different meanings depending on `current_tool`
    pub selection: Vec<Selection>,

    /// The way user input should be used
    pub current_tool: Tool,

    /// The viewport camera
    pub camera: Camera2D,

    /// The style being edited right now
    ///
    /// May reference an existing style in the document, or a
    /// new one that should be applied to the next styled item
    /// created by this editor
    pub current_style: MaybeNew<Style>,
}

impl Editor {
    /// Construct a new editor with default values and no allocation
    pub const fn new(document: Document, current_style: MaybeNew<Style>) -> Self {
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

    /// Push `current_style` to the document's local styles and get a weak reference to it
    pub fn upgrade_current_style(&mut self) -> &WeakStyle {
        if let MaybeNew::New(style) = std::mem::take(&mut self.current_style) {
            let style = Arc::downgrade(self.document.create_style(style));
            self.current_style = MaybeNew::Existing(style);
        }
        let MaybeNew::Existing(weak_style) = &self.current_style else { unreachable!("current_style should have either already been Existing or just been assigned Existing") };
        weak_style
    }
}
