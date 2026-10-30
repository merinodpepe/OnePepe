// SPDX-License-Identifier: GPL-3.0-or-later
//! OnePepe: the scratch-out gesture (a zigzag over ink erases it), phase 1 of the MyScript-like plan.
//!
//! A zigzag goes back and forth several times (four or more turns, in any direction). A "z" turns back twice and a
//! "w" three times. Only the shape is checked here; the engine also asks
//! that the zigzag covers existing ink, otherwise it is kept as ink.

// Imports
use p2d::math::Vector2;

/// Parameters of the gesture.
#[derive(Debug, Clone, Copy)]
pub struct ScratchParams {
    /// How many times the stroke must turn back along its main direction.
    pub min_reversals: usize,
    /// Each back and forth must cover at least this part of the zigzag's length.
    pub min_swing: f64,
    /// Smaller zigzags (length along the main direction) are never a scratch-out.
    pub min_size: f64,
}

impl Default for ScratchParams {
    fn default() -> Self {
        Self {
            min_reversals: 4,
            min_swing: 0.5,
            min_size: 10.0,
        }
    }
}

impl ScratchParams {
    /// Parameters for a sensitivity from 0.0 (strict: more turns, wider swings) to 1.0 (permissive).
    pub fn with_sensitivity(sensitivity: f64, min_size: f64) -> Self {
        let s = sensitivity.clamp(0.0, 1.0);
        Self {
            // a «w» turns back three times: a zigzag, four or more
            min_reversals: if s < 0.34 { 5 } else { 4 },
            min_swing: 0.65 - 0.3 * s,
            min_size,
        }
    }
}

/// How many times the points turn back along `dir`, counting only swings of at least `min_swing` of their
/// length along it (and that length).
fn reversals_along(points: &[Vector2], dir: Vector2, min_swing: f64) -> (usize, f64) {
    let along = points.iter().map(|p| p.dot(dir)).collect::<Vec<f64>>();
    let (lo, hi) = along
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
    let length = hi - lo;
    let swing = min_swing * length;
    let mut reversals = 0;
    let mut going_up: Option<bool> = None;
    let mut extreme = along[0];
    for &v in &along[1..] {
        match going_up {
            None => {
                if (v - extreme).abs() >= swing {
                    going_up = Some(v > extreme);
                    extreme = v;
                }
            }
            Some(true) => {
                if v > extreme {
                    extreme = v;
                } else if extreme - v >= swing {
                    reversals += 1;
                    going_up = Some(false);
                    extreme = v;
                }
            }
            Some(false) => {
                if v < extreme {
                    extreme = v;
                } else if v - extreme >= swing {
                    reversals += 1;
                    going_up = Some(true);
                    extreme = v;
                }
            }
        }
    }
    (reversals, length)
}

/// Whether the stroke is a scratch-out zigzag: in some direction it goes back and forth enough times.
pub fn is_scratch_out(points: &[Vector2], params: ScratchParams) -> bool {
    if points.len() < 8 {
        return false;
    }
    // every 15°: a zigzag over one symbol is as tall as wide, so its main direction is not clear
    (0..12).any(|i| {
        let angle = i as f64 * std::f64::consts::PI / 12.0;
        let (reversals, length) = reversals_along(
            points,
            Vector2::new(angle.cos(), angle.sin()),
            params.min_swing,
        );
        length >= params.min_size && reversals >= params.min_reversals
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polyline(corners: &[(f64, f64)]) -> Vec<Vector2> {
        corners
            .windows(2)
            .flat_map(|w| {
                (0..10).map(move |i| {
                    let t = i as f64 / 10.0;
                    Vector2::new(
                        w[0].0 + (w[1].0 - w[0].0) * t,
                        w[0].1 + (w[1].1 - w[0].1) * t,
                    )
                })
            })
            .collect()
    }

    #[test]
    fn zigzags_are_scratch_outs() {
        let p = ScratchParams::default();
        // horizontal, over a word
        let zz = polyline(&[
            (0., 0.),
            (60., 5.),
            (2., 10.),
            (58., 15.),
            (0., 20.),
            (60., 25.),
        ]);
        assert!(is_scratch_out(&zz, p));
        // vertical and tilted
        let zz = polyline(&[
            (0., 0.),
            (5., 40.),
            (10., 2.),
            (15., 38.),
            (20., 0.),
            (25., 40.),
        ]);
        assert!(is_scratch_out(&zz, p));
    }

    #[test]
    fn letters_are_not() {
        let p = ScratchParams::default();
        let w = polyline(&[(0., 0.), (5., 20.), (10., 5.), (15., 20.), (20., 0.)]);
        assert!(!is_scratch_out(&w, p));
        let z = polyline(&[(0., 0.), (20., 0.), (0., 20.), (20., 20.)]);
        assert!(!is_scratch_out(&z, p));
        let m = polyline(&[(0., 20.), (0., 0.), (10., 15.), (20., 0.), (20., 20.)]);
        assert!(!is_scratch_out(&m, p));
        // a short back and forth (an «N»)
        let n = polyline(&[(0., 20.), (0., 0.), (15., 20.), (15., 0.)]);
        assert!(!is_scratch_out(&n, p));
        // too small
        let zz = polyline(&[(0., 0.), (6., 0.5), (0., 1.), (6., 1.5), (0., 2.)]);
        assert!(!is_scratch_out(&zz, p));
    }

    #[test]
    fn sensitivity() {
        // four turns: only with normal or permissive sensitivity
        let zz = polyline(&[
            (0., 0.),
            (40., 4.),
            (0., 8.),
            (40., 12.),
            (0., 16.),
            (40., 20.),
        ]);
        assert!(is_scratch_out(
            &zz,
            ScratchParams::with_sensitivity(0.5, 10.0)
        ));
        assert!(!is_scratch_out(
            &zz,
            ScratchParams::with_sensitivity(0.0, 10.0)
        ));
    }
}
