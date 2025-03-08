use std::{cell::RefCell, sync::{Arc, Weak}};

use parking_lot::ReentrantMutex;
use raylib::prelude::*;

/// A color or texture that can be applied to a stroke or fill
#[derive(Debug, Clone)]
pub enum Pattern {
    /// A solid color across the entire region
    Solid(Color),

    /// A texture applied to the region
    ///
    /// The texture can be painted to with [`RasterBrush`][`crate::editor::Tool::RasterBrush`],
    /// modifying all linked instances simultaneously
    Texture(WeakRenderTexture2D),
}

impl Default for Pattern {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern {
    /// A transparent solid pattern
    pub const fn new() -> Self {
        Self::Solid(Color::BLANK)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WidthProfileVertex {
    /// The entry intensity of the thickness
    pub speed_in: f32,

    /// The thickness of this control
    pub thick: f32,

    /// The exit intensity of the thickness
    pub speed_out: f32,
}

impl WidthProfileVertex {
    /// Construct an empty vertex
    pub const fn new() -> Self {
        Self {
            speed_in: 0.0,
            thick: 0.0,
            speed_out: 0.0,
        }
    }

    /// Construct a corner vertex (0 in/out velocity)
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
    /// The t-value along the curve to which this control relates
    pub t: f32,

    /// The thickness on the "counter-clockwise-rotated tangent" side of the line
    pub inner: WidthProfileVertex,

    /// The thickness on the "clockwise-rotated tangent" side of the line
    pub outer: WidthProfileVertex,
}

impl WidthProfileControl {
    /// Construct an empty control filled with 0s
    pub const fn new() -> Self {
        Self {
            t: 0.0,
            inner: WidthProfileVertex::new(),
            outer: WidthProfileVertex::new(),
        }
    }

    /// Construct a new control at `t` with the same thickness on both sides
    pub fn new_even(t: f32, vert: WidthProfileVertex) -> Self {
        Self {
            t,
            inner: vert,
            outer: vert,
        }
    }
}

/// A curve representing the thickness of a stroke along a path
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
    /// The width profile used when the user hasn't customized it
    pub const fn default_width_profile() -> Self {
        Self::Constant { inner: 5.0, outer: 5.0 }
    }

    /// Construct an empty width profile
    pub const fn new() -> Self {
        Self::Constant { inner: 0.0, outer: 0.0 }
    }

    /// Construct a new constant-width profile that is equally thick on both sides
    pub const fn new_flat(thick: f32) -> Self {
        Self::Constant { inner: thick, outer: thick }
    }

    /// Construct an empty variable-width profile without allocating
    pub const fn new_variable() -> Self {
        Self::Variable(Vec::new())
    }
}

pub type StrongWidthProfile =  Arc<ReentrantMutex<RefCell<WidthProfile>>>;
pub type WeakWidthProfile   = Weak<ReentrantMutex<RefCell<WidthProfile>>>;

#[derive(Debug, Clone)]
pub struct Stroke {
    /// The color pattern applied to the stroke
    pub pattern: Pattern,

    /// The thickness curve of the stroke
    pub width: Option<WeakWidthProfile>,
}

impl Default for Stroke {
    fn default() -> Self {
        Self::new()
    }
}

impl Stroke {
    /// Construct a transparent stroke with no width
    pub const fn new() -> Self {
        Self {
            pattern: Pattern::new(),
            width: None,
        }
    }
}

/// A style modifier
///
/// Represents a transformation of the previous style or base path
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Modifier {
    /// Applies an additional fill to a path
    Fill(Pattern),

    /// Outlines a path with a styled, possibly variable-width stroke
    Stroke(Stroke),

    // ...
}

impl Modifier {
    /// Get the Title Case static name of the modifier
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Fill(_) => "Fill",
            Self::Stroke(_) => "Stroke",
            // ...
        }
    }
}

/// A wrapper for [`Modifier`] that includes the display name
#[derive(Debug, Clone)]
pub struct StyleItem {
    /// If [`None`], defaults to the modifier variant
    pub name: Option<String>,

    /// The modifier being applied by this item
    pub modifier: Modifier,
}

impl StyleItem {
    /// Construct an unnamed style item
    pub const fn new(modifier: Modifier) -> Self {
        Self {
            name: None,
            modifier,
        }
    }

    /// The display name of the item
    ///
    /// Defaults to [`Modifier::name`] if `name` is [`None`]
    #[inline]
    pub fn name(&self) -> &str {
        if let Some(name) = &self.name {
            name
        } else {
            self.modifier.name()
        }
    }
}

/// The appearance of a layer
#[derive(Debug, Clone)]
pub struct Style {
    /// Every path must have at least one (possibly transparent) fill
    ///
    /// A transparent fill tells the renderer to skip filling the path
    pub fill: Pattern,

    /// Every path must have at least one (possibly transparent, zero-width) stroke
    ///
    /// A transparent fill or zero-width thickness tells the renderer to skip outlining the path
    pub stroke: Stroke,

    /// The collection of appearance modifiers
    ///
    /// Stored in the order they are applied
    pub items: Vec<StyleItem>,
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl Style {
    /// The style used when the user hasn't customized it
    pub const fn default_style(width: WeakWidthProfile) -> Self {
        Self {
            fill: Pattern::Solid(Color::SLATEBLUE),
            stroke: Stroke {
                pattern: Pattern::Solid(Color::BLACK),
                width: Some(width),
            },
            items: Vec::new(),
        }
    }

    /// Construct an empty, transparent, zero-width style without allocating
    pub const fn new() -> Self {
        Self {
            fill: Pattern::new(),
            stroke: Stroke::new(),
            items: Vec::new(),
        }
    }
}

pub type StrongStyle =  Arc<ReentrantMutex<RefCell<Style>>>;
pub type WeakStyle   = Weak<ReentrantMutex<RefCell<Style>>>;
