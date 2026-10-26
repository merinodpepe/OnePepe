//! OnePepe: recognition of hand drawn shapes ("ink to shape").
//!
//! Given the points of a finished stroke, tries to fit a line, circle, ellipse, triangle or rectangle.
//! When nothing fits clearly, nothing is returned and the stroke should be kept as it is.
//!
//! Independent of the UI, so it can be tested with synthetic strokes.

// Imports
use crate::shapes::{Ellipse, Line, Polygon, Rectangle, Shape};
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use std::f64::consts::PI;

/// Parameters for the recognition.
#[derive(Debug, Clone, Copy)]
pub struct RecognitionParams {
    /// How permissive the recognition is, from 0.0 (strict) to 1.0 (permissive).
    pub sensitivity: f64,
    /// Strokes with a smaller bounding box diagonal are never recognized (avoids converting handwriting).
    pub min_size: f64,
}

impl Default for RecognitionParams {
    fn default() -> Self {
        Self {
            sensitivity: 0.5,
            min_size: 50.0,
        }
    }
}

/// A recognized shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecognizedShape {
    /// A straight line.
    Line {
        /// Start.
        start: Vector2,
        /// End.
        end: Vector2,
    },
    /// A circle.
    Circle {
        /// Center.
        center: Vector2,
        /// Radius.
        radius: f64,
    },
    /// An ellipse.
    Ellipse {
        /// Center.
        center: Vector2,
        /// Radii along its own axes.
        radii: Vector2,
        /// Rotation angle in radians.
        angle: f64,
    },
    /// A triangle.
    Triangle([Vector2; 3]),
    /// A (possibly rotated) rectangle.
    Rectangle {
        /// Center.
        center: Vector2,
        /// Half extents along its own axes.
        half_extents: Vector2,
        /// Rotation angle in radians.
        angle: f64,
    },
}

impl RecognizedShape {
    /// A short name, for tests and debugging.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Line { .. } => "line",
            Self::Circle { .. } => "circle",
            Self::Ellipse { .. } => "ellipse",
            Self::Triangle(_) => "triangle",
            Self::Rectangle { .. } => "rectangle",
        }
    }

    /// Converts into a shape that can be drawn.
    pub fn to_shape(&self) -> Shape {
        match *self {
            Self::Line { start, end } => Shape::Line(Line::new(start, end)),
            Self::Circle { center, radius } => Shape::Ellipse(Ellipse {
                radii: Vector2::splat(radius),
                affine: DAffine2::from_translation(center),
            }),
            Self::Ellipse {
                center,
                radii,
                angle,
            } => Shape::Ellipse(Ellipse {
                radii,
                affine: DAffine2::from_angle_translation(angle, center),
            }),
            Self::Triangle([a, b, c]) => Shape::Polygon(Polygon {
                start: a,
                path: vec![b, c, a],
            }),
            Self::Rectangle {
                center,
                half_extents,
                angle,
            } => {
                let mut rect = Rectangle::from_half_extents(Vector2::ZERO, half_extents);
                rect.affine = DAffine2::from_angle_translation(angle, center);
                Shape::Rectangle(rect)
            }
        }
    }
}

/// Number of points the stroke is resampled to.
const N_RESAMPLE: usize = 128;

/// Tries to recognize a shape from the points of a stroke.
pub fn recognize(points: &[Vector2], params: RecognitionParams) -> Option<RecognizedShape> {
    recognize_w_reason(points, params).ok()
}

