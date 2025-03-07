use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CurvePoint {
    pub c_in:  na::Vector2<f32>, // relative to p
    pub p:     na::Vector2<f32>,
    pub c_out: na::Vector2<f32>, // relative to p
}

struct CurveIter<'a> {
    iter: std::slice::Iter<'a, CurvePoint>,
    first: Option<&'a CurvePoint>,
    is_closed: bool,
}

impl<'a> CurveIter<'a> {
    fn new(iter: std::slice::Iter<'a, CurvePoint>, is_closed: bool) -> Self {
        Self {
            iter,
            first: None,
            is_closed,
        }
    }
}

impl<'a> Iterator for CurveIter<'a> {
    type Item = &'a CurvePoint;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pp) = self.iter.next() {
            if self.is_closed && self.first.is_none() {
                self.first = Some(pp);
            }
            Some(pp)
        } else {
            self.first.take()
        }
    }
}

impl ExactSizeIterator for CurveIter<'_> {
    fn len(&self) -> usize {
        let base_len = self.iter.len();
        if base_len == 0 {
            self.first.is_some() as usize
        } else {
            base_len + self.is_closed as usize
        }
    }
}

struct FlatCurveIter<'a> {
    iter: CurveIter<'a>,
    buffer: [na::Vector2<f32>; 3],
    offset: u8,
}

impl<'a> FlatCurveIter<'a> {
    fn new(iter: CurveIter<'a>) -> Self {
        Self {
            iter,
            buffer: [na::Vector2::zeros(); 3],
            offset: 3,
        }
    }
}

impl<'a> Iterator for FlatCurveIter<'a> {
    type Item = na::Vector2<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.offset <= 3);
        if self.offset == 3 {
            let item = self.iter.next()?;
            self.buffer = [item.p + item.c_in, item.p, item.p + item.c_out];
            self.offset = 0;
        }
        let idx = self.offset;
        self.offset += 1;
        Some(self.buffer[idx as usize])
    }
}

impl ExactSizeIterator for FlatCurveIter<'_> {
    fn len(&self) -> usize {
        (self.iter.len() + 1) * 3 - self.offset as usize
    }
}

struct SplineWindows<'a> {
    is_first: bool,
    iter: FlatCurveIter<'a>,
    buffer: [na::Vector2<f32>; 4],
}

impl<'a> SplineWindows<'a> {
    fn new(iter: FlatCurveIter<'a>) -> Self {
        Self {
            is_first: true,
            iter,
            buffer: [na::Vector2::zeros(); 4],
        }
    }
}

impl<'a> Iterator for SplineWindows<'a> {
    type Item = [na::Vector2<f32>; 4];

    fn next(&mut self) -> Option<Self::Item> {
        // skip from `c_in` to `p`
        if self.is_first {
            _ = self.iter.next();
            self.is_first = false;
            self.buffer[3] = self.iter.next()?;
        }
        self.buffer[0] = self.buffer[3];
        self.buffer[1] = self.iter.next()?;
        self.buffer[2] = self.iter.next()?;
        self.buffer[3] = self.iter.next()?;
        Some(self.buffer.clone())
    }
}

impl ExactSizeIterator for SplineWindows<'_> {
    fn len(&self) -> usize {
        self.iter.iter.len() - 1
    }
}

const SPLINE_DIVISIONS_PER_SEGMENT: u16 = 40;

#[allow(non_snake_case)]
struct Subdivided<'a> {
    iter: SplineWindows<'a>,
    i: u16,
    G: na::Matrix4x2<f32>,
}

impl<'a> Subdivided<'a> {
    fn new(iter: SplineWindows<'a>) -> Self {
        Self {
            iter,
            i: SPLINE_DIVISIONS_PER_SEGMENT,
            G: na::Matrix4x2::zeros(),
        }
    }
}

impl<'a> Iterator for Subdivided<'a> {
    type Item = na::Vector2<f32>;

