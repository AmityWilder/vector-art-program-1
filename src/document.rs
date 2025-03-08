use std::{cell::RefCell, path::PathBuf, sync::Arc};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;
use crate::{curve::{Curve, StrongCurve}, layer::{Layer, StrongRenderTexture2D}, style::{StrongStyle, StrongWidthProfile, Style, WidthProfile}};

/// An independently-exported crop region of the document artwork
#[derive(Debug)]
pub struct Artboard {
    /// The display name of the artboard
    pub name: String,

    /// The worldspace rectangle the artboard crops
    pub rect: Rectangle,
}

impl Artboard {
    /// Construct a new artboard
    pub const fn new(name: String, rect: Rectangle) -> Self {
        Self { name, rect }
    }
}

/// A self-contained vector artwork document
#[derive(Debug)]
pub struct Document {
    /// Multiple styles can reference the same raster
    ///
    /// A raster should be removed when the weak count is 0; the
    /// document doesn't need a raster if no style references it
    pub rasters: Vec<StrongRenderTexture2D>,

    /// Multiple styles can reference the same width_profile
    ///
    /// A width_profile should be removed when the weak count is 0; the
    /// document doesn't need a width_profile if no style references it
    pub width_profiles: Vec<StrongWidthProfile>,

    /// Multiple curves can reference the same style
    ///
    /// A style should be removed when the weak count is 0; the
    /// document doesn't need a style if no curve references it
    pub styles: Vec<StrongStyle>,

    /// Multiple layers can reference the same curve
    ///
    /// A curve should be removed when the weak count is 0; the
    /// document doesn't need a curve if no layer references it
    pub curves: Vec<StrongCurve>,

    /// Where the document is stored on the harddrive \
    /// [`None`] if new, unsaved document
    pub file_path: Option<PathBuf>,

    /// Displayname of the document
    pub title: String,

    /// Color of the background in the artboard
    pub paper_color: Color,

    /// Enby tree structure
    pub layers: Vec<Layer>,

    /// Separately exported cropped regions of vector artwork
    pub artboards: Vec<Artboard>,
}

impl Document {
    /// Construct an empty file without any allocations
    pub const fn new(title: String) -> Self {
        Self {
            rasters: Vec::new(),
            width_profiles: Vec::new(),
            styles: Vec::new(),
            curves: Vec::new(),

            file_path: None,
            title,
            paper_color: Color::GRAY,
            layers: Vec::new(),
            artboards: Vec::new(),
        }
    }

    /// Push a new local raster to the document and get a reference to it
    pub fn create_raster(&mut self, rtex: RenderTexture2D) -> &StrongRenderTexture2D {
        self.rasters.push(Arc::new(ReentrantMutex::new(RefCell::new(rtex))));
        #[cfg(debug_assertions)] {
            self.rasters.last().expect("should have at least one element after push")
        } #[cfg(not(debug_assertions))] {
            unsafe { self.rasters.last().unwrap_unchecked() }
        }
    }

    /// Push a new local width profile to the document and get a reference to it
    pub fn create_width_profile(&mut self, profile: WidthProfile) -> &StrongWidthProfile {
        self.width_profiles.push(Arc::new(ReentrantMutex::new(RefCell::new(profile))));
        #[cfg(debug_assertions)] {
            self.width_profiles.last().expect("should have at least one element after push")
        } #[cfg(not(debug_assertions))] {
            unsafe { self.width_profiles.last().unwrap_unchecked() }
        }
    }

    /// Push a new local style to the document and get a reference to it
    pub fn create_style(&mut self, style: Style) -> &StrongStyle {
        self.styles.push(Arc::new(ReentrantMutex::new(RefCell::new(style))));
        #[cfg(debug_assertions)] {
            self.styles.last().expect("should have at least one element after push")
        } #[cfg(not(debug_assertions))] {
            unsafe { self.styles.last().unwrap_unchecked() }
        }
    }

    /// Push a new local curve to the document and get a reference to it
    pub fn create_curve(&mut self, curve: Curve) -> &StrongCurve {
        self.curves.push(Arc::new(ReentrantMutex::new(RefCell::new(curve))));
        #[cfg(debug_assertions)] {
            self.curves.last().expect("should have at least one element after push")
        } #[cfg(not(debug_assertions))] {
            unsafe { self.curves.last().unwrap_unchecked() }
        }
    }
}