/// Like [`recognize`], but tells why nothing was recognized.
pub fn recognize_w_reason(
    points: &[Vector2],
    params: RecognitionParams,
) -> Result<RecognizedShape, &'static str> {
    let points = dedup(points);
    if points.len() < 5 {
        return Err("too few points");
    }
    let (mins, maxs) = bounds(&points);
    let diag = (maxs - mins).length();
    if diag < params.min_size.max(f64::EPSILON) {
        return Err("too small");
    }
    // tolerance factor, between 0.6 (strict) and 1.4 (permissive)
    let k = 0.6 + 0.8 * params.sensitivity.clamp(0.0, 1.0);

    // resample and smooth away the jitter of the hand / the tablet
    let path = smooth_open(&resample(&points, N_RESAMPLE), 2);
    let length = polyline_length(&resample(&path, 32));
    let gap = (path[0] - path[path.len() - 1]).length();

    if gap > 0.25 * length {
        return recognize_line(&path, length, k).ok_or("open, but not a line");
    }

    // closed: cut away the overshoot at the end. It must follow the outline, else it is a tail (like in an "a").
    let (closed, overshoot) = trim_overshoot(&path);
    if overshoot
        .iter()
        .any(|p| dist_to_closed_polyline(*p, &closed) > 0.06 * k * diag)
    {
        return Err("closed, but with a tail");
    }
    let (abs_turn, net_turn) = turning(&resample_closed(&closed, 24), true);
    // shapes go around once; letters like "8", "e" or scribbles turn back or around several times
    if net_turn.abs() < 1.5 * PI {
        return Err("closed, but does not go around");
    }
    if abs_turn > (2.0 + 0.9 * k) * PI {
        return Err("closed, but turns too much");
    }
    let loop_pts = resample_closed(&closed, N_RESAMPLE);

    let ellipse = fit_ellipse(&loop_pts, k);
    let polygon = fit_polygon(&loop_pts, k);
    match (ellipse, polygon) {
        (Ok((e, e_score)), Ok((p, p_score))) => Ok(if e_score <= p_score { e } else { p }),
        (Ok((e, _)), Err(_)) => Ok(e),
        (Err(_), Ok((p, _))) => Ok(p),
        (Err(e), Err(p)) => Err(if e.1 <= p.1 { e.0 } else { p.0 }),
    }
}

fn recognize_line(path: &[Vector2], length: f64, k: f64) -> Option<RecognizedShape> {
    let (start, end) = (path[0], path[path.len() - 1]);
    let chord = (end - start).length();
    if chord < f64::EPSILON || chord / length < 1.0 - 0.05 * k {
        return None;
    }
    // total least squares line through the points
    let (center, dir) = principal_axis(path);
    let normal = Vector2::new(-dir.y, dir.x);
    let max_dev = path
        .iter()
        .map(|p| (*p - center).dot(normal).abs())
        .fold(0.0, f64::max);
    if max_dev / chord > 0.045 * k {
        return None;
    }
    let project = |p: Vector2| center + dir * (p - center).dot(dir);
    Some(RecognizedShape::Line {
        start: project(start),
        end: project(end),
    })
}

/// Returns the shape and a score (error relative to the tolerance, lower is better).
fn fit_ellipse(pts: &[Vector2], k: f64) -> Result<(RecognizedShape, f64), (&'static str, f64)> {
    let (mean, dir) = principal_axis(pts);
    let normal = Vector2::new(-dir.y, dir.x);
    let local = pts
        .iter()
        .map(|p| Vector2::new((*p - mean).dot(dir), (*p - mean).dot(normal)))
        .collect::<Vec<Vector2>>();
    let (lmins, lmaxs) = bounds(&local);
    let radii = (lmaxs - lmins) * 0.5;
    if radii.x < f64::EPSILON || radii.y < f64::EPSILON {
        return Err(("ellipse: degenerate", f64::MAX));
    }
    let lcenter = (lmins + lmaxs) * 0.5;
    let center = mean + dir * lcenter.x + normal * lcenter.y;

    let errors = local
        .iter()
        .map(|p| {
            let q = (*p - lcenter) / radii;
            (q.length() - 1.0).abs()
        })
        .collect::<Vec<f64>>();
    let mean_err = errors.iter().sum::<f64>() / errors.len() as f64;
    let max_err = errors.iter().copied().fold(0.0, f64::max);
    let (mean_tol, max_tol) = (0.055 * k, 0.17 * k);
    let score = (mean_err / mean_tol).max(max_err / max_tol);
    if score > 1.0 {
        return Err(if mean_err > mean_tol {
            ("ellipse: mean error", score)
        } else {
            ("ellipse: max error", score)
        });
    }
    // the points must go all the way around
    let mut bins = [false; 12];
    for p in local.iter() {
        let q = (*p - lcenter) / radii;
        let a = q.y.atan2(q.x).rem_euclid(2.0 * PI);
        bins[((a / (2.0 * PI) * 12.0) as usize).min(11)] = true;
    }
    if bins.iter().any(|b| !b) {
        return Err(("ellipse: not all around", score));
    }
    let angle = dir.y.atan2(dir.x);
    let (a, b) = (radii.x.max(radii.y), radii.x.min(radii.y));
    if b / a < 0.12 {
        return Err(("ellipse: too thin", score));
    }
    let shape = if b / a > 0.82 {
        let radius = pts.iter().map(|p| (*p - center).length()).sum::<f64>() / pts.len() as f64;
        RecognizedShape::Circle { center, radius }
    } else {
        RecognizedShape::Ellipse {
            center,
            radii,
            angle,
        }
    };
    Ok((shape, score))
}

