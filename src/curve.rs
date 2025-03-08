use std::{cell::RefCell, sync::{Arc, Weak}};
use parking_lot::ReentrantMutex;
use raylib::prelude::*;

/// A point in a [`Curve`]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CurvePoint {
    /// Entry velocity
    ///
    /// Relative to `p`
    pub c_in: na::Vector2<f32>,

    /// Anchor position
    pub p: na::Vector2<f32>,

    /// Entry velocity
    ///
    /// Relative to `p`
    pub c_out: na::Vector2<f32>,
}

pub struct CurveIter<'a> {
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

    /// Flatten [`CurvePoint`]s into an array of vectors
    ///
    /// `[{c,p,c}, {c,p,c}, ...]` into `[c,p,c, c,p,c, ...]`
    pub fn spline(self) -> FlatCurveIter<'a> {
        FlatCurveIter::new(self)
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let base_len = self.iter.len();
        let len = if base_len == 0 {
            self.first.is_some() as usize
        } else {
            base_len + self.is_closed as usize
        };
        (len, Some(len))
    }
}

impl ExactSizeIterator for CurveIter<'_> {}

pub struct FlatCurveIter<'a> {
    iter: CurveIter<'a>,
    buffer: [na::Vector2<f32>; 3],
    offset: u8,
}

impl<'a> FlatCurveIter<'a> {
    fn new(iter: CurveIter<'a>) -> Self {
        Self {
            iter,
            buffer: Default::default(),
            offset: 3,
        }
    }

    /// Group cubic bezier points into segments with overlapping anchor points
    ///
    /// `[c,p,c, c,p,c, ...]` into `[[p1,c2,c3,p4], [p4,c5,c6,p7], ...]`
    pub fn spline_windows(self) -> SplineWindows<'a>  {
        SplineWindows::new(self)
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.iter.len() + 1) * 3 - self.offset as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for FlatCurveIter<'_> {}

pub struct SplineWindows<'a> {
    is_first: bool,
    iter: FlatCurveIter<'a>,
    buffer: [na::Vector2<f32>; 4],
}

impl<'a> SplineWindows<'a> {
    fn new(iter: FlatCurveIter<'a>) -> Self {
        Self {
            is_first: true,
            iter,
            buffer: Default::default(),
        }
    }

    /// Subdivide segments into `RES` evenly-separated t-values `[0.0..1.0]` and bezier indices
    ///
    /// `[[p1,c2,c3,p4], [p4,c5,c6,p7], ...]` into `[(0, 0.0), (0, 1/RES), (0, 2/RES), ..., (n-1, (RES-1)/RES)]`
    pub fn sampled<const RES: u16>(self) -> Sampled<'a, RES> {
        Sampled::new(self)
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.iter.len().saturating_sub(1);
        (len, Some(len))
    }
}

impl ExactSizeIterator for SplineWindows<'_> {}

pub struct Sampled<'a, const RES: u16> {
    iter: SplineWindows<'a>,
    mat: na::Matrix2x4<f32>,
    spline_index: u32,
    is_initialized: bool,
    segment: u16,
}

impl<'a, const RES: u16> Sampled<'a, RES> {
    pub const RESOLUTION: u16 = RES;
    pub const STEP: f32 = 1.0 / Self::RESOLUTION as f32;

    fn new(iter: SplineWindows<'a>) -> Self {
        Self {
            iter,
            mat: Default::default(),
            spline_index: 0,
            is_initialized: false,
            segment: Self::RESOLUTION,
        }
    }

    /// Get the buffered matrix of bezier control points
    ///
    /// The buffer reflects the state of the current iteration
    /// (whatever was most recently returned by [`Sampled::next()`])
    fn mat(&self) -> &na::Matrix2x4<f32> {
        &self.mat
    }
}

impl<'a, const RES: u16> Iterator for Sampled<'a, RES> {
    type Item = (u32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.segment == Self::RESOLUTION {
            if self.is_initialized {
                self.spline_index += 1;
            } else {
                self.is_initialized = true;
            }
            self.segment = 0;
            let vecs = self.iter.next()?;
            self.mat = na::Matrix::from_columns(&vecs);
        }
        let t = self.segment as f32 * Self::STEP;
        self.segment += 1;
        Some((self.spline_index, t))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.iter.len() + 1) * Self::RESOLUTION as usize - self.segment as usize;
        (len, Some(len))
    }
}

impl<const RES: u16> ExactSizeIterator for Sampled<'_, RES> {}

trait SamplingHelper: Sized + Iterator {
    const RES: u16;
    type Sampled: ExactSizeIterator<Item = (u32, f32)>;

    /// ```
    /// [[x1, x2, x3, x4]
    ///  [y1, y2, y3, y4]]
    /// ```
    fn mat(&self) -> &na::Matrix2x4<f32>;