    #[allow(non_snake_case)]
    fn next(&mut self) -> Option<Self::Item> {
        const STEP: f32 = 1.0 / SPLINE_DIVISIONS_PER_SEGMENT as f32;
        const M_B: na::Matrix4<f32> = na::Matrix4::new(
            -1.0,  3.0, -3.0,  1.0,
             3.0, -6.0,  3.0,  0.0,
            -3.0,  3.0,  0.0,  0.0,
             1.0,  0.0,  0.0,  0.0,
        );
        if self.i == SPLINE_DIVISIONS_PER_SEGMENT {
            let rows = self.iter.next()?.map(|v| v.transpose());
            self.G = na::Matrix::from_rows(&rows);
            self.i = 0;
        }
        let t = self.i as f32 * STEP;
        self.i += 1;
        let T = na::Matrix1x4::new(t*t*t, t*t, t, 1.0);
        Some((T*M_B*self.G).transpose())
    }
}

impl ExactSizeIterator for Subdivided<'_> {
    fn len(&self) -> usize {
        (self.iter.len() + 1) * SPLINE_DIVISIONS_PER_SEGMENT as usize - self.i as usize
    }
}

#[derive(Debug, Clone, Default)]
pub struct Curve {
    pub points: Vec<CurvePoint>,
    pub is_closed: bool,
}

pub type StrongCurve =  Arc<ReentrantMutex<RefCell<Curve>>>;
pub type WeakCurve   = Weak<ReentrantMutex<RefCell<Curve>>>;

impl From<Rectangle> for Curve {
    fn from(Rectangle { x, y, width, height }: Rectangle) -> Self {
        let left   = x;
        let top    = y;
        let right  = x + width;
        let bottom = y + height;
        Self {
            points: vec![
                CurvePoint { c_in: na::Vector2::zeros(), p: na::Vector2::new( left,    top), c_out: na::Vector2::zeros() },
                CurvePoint { c_in: na::Vector2::zeros(), p: na::Vector2::new(right,    top), c_out: na::Vector2::zeros() },
                CurvePoint { c_in: na::Vector2::zeros(), p: na::Vector2::new(right, bottom), c_out: na::Vector2::zeros() },
                CurvePoint { c_in: na::Vector2::zeros(), p: na::Vector2::new( left, bottom), c_out: na::Vector2::zeros() },
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

    fn iter(&self) -> CurveIter<'_> {
        CurveIter::new(self.points.iter(), self.is_closed)
    }

    /// Flattens `[{c,p,c}, {c,p,c}, ...]` into `[c,p,c, c,p,c, ...]`
    fn spline(&self) -> FlatCurveIter<'_> {
        FlatCurveIter::new(self.iter())
    }

    /// Iterator over windows; `[{c,p,c}, {c,p,c}, ...]` into `[c,p,c, c,p,c, ...]` into `[[p1,c2,c3,p4], [p4,c5,c6,p7], ...]`
    fn spline_windows(&self) -> SplineWindows<'_>  {
        SplineWindows::new(self.spline())
    }

    /// Converts [{c,p,c}, {c,p,c}, ...] into [v,v,v,v,v,v,...]
    fn subdivided(&self) -> Subdivided<'_> {
        Subdivided::new(self.spline_windows())
    }

    // fn polygonize(&self) -> Box<[na::Vector2<f32>]> {
    //     let polygon;
    //     for curve_point in &self.points {
    //         curve_point.p
    //     }
    //     polygon
    // }

    pub fn triangulate(&self) -> Box<[na::Vector2<f32>]> {
        todo!()
    }
}

#[macro_export]
macro_rules! make_curve_point {
    ([$x_in:expr, $y_in:expr] ($x:expr, $y:expr) [$x_out:expr, $y_out:expr]) => {
        $crate::curve::CurvePoint {
            c_in:  $crate::na::Vector2::new($x_in as f32, $y_in as f32),
            p:     $crate::na::Vector2::new($x as f32, $y as f32),
            c_out: $crate::na::Vector2::new($x_out as f32, $y_out as f32),
        }
    };
    (($x:expr, $y:expr) [$x_out:expr, $y_out:expr]) => {
        $crate::make_curve_point!([0, 0] ($x, $y) [$x_out, $y_out])
    };
    ([$x_in:expr, $y_in:expr] ($x:expr, $y:expr)) => {
        $crate::make_curve_point!([$x_in, $y_in] ($x, $y) [0, 0])
    };
    (($x:expr, $y:expr)) => {
        $crate::make_curve_point!([0, 0] ($x, $y) [0, 0])
    };
}

