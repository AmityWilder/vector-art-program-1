use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct CurvePoint {
    pub c_in: Option<Vector2>,
    pub p: Vector2,
    pub c_out: Option<Vector2>,
}

#[derive(Debug, Clone, Default)]
pub struct Curve {
    pub points: Vec<CurvePoint>,
    pub is_closed: bool,
}

pub type StrongCurve = Arc<ReentrantMutex<RefCell<Curve>>>;
pub type WeakCurve = Weak<ReentrantMutex<RefCell<Curve>>>;

impl From<Rectangle> for Curve {
    fn from(Rectangle { x, y, width, height }: Rectangle) -> Self {
        let left = x;
        let top = y;
        let right = x + width;
        let bottom = y + height;
        Self {
            points: vec![
                CurvePoint { c_in: None, p: Vector2::new(left, top), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(right, top), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(right, bottom), c_out: None },
                CurvePoint { c_in: None, p: Vector2::new(left, bottom), c_out: None },
            ],
            is_closed: true,
        }
    }
}

impl Curve {
    pub const fn new() -> Self {
        Self {
            points: Vec::new(),
            is_closed: false,
        }
    }
}
