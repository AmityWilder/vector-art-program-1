use std::{cell::RefCell, sync::{Arc, Weak}};

use parking_lot::ReentrantMutex;
use raylib::prelude::*;

#[derive(Debug, Clone)]
pub enum Pattern {
    Solid(Color),
    Texture(WeakRenderTexture2D),
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

pub type StrongStyle = Arc<ReentrantMutex<RefCell<Style>>>;
pub type WeakStyle = Weak<ReentrantMutex<RefCell<Style>>>;
