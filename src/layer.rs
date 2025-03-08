use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;
use crate::{curve::WeakCurve, style::WeakStyle};

pub type StrongRenderTexture2D =  Arc<ReentrantMutex<RefCell<RenderTexture2D>>>;
pub type WeakRenderTexture2D   = Weak<ReentrantMutex<RefCell<RenderTexture2D>>>;

/// A subset of layers that get rendered in a buffer together
#[derive(Debug, Default)]
pub struct Group {
    /// The layers in the group
    pub layers: Vec<Layer>,
}

/// The actual content of a layer; either artwork or a collection of artwork
#[derive(Debug)]
pub enum LayerContent {
    Group(Group),
    Curve(WeakCurve),
}

#[derive(Debug)]
pub struct Layer {
    /// The name of the layer, shown in the layer panel
    pub name: String,

    /// The artwork content of the layer
    pub content: LayerContent,

    /// The style applied to this layer's content
    ///
    /// Weakly refences a reuseably style stored at the [`Document`][`crate::document::Document`] level
    pub style: WeakStyle,
}