#[macro_export]
macro_rules! make_curve {
    ($($([$x_in:expr, $y_in:expr])? ($x:expr, $y:expr) $([$x_out:expr, $y_out:expr])?)->* -> cycle) => {
        $crate::curve::Curve {
            points: vec![$(
                $crate::make_curve_point!(
                    $([$x_in, $y_in])?
                    ($x, $y)
                    $([$x_out, $y_out])?
                ),
            )*],
            is_closed: true,
        }
    };
    ($($([$x_in:expr, $y_in:expr])? ($x:expr, $y:expr) $([$x_out:expr, $y_out:expr])?)->*) => {
        $crate::curve::Curve {
            points: vec![$(
                $crate::make_curve_point!(
                    $([$x_in, $y_in])?
                    ($x, $y)
                    $([$x_out, $y_out])?
                ),
            )*],
            is_closed: false,
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::make_curve;

    static RL_MUX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn rl_lock() -> std::sync::MutexGuard<'static, ()> {
        RL_MUX.lock().unwrap_or_else(|x| { RL_MUX.clear_poison(); x.into_inner() })
    }

    macro_rules! vector_arr {
        ($(($x:expr,$y:expr)),* $(,)?) => {
            [$(na::Vector2::new($x as f32, $y as f32)),*]
        };
    }

    #[test]
    fn test_curve_iter() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);

        let iter = curve.iter();
        assert_eq!(iter.len(), curve.points.len());

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len());

        assert_eq!(points[0], &make_curve_point!([0,1](2,3)[4,5]));
        assert_eq!(points[1], &make_curve_point!([6,7](8,9)[10,11]));
        assert_eq!(points[2], &make_curve_point!([12,13](14,15)[16,17]));
    }

    #[test]
    fn test_curve_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);

        let iter = curve.iter();
        assert_eq!(iter.len(), curve.points.len() + 1);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len() + 1);

        assert_eq!(points[0], &make_curve_point!([0,1](2,3)[4,5]));
        assert_eq!(points[1], &make_curve_point!([6,7](8,9)[10,11]));
        assert_eq!(points[2], &make_curve_point!([12,13](14,15)[16,17]));
        assert_eq!(points[3], &make_curve_point!([0,1](2,3)[4,5]));
    }

    #[test]
    fn test_spline_iter() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);

        let iter = curve.spline();
        assert_eq!(iter.len(), curve.points.len() * 3);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len() * 3);

        assert_eq!(&points[..], &vector_arr![(2+0,3+1),(2,3),(2+4,3+5),(8+6,9+7),(8,9),(8+10,9+11),(14+12,15+13),(14,15),(14+16,15+17)]);
    }

    #[test]
    fn test_spline_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);

        let iter = curve.spline();
        assert_eq!(iter.len(), (curve.points.len() + 1) * 3);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), (curve.points.len() + 1) * 3);

        assert_eq!(&points[..], &vector_arr![(2+0,3+1),(2,3),(2+4,3+5),(8+6,9+7),(8,9),(8+10,9+11),(14+12,15+13),(14,15),(14+16,15+17),(2+0,3+1),(2,3),(2+4,3+5)]);
    }

    #[test]
    fn test_spline_windows_iter() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);

        let iter = curve.spline_windows();
        assert_eq!(iter.len(), curve.points.len() - 1);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len() - 1);

        assert_eq!(&points[0], &vector_arr![(2,3),(2+4,3+5),(8+6,9+7),(8,9)]);
        assert_eq!(&points[1], &vector_arr![(8,9),(8+10,9+11),(14+12,15+13),(14,15)]);
    }

    #[test]
    fn test_spline_windows_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);

        let iter = curve.spline_windows();
        assert_eq!(iter.len(), curve.points.len());

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len());

        assert_eq!(&points[0], &vector_arr![(2,3),(2+4,3+5),(8+6,9+7),(8,9)]);
        assert_eq!(&points[1], &vector_arr![(8,9),(8+10,9+11),(14+12,15+13),(14,15)]);
        assert_eq!(&points[2], &vector_arr![(14,15),(14+16,15+17),(2+0,3+1),(2,3)]);
    }

    #[test]
    fn test_subdivided_iter() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);
        let expected_count = (curve.points.len() - 1) * SPLINE_DIVISIONS_PER_SEGMENT as usize;

        let iter = curve.subdivided();
        assert_eq!(iter.len(), expected_count);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), expected_count);
    }

    #[test]
    fn test_subdivided_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);
        let expected_count = (curve.points.len()) * SPLINE_DIVISIONS_PER_SEGMENT as usize;

        let iter = curve.subdivided();
        assert_eq!(iter.len(), expected_count);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), expected_count);
    }

    #[test]
    fn test_subdivided_iter_positions() {
        let mut success = false;
        {
            let _lock = rl_lock();
            let (mut rl, thread) = init()
                .title("test_subdivided_iter")
                .build();
            rl.set_target_fps(60);
            let curve = make_curve!([-50,0](60,300)[50,0]->[-50,0](320,100)[50,0]->[-50,0](580,300)[50,0]);
            let mut buffer_actual = Vec::new();
            while !rl.window_should_close() {

                buffer_actual.clear();
                buffer_actual.extend(curve.subdivided().map(Vector2::from));

                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::RAYWHITE);

                // draw expected
                d.draw_spline_segment_bezier_cubic(
                    Vector2::from(curve.points[0].p),
                    Vector2::from(curve.points[0].p + curve.points[0].c_out),
                    Vector2::from(curve.points[1].p + curve.points[1].c_in),
                    Vector2::from(curve.points[1].p),
                    5.0,
                    Color::GREEN.alpha(0.5),
                );
                d.draw_spline_segment_bezier_cubic(
                    Vector2::from(curve.points[1].p),
                    Vector2::from(curve.points[1].p + curve.points[1].c_out),
                    Vector2::from(curve.points[2].p + curve.points[2].c_in),
                    Vector2::from(curve.points[2].p),
                    5.0,
                    Color::GREEN.alpha(0.5),
                );

                // draw actual
                d.draw_line_strip(&buffer_actual[..], Color::MAGENTA);

                for point in &curve.points {
                    let p = Vector2::from(point.p);
                    let p_in = Vector2::from(point.p + point.c_in);
                    let p_out = Vector2::from(point.p + point.c_out);
                    d.draw_line_v(p, p_in, Color::GRAY.alpha(0.5));
                    d.draw_line_v(p, p_out, Color::GRAY.alpha(0.5));
                    d.draw_ring(p, 9.0, 11.0, 0.0, 360.0, 30, Color::RED);
                    d.draw_ring(p_in,  4.0, 6.0, 0.0, 360.0, 20, Color::BLUE);
                    d.draw_ring(p_out, 4.0, 6.0, 0.0, 360.0, 20, Color::BLUE);
                }

                d.draw_rectangle_rec(Rectangle::new(2.0, 2.0, 54.0, 21.0), if success { Color::GREEN } else { Color::RED }.alpha(0.5));
                d.gui_check_box(Rectangle::new(5.0, 5.0, 15.0, 15.0), Some(if success { c"pass" } else { c"fail" }), &mut success);
            }
        }
        assert!(success, "manually failed");
    }

    // #[test]
    // fn polygon_test0() {
    //     let curve = make_curve!([1, 5] (5, 3) -> (3, 3) -> cycle);

    //     println!("{curve:#?}");

    //     // let polygon = curve.polygonize();

    //     // println!("{polygon:?}");
    // }
}
