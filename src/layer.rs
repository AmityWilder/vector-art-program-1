use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;
use crate::{curve::Curve, style::StrongStyle};

pub type StrongRenderTexture2D =  Arc<ReentrantMutex<RefCell<RenderTexture2D>>>;
pub type WeakRenderTexture2D   = Weak<ReentrantMutex<RefCell<RenderTexture2D>>>;

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
    pub style: StrongStyle,
}