    /// Get the output of the latest [`Sampled`] in the chain,
    /// regardless of iterator nesting
    fn item_sample(item: &Self::Item) -> <Self::Sampled as Iterator>::Item;
}

impl<'a, const RES: u16> SamplingHelper for Sampled<'a, RES> {
    const RES: u16 = RES;
    type Sampled = Self;

    #[inline]
    fn mat(&self) -> &na::Matrix2x4<f32> {
        self.mat()
    }

    #[inline]
    fn item_sample(item: &Self::Item) -> <Self::Sampled as Iterator>::Item {
        *item
    }
}

#[allow(private_bounds)]
pub trait Sampling: SamplingHelper {
    /// Calculate the position alongside each sample
    #[inline]
    fn with_positions(self) -> Positions<Self> {
        Positions::new(self)
    }

    /// Calculate the velocity alongside each sample
    #[inline]
    fn with_velocities(self) -> Velocities<Self> {
        Velocities::new(self)
    }
}

impl<I: SamplingHelper> Sampling for I {}

#[allow(non_snake_case)]
pub struct Positions<I> {
    iter: I,
}

impl<I> Positions<I> {
    fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: SamplingHelper> Iterator for Positions<I> {
    type Item = (I::Item, na::Vector2<f32>);

    #[allow(non_snake_case)]
    fn next(&mut self) -> Option<Self::Item> {
        const P_COEFS: na::Matrix4<f32> = na::Matrix4::new(
            -1.0,  3.0, -3.0,  1.0,
             3.0, -6.0,  3.0,  0.0,
            -3.0,  3.0,  0.0,  0.0,
             1.0,  0.0,  0.0,  0.0,
        );
        let item = self.iter.next()?;
        let (_, t) = I::item_sample(&item);
        let T = na::Vector4::new(t*t*t, t*t, t, 1.0);
        Some((item, (self.mat()*P_COEFS*T)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: SamplingHelper> ExactSizeIterator for Positions<I> {}

impl<I: SamplingHelper> SamplingHelper for Positions<I> {
    const RES: u16 = I::RES;
    type Sampled = I::Sampled;

    #[inline]
    fn mat(&self) -> &na::Matrix2x4<f32> {
        self.iter.mat()
    }

    #[inline]
    fn item_sample(item: &Self::Item) -> <Self::Sampled as Iterator>::Item {
        I::item_sample(&item.0)
    }
}

pub struct Velocities<I> {
    iter: I,
}

impl<I> Velocities<I> {
    fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: SamplingHelper> Iterator for Velocities<I> {
    type Item = (I::Item, na::Vector2<f32>);

    #[allow(non_snake_case)]
    fn next(&mut self) -> Option<Self::Item> {
        const V_COEFS: na::Matrix4x3<f32> = na::Matrix4x3::new(
            -3.0,   6.0, -3.0,
             9.0, -12.0,  3.0,
            -9.0,   6.0,  0.0,
             3.0,   0.0,  0.0,
        );
        let item = self.iter.next()?;
        let (_, t) = I::item_sample(&item);
        let T = na::Vector3::new(t*t, t, 1.0);
        Some((item, self.mat()*V_COEFS*T))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: SamplingHelper> SamplingHelper for Velocities<I> {
    const RES: u16 = I::RES;
    type Sampled = I::Sampled;

    #[inline]
    fn mat(&self) -> &na::Matrix2x4<f32> {
        self.iter.mat()
    }

    #[inline]
    fn item_sample(item: &Self::Item) -> <Self::Sampled as Iterator>::Item {
        I::item_sample(&item.0)
    }
}

/// A collection of cubic bezier curve patches.
#[derive(Debug, Clone, Default)]
pub struct Curve {
    /// The array of bezier control points
    ///
    /// `[{c,p,c}, {c,p,c}, ...]`
    pub points: Vec<CurvePoint>,

    /// Whether the curve is a closed loop \
    /// "is the tail connected to the tip?"
    ///
    /// The tip and tail **don't need** to be at
    /// the same position, and preferrably aren't
    pub is_closed: bool,
}

pub type StrongCurve =  Arc<ReentrantMutex<RefCell<Curve>>>;
pub type WeakCurve   = Weak<ReentrantMutex<RefCell<Curve>>>;

impl From<Rectangle> for Curve {
    /// Construct a curve that matches the shape and position of a rectangle
    ///
    /// Useful for creating raster paths
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

    /// Iterate over points in a [`Curve`]
    ///
    /// Includes the first point a second time,
    /// after the last point, if the curve is closed.
    pub fn iter(&self) -> CurveIter<'_> {
        CurveIter::new(self.points.iter(), self.is_closed)
    }

    /// Convenience method for
    ///
    /// 1. [.`iter()`](`Curve::iter`)
    /// 2. [.`spline()`](`CurveIter::spline`)
    /// 3. [.`spline_windows()`](`FlatCurveIter::spline_windows`)
    /// 4. [.`sampled()`](`SplineWindows::sampled`)
    #[inline]
    pub fn sampled_iter<const RES: u16>(&self) -> Sampled<'_, RES> {
        self.iter()
            .spline()
            .spline_windows()
            .sampled::<RES>()
    }

    /// Get an iterator over positions and velocities
    ///
    /// Convenience method for
    ///
    /// 1. [.`iter()`](`Curve::iter`)
    /// 2. [.`spline()`](`CurveIter::spline`)
    /// 3. [.`spline_windows()`](`FlatCurveIter::spline_windows`)
    /// 4. [.`sampled()`](`SplineWindows::sampled`)
    /// 5. [.`with_positions()`](`Sampled::with_positions`)
    /// 6. [.`with_velocities()`](`Sampled::with_velocities`)
    #[inline]
    pub fn pos_vel_iter<const RES: u16>(&self) -> std::iter::Map<
        Velocities<Positions<Sampled<'_, RES>>>,
        impl FnMut((((u32, f32), na::Vector2<f32>), na::Vector2<f32>)) -> (u32, f32, na::Vector2<f32>, na::Vector2<f32>),
    > {
        self.iter()
            .spline()
            .spline_windows()
            .sampled::<RES>()
            .with_positions()
            .with_velocities()
            .map(|(((i, t), p), v)| (i, t, p, v))
    }
}

/// Construct a [`CurvePoint`] using Tikz-inspired syntax
///
/// - `(..., ...)` - Anchor point (mandatory)
/// - `[..., ...]` - Velocity control (optional - defaults to 0,0)
///
/// # Example
/// ```
/// # use crate::make_curve;
/// # use crate::na;
/// let pp = make_curve_point!([0,1] (2,3) [4,5]);
/// assert_eq!(pp.c_in,  na::Vector2::new(0.0, 1.0));
/// assert_eq!(pp.p,     na::Vector2::new(2.0, 3.0));
/// assert_eq!(pp.c_out, na::Vector2::new(4.0, 5.0));
///
/// let pp = make_curve_point!((2,3) [4,5]);
/// assert_eq!(pp.c_in, na::Vector2::new(0.0, 0.0));
///
/// let pp = make_curve_point!([0,1] (2,3));
/// assert_eq!(pp.c_out, na::Vector2::new(0.0, 0.0));
/// ```
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

/// Construct a [`Curve`] using Tikz-inspired syntax
///
/// - `(..., ...)` - Anchor point (mandatory)
/// - `[..., ...]` - Velocity control (optional - defaults to 0,0)
/// - `->` - Separator between controls
/// - `cycle` - Curve is a closed loop (only valid at end)
///
/// # Example
/// ```
/// # use crate::make_curve;
/// let curve = make_curve!([0,1] (2,3) [4,5] -> [6,7] (8,9) [10,11] -> [12,13] (14,15) [16,17]);
/// assert_eq!(curve.points, &[
///     make_curve_point!([ 0, 1] ( 2, 3) [ 4, 5]),
///     make_curve_point!([ 6, 7] ( 8, 9) [10,11]),
///     make_curve_point!([12,13] (14,15) [16,17]),
/// ]);
/// assert!(!curve.is_closed);
/// ```
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

    /// Ensure only one visual test can be open at a time
    fn rl_lock() -> std::sync::MutexGuard<'static, ()> {
        static RL_MUX: std::sync::Mutex<()> = std::sync::Mutex::new(());
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

        let iter = curve.iter().spline();
        assert_eq!(iter.len(), curve.points.len() * 3);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len() * 3);

        assert_eq!(&points[..], &vector_arr![(2+0,3+1),(2,3),(2+4,3+5),(8+6,9+7),(8,9),(8+10,9+11),(14+12,15+13),(14,15),(14+16,15+17)]);
    }

    #[test]
    fn test_spline_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);

        let iter = curve.iter().spline();
        assert_eq!(iter.len(), (curve.points.len() + 1) * 3);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), (curve.points.len() + 1) * 3);