/// Returns the shape and a score (error relative to the tolerance, lower is better).
fn fit_polygon(pts: &[Vector2], k: f64) -> Result<(RecognizedShape, f64), (&'static str, f64)> {
    let perimeter = polyline_length(pts) + (pts[0] - pts[pts.len() - 1]).length();
    let (mins, maxs) = bounds(pts);
    let diag = (maxs - mins).length();

    let mut vertices = simplify_closed(pts, 0.045 * diag);
    // remove vertices that barely turn or are too close to their neighbours
    loop {
        let n = vertices.len();
        if n <= 3 {
            break;
        }
        let (i, turn) = (0..n)
            .map(|i| {
                (
                    i,
                    turn_angle(
                        vertices[(i + n - 1) % n],
                        vertices[i],
                        vertices[(i + 1) % n],
                    ),
                )
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let shortest = (0..n)
            .map(|i| (vertices[(i + 1) % n] - vertices[i]).length())
            .fold(f64::MAX, f64::min);
        if turn < 35_f64.to_radians() {
            vertices.remove(i);
        } else if shortest < 0.07 * perimeter {
            let j = (0..n)
                .min_by(|&a, &b| {
                    let la = (vertices[(a + 1) % n] - vertices[a]).length();
                    let lb = (vertices[(b + 1) % n] - vertices[b]).length();
                    la.total_cmp(&lb)
                })
                .unwrap();
            // merge the short edge into its midpoint
            let mid = (vertices[j] + vertices[(j + 1) % n]) * 0.5;
            vertices[j] = mid;
            vertices.remove((j + 1) % n);
        } else {
            break;
        }
    }
    let vertices = refine_polygon(pts, &vertices);
    let n = vertices.len();
    if !(n == 3 || n == 4) {
        return Err(("polygon: not 3 or 4 corners", 10.0 + n as f64));
    }
    // interior angles
    let interior = (0..n)
        .map(|i| {
            PI - turn_angle(
                vertices[(i + n - 1) % n],
                vertices[i],
                vertices[(i + 1) % n],
            )
        })
        .collect::<Vec<f64>>();

    let shape = if n == 3 {
        if interior.iter().any(|&a| a < 12_f64.to_radians()) {
            return Err(("triangle: too thin", 5.0));
        }
        RecognizedShape::Triangle([vertices[0], vertices[1], vertices[2]])
    } else {
        let tol = (18.0 + 8.0 * k).to_radians();
        if interior.iter().any(|&a| (a - PI / 2.0).abs() > tol) {
            return Err(("quadrilateral: not right angles", 5.0));
        }
        fit_rectangle(&vertices)
    };

    // distance of the stroke to the outline of the fitted shape
    let outline = match shape {
        RecognizedShape::Triangle(v) => v.to_vec(),
        RecognizedShape::Rectangle {
            center,
            half_extents,
            angle,
        } => {
            let (d, nrm) = (
                Vector2::new(angle.cos(), angle.sin()),
                Vector2::new(-angle.sin(), angle.cos()),
            );
            [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                .iter()
                .map(|(sx, sy)| center + d * (sx * half_extents.x) + nrm * (sy * half_extents.y))
                .collect()
        }
        _ => unreachable!(),
    };
    let dists = pts
        .iter()
        .map(|p| dist_to_closed_polyline(*p, &outline))
        .collect::<Vec<f64>>();
    let mean_d = dists.iter().sum::<f64>() / dists.len() as f64 / perimeter;
    let max_d = dists.iter().copied().fold(0.0, f64::max) / perimeter;
    let (mean_tol, max_tol) = (0.016 * k, 0.05 * k);
    let score = (mean_d / mean_tol).max(max_d / max_tol);
    if score > 1.0 {
        return Err(if mean_d > mean_tol {
            ("polygon: mean distance", score)
        } else {
            ("polygon: max distance", score)
        });
    }
    Ok((shape, score))
}

/// Moves the corners to the intersections of lines fitted to the sides, which do not suffer from rounded corners.
fn refine_polygon(pts: &[Vector2], vertices: &[Vector2]) -> Vec<Vector2> {
    let n = vertices.len();
    if n < 3 {
        return vertices.to_vec();
    }
    let mut side_points = vec![Vec::new(); n];
    for p in pts {
        let (i, _) = (0..n)
            .map(|i| (i, dist_to_segment(*p, vertices[i], vertices[(i + 1) % n])))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let (a, b) = (vertices[i], vertices[(i + 1) % n]);
        let t = (*p - a).dot(b - a) / (b - a).length_squared().max(f64::EPSILON);
        // leave out the corners
        if (0.15..=0.85).contains(&t) {
            side_points[i].push(*p);
        }
    }
    let lines = side_points
        .iter()
        .enumerate()
        .map(|(i, sp)| {
            if sp.len() >= 3 {
                principal_axis(sp)
            } else {
                let (a, b) = (vertices[i], vertices[(i + 1) % n]);
                (a, (b - a).normalize_or_zero())
            }
        })
        .collect::<Vec<(Vector2, Vector2)>>();
    (0..n)
        .map(|i| {
            let (p0, d0) = lines[(i + n - 1) % n];
            let (p1, d1) = lines[i];
            let cross = d0.x * d1.y - d0.y * d1.x;
            if cross.abs() < 0.1 {
                return vertices[i];
            }
            let t = ((p1 - p0).x * d1.y - (p1 - p0).y * d1.x) / cross;
            let v = p0 + d0 * t;
            // never move a corner far away
            if (v - vertices[i]).length() > 0.25 * (vertices[(i + 1) % n] - vertices[i]).length() {
                vertices[i]
            } else {
                v
            }
        })
        .collect()
}

fn fit_rectangle(v: &[Vector2]) -> RecognizedShape {
    // average edge direction modulo 90 degrees, using the quadrupled angle
    let mut acc = Vector2::ZERO;
    for i in 0..4 {
        let e = v[(i + 1) % 4] - v[i];
        let a = e.y.atan2(e.x);
        acc += Vector2::new((4.0 * a).cos(), (4.0 * a).sin()) * e.length();
    }
    let angle = acc.y.atan2(acc.x) / 4.0;
    let (d, nrm) = (
        Vector2::new(angle.cos(), angle.sin()),
        Vector2::new(-angle.sin(), angle.cos()),
    );
    let mut us = v.iter().map(|p| p.dot(d)).collect::<Vec<f64>>();
    let mut ws = v.iter().map(|p| p.dot(nrm)).collect::<Vec<f64>>();
    us.sort_by(f64::total_cmp);
    ws.sort_by(f64::total_cmp);
    let (u0, u1) = ((us[0] + us[1]) * 0.5, (us[2] + us[3]) * 0.5);
    let (w0, w1) = ((ws[0] + ws[1]) * 0.5, (ws[2] + ws[3]) * 0.5);
    RecognizedShape::Rectangle {
        center: d * ((u0 + u1) * 0.5) + nrm * ((w0 + w1) * 0.5),
        half_extents: Vector2::new((u1 - u0) * 0.5, (w1 - w0) * 0.5),
        angle,
    }
}

// ---------- helpers ----------

fn dedup(points: &[Vector2]) -> Vec<Vector2> {
    let mut out: Vec<Vector2> = Vec::with_capacity(points.len());
    for &p in points {
        if !p.is_finite() {
            continue;
        }
        if out.last().is_none_or(|last| (p - *last).length() > 1e-6) {
            out.push(p);
        }
    }
    out
}

fn bounds(points: &[Vector2]) -> (Vector2, Vector2) {
    points.iter().fold(
        (Vector2::splat(f64::MAX), Vector2::splat(f64::MIN)),
        |(mins, maxs), p| (mins.min(*p), maxs.max(*p)),
    )
}

fn polyline_length(points: &[Vector2]) -> f64 {
    points.windows(2).map(|w| (w[1] - w[0]).length()).sum()
}

/// Resamples an open polyline to `n` points with equal spacing.
fn resample(points: &[Vector2], n: usize) -> Vec<Vector2> {
    let total = polyline_length(points);
    if total < f64::EPSILON || points.len() < 2 {
        return points.to_vec();
    }
    let step = total / (n - 1) as f64;
    let mut out = vec![points[0]];
    let mut acc = 0.0;
    let mut target = step;
    for w in points.windows(2) {
        let seg = (w[1] - w[0]).length();
        while seg > 0.0 && acc + seg >= target && out.len() < n - 1 {
            let t = (target - acc) / seg;
            out.push(w[0] + (w[1] - w[0]) * t);
            target += step;
        }
        acc += seg;
    }
    out.push(points[points.len() - 1]);
    out
}

/// Moving average over an open polyline, keeping the end points.
fn smooth_open(pts: &[Vector2], window: usize) -> Vec<Vector2> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let w = window.min(i).min(n - 1 - i);
            let slice = &pts[i - w..=i + w];
            slice.iter().fold(Vector2::ZERO, |acc, p| acc + *p) / slice.len() as f64
        })
        .collect()
}

/// Resamples a closed polyline (the last point connects to the first) to `n` points.
fn resample_closed(points: &[Vector2], n: usize) -> Vec<Vector2> {
    let mut closed = points.to_vec();
    closed.push(points[0]);
    let mut out = resample(&closed, n + 1);
    out.pop();
    out
}

/// If the end of a closed stroke passes over its start again, cuts the part after the closest approach.
///
/// Returns the closed part and the part that was cut.
fn trim_overshoot(path: &[Vector2]) -> (Vec<Vector2>, Vec<Vector2>) {
    let n = path.len();
    let start = path[0];
    let from = n * 3 / 4;
    let (best, _) = (from..n)
        .map(|i| (i, (path[i] - start).length()))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    let cut = best.max(3);
    (path[..=cut].to_vec(), path[cut + 1..].to_vec())
}

/// Total absolute and net turning angle of a polyline.
fn turning(points: &[Vector2], closed: bool) -> (f64, f64) {
    let n = points.len();
    let (mut abs_turn, mut net_turn) = (0.0, 0.0);
    let count = if closed { n } else { n.saturating_sub(2) };
    for i in 0..count {
        let (a, b, c) = if closed {
            (points[i], points[(i + 1) % n], points[(i + 2) % n])
        } else {
            (points[i], points[i + 1], points[i + 2])
        };
        let (u, v) = (b - a, c - b);
        if u.length() < f64::EPSILON || v.length() < f64::EPSILON {
            continue;
        }
        let t = (u.x * v.y - u.y * v.x).atan2(u.dot(v));
        abs_turn += t.abs();
        net_turn += t;
    }
    (abs_turn, net_turn)
}