        assert_eq!(&points[..], &vector_arr![(2+0,3+1),(2,3),(2+4,3+5),(8+6,9+7),(8,9),(8+10,9+11),(14+12,15+13),(14,15),(14+16,15+17),(2+0,3+1),(2,3),(2+4,3+5)]);
    }

    #[test]
    fn test_spline_windows_iter() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);

        let iter = curve.iter().spline().spline_windows();
        assert_eq!(iter.len(), curve.points.len() - 1);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len() - 1);

        assert_eq!(&points[0], &vector_arr![(2,3),(2+4,3+5),(8+6,9+7),(8,9)]);
        assert_eq!(&points[1], &vector_arr![(8,9),(8+10,9+11),(14+12,15+13),(14,15)]);
    }

    #[test]
    fn test_spline_windows_iter_cyclic() {
        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);

        let iter = curve.iter().spline().spline_windows();
        assert_eq!(iter.len(), curve.points.len());

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), curve.points.len());

        assert_eq!(&points[0], &vector_arr![(2,3),(2+4,3+5),(8+6,9+7),(8,9)]);
        assert_eq!(&points[1], &vector_arr![(8,9),(8+10,9+11),(14+12,15+13),(14,15)]);
        assert_eq!(&points[2], &vector_arr![(14,15),(14+16,15+17),(2+0,3+1),(2,3)]);
    }

    #[test]
    fn test_sampled_iter() {
        const RES: u16 = 4;

        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);
        let expected_count = (curve.points.len() - 1) * RES as usize;

        let iter = curve.iter().spline().spline_windows().sampled::<RES>();
        assert_eq!(iter.len(), expected_count);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), expected_count);

        for (i, t) in points {
            println!("{i}, {t}");
        }
    }

    #[test]
    fn test_positions_iter() {
        const RES: u16 = 40;

        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]);
        let expected_count = (curve.points.len() - 1) * RES as usize;

        let iter = curve.iter().spline().spline_windows().sampled::<RES>().with_positions();
        assert_eq!(iter.len(), expected_count);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), expected_count);
    }

    #[test]
    fn test_positions_iter_cyclic() {
        const RES: u16 = 40;

        let curve = make_curve!([0,1](2,3)[4,5]->[6,7](8,9)[10,11]->[12,13](14,15)[16,17]->cycle);
        let expected_count = (curve.points.len()) * RES as usize;

        let iter = curve.iter().spline().spline_windows().sampled::<RES>().with_positions();
        assert_eq!(iter.len(), expected_count);

        let points = iter.collect::<Vec<_>>();
        assert_eq!(points.len(), expected_count);
    }

    #[test]
    fn test_positions_iter_vis() {
        const RES: u16 = 40;

        let mut success = false;
        {
            let _lock = rl_lock();
            let (mut rl, thread) = init()
                .title("test_positions_iter_vis")
                .build();
            rl.set_target_fps(60);
            let curve = make_curve!([-50,0](60,300)[50,0]->[-50,0](320,100)[50,0]->[-50,0](580,300)[50,0]);
            let mut positions_actual = Vec::new();
            while !rl.window_should_close() {

                positions_actual.clear();
                for (_, p) in curve.iter().spline().spline_windows().sampled::<RES>().with_positions() {
                    positions_actual.push(Vector2::from(p));
                }

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
                d.draw_line_strip(&positions_actual[..], Color::MAGENTA);

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

                d.draw_rectangle_rec(Rectangle::new(2.0, 2.0, 21.0, 21.0), if success { Color::GREEN } else { Color::RED }.alpha(0.5));
                d.gui_check_box(Rectangle::new(5.0, 5.0, 15.0, 15.0), None, &mut success);
            }
        }
        assert!(success, "test failed");
    }

    #[test]
    fn test_velocities_iter_vis() {
        const RES: u16 = 40;

        let mut success = false;
        {
            let _lock = rl_lock();
            let (mut rl, thread) = init()
                .title("test_velocities_iter_vis")
                .build();
            rl.set_target_fps(60);
            let curve = make_curve!([-50,0](60,300)[50,0]->[-50,0](320,100)[50,0]->[-50,0](580,300)[50,0]);
            let mut positions_actual = Vec::new();
            let mut velocities_actual = Vec::new();
            while !rl.window_should_close() {
                positions_actual.clear();
                velocities_actual.clear();
                for ((_, p), v) in curve.iter().spline().spline_windows().sampled::<RES>().with_positions().with_velocities() {
                    positions_actual.push(Vector2::from(p));
                    velocities_actual.push(Vector2::from(v));
                }

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
                d.draw_line_strip(&positions_actual[..], Color::MAGENTA);
                for (p, v) in positions_actual.iter().zip(velocities_actual.iter()) {
                    d.draw_line_v(p, *p + *v, Color::ORANGE);
                }

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

                d.draw_rectangle_rec(Rectangle::new(2.0, 2.0, 21.0, 21.0), if success { Color::GREEN } else { Color::RED }.alpha(0.5));
                d.gui_check_box(Rectangle::new(5.0, 5.0, 15.0, 15.0), None, &mut success);
            }
        }
        assert!(success, "test failed");
    }
}