/// The (unsigned) angle the path turns at `b`, in radians.
fn turn_angle(a: Vector2, b: Vector2, c: Vector2) -> f64 {
    let (u, v) = (b - a, c - b);
    (u.x * v.y - u.y * v.x).atan2(u.dot(v)).abs()
}

/// Mean and unit direction of the principal axis.
fn principal_axis(points: &[Vector2]) -> (Vector2, Vector2) {
    let n = points.len() as f64;
    let mean = points.iter().fold(Vector2::ZERO, |acc, p| acc + *p) / n;
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for p in points {
        let d = *p - mean;
        sxx += d.x * d.x;
        sxy += d.x * d.y;
        syy += d.y * d.y;
    }
    let angle = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    (mean, Vector2::new(angle.cos(), angle.sin()))
}

fn dist_to_segment(p: Vector2, a: Vector2, b: Vector2) -> f64 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < f64::EPSILON {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

fn dist_to_closed_polyline(p: Vector2, v: &[Vector2]) -> f64 {
    (0..v.len())
        .map(|i| dist_to_segment(p, v[i], v[(i + 1) % v.len()]))
        .fold(f64::MAX, f64::min)
}

/// Douglas-Peucker simplification of an open polyline. Returns the indices of the kept points.
fn simplify_indices(pts: &[Vector2], eps: f64, from: usize, to: usize, out: &mut Vec<usize>) {
    if to <= from + 1 {
        return;
    }
    let (mut best, mut best_d) = (from, 0.0);
    for i in from + 1..to {
        let d = dist_to_segment(pts[i], pts[from], pts[to]);
        if d > best_d {
            best = i;
            best_d = d;
        }
    }
    if best_d > eps {
        simplify_indices(pts, eps, from, best, out);
        out.push(best);
        simplify_indices(pts, eps, best, to, out);
    }
}

/// Douglas-Peucker simplification of a closed polyline.
fn simplify_closed(pts: &[Vector2], eps: f64) -> Vec<Vector2> {
    let n = pts.len();
    // split at the point farthest from the first one
    let far = (0..n)
        .max_by(|&a, &b| {
            (pts[a] - pts[0])
                .length()
                .total_cmp(&(pts[b] - pts[0]).length())
        })
        .unwrap();
    let mut ring = pts.to_vec();
    ring.push(pts[0]);
    let mut idx = vec![0];
    simplify_indices(&ring, eps, 0, far, &mut idx);
    idx.push(far);
    simplify_indices(&ring, eps, far, n, &mut idx);
    idx.into_iter().map(|i| ring[i]).collect()
}

#[cfg(test)]
mod tests;
