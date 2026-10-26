//! Tests of the shape recognition with synthetic strokes.
//!
//! Run `cargo test -p rnote-compose shaperecognition -- --nocapture` to see the statistics.

use super::*;
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg64;

/// How much a synthetic stroke deviates from the ideal shape.
#[derive(Debug, Clone, Copy)]
struct Noise {
    /// Amplitude of slow wobbles, relative to the size.
    wobble: f64,
    /// Standard deviation of the jitter, relative to the size.
    jitter: f64,
    /// Max. gap or overshoot when closing, relative to the perimeter.
    closing: f64,
}

const LOW: Noise = Noise {
    wobble: 0.01,
    jitter: 0.003,
    closing: 0.04,
};
const MODERATE: Noise = Noise {
    wobble: 0.025,
    jitter: 0.006,
    closing: 0.08,
};

fn gauss(rng: &mut Pcg64) -> f64 {
    let u1: f64 = rng.random::<f64>().max(1e-12);
    let u2: f64 = rng.random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

fn rotate(p: Vector2, angle: f64) -> Vector2 {
    Vector2::new(
        p.x * angle.cos() - p.y * angle.sin(),
        p.x * angle.sin() + p.y * angle.cos(),
    )
}

/// Samples a closed polygon outline with spacing ~2 units.
fn sample_polygon(vertices: &[Vector2]) -> Vec<Vector2> {
    let mut out = Vec::new();
    for i in 0..vertices.len() {
        let (a, b) = (vertices[i], vertices[(i + 1) % vertices.len()]);
        let n = ((b - a).length() / 2.0).ceil().max(1.0) as usize;
        for j in 0..n {
            out.push(a + (b - a) * (j as f64 / n as f64));
        }
    }
    out
}

fn sample_ellipse(a: f64, b: f64) -> Vec<Vector2> {
    let n = (PI * (a + b) / 2.0).ceil() as usize;
    (0..n)
        .map(|i| {
            let t = 2.0 * PI * i as f64 / n as f64;
            Vector2::new(a * t.cos(), b * t.sin())
        })
        .collect()
}

/// Moving average over a closed outline, rounds corners like a hand does.
fn smooth_closed(pts: &[Vector2], window: usize) -> Vec<Vector2> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let mut acc = Vector2::ZERO;
            for j in 0..(2 * window + 1) {
                acc += pts[(i + n + j - window) % n];
            }
            acc / (2 * window + 1) as f64
        })
        .collect()
}

/// Turns an ideal closed outline into a hand drawn stroke: random start, wobble, jitter, gap or overshoot,
/// rotation, scale and position.
fn hand_draw_closed(
    ideal: &[Vector2],
    noise: Noise,
    rng: &mut Pcg64,
    round_corners: bool,
) -> Vec<Vector2> {
    let n = ideal.len();
    let mut pts = if round_corners {
        smooth_closed(ideal, (n / 60).max(1))
    } else {
        ideal.to_vec()
    };
    let start = rng.random_range(0..n);
    pts.rotate_left(start);
    // gap (negative) or overshoot (positive)
    let closing = rng.random_range(-noise.closing..noise.closing);
    let extra = (closing * n as f64).round() as i64;
    let mut stroke = if extra >= 0 {
        let mut s = pts.clone();
        s.extend(pts.iter().take(extra as usize));
        s
    } else {
        pts[..(n as i64 + extra).max(3) as usize].to_vec()
    };
    let (mins, maxs) = bounds(&stroke);
    let size = (maxs - mins).length();
    let phases = [
        rng.random::<f64>() * 2.0 * PI,
        rng.random::<f64>() * 2.0 * PI,
    ];
    let freqs = [rng.random_range(1.0..3.0), rng.random_range(3.0..6.0)];
    let len = stroke.len() as f64;
    let center = (mins + maxs) * 0.5;
    for (i, p) in stroke.iter_mut().enumerate() {
        let t = i as f64 / len * 2.0 * PI;
        let radial = (*p - center).normalize_or_zero();
        let wobble = noise.wobble
            * size
            * 0.5
            * ((freqs[0] * t + phases[0]).sin() + (freqs[1] * t + phases[1]).sin());
        *p += radial * wobble + Vector2::new(gauss(rng), gauss(rng)) * noise.jitter * size;
    }
    place(stroke, rng)
}

fn place(stroke: Vec<Vector2>, rng: &mut Pcg64) -> Vec<Vector2> {
    let angle = rng.random_range(-PI..PI);
    let offset = Vector2::new(
        rng.random_range(-500.0..500.0),
        rng.random_range(-500.0..500.0),
    );
    stroke
        .into_iter()
        .map(|p| rotate(p, angle) + offset)
        .collect()
}

fn hand_line(noise: Noise, rng: &mut Pcg64) -> Vec<Vector2> {
    let len = rng.random_range(80.0..500.0);
    let n = (len / 2.0) as usize;
    let phase = rng.random::<f64>() * 2.0 * PI;
    let pts = (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = noise.wobble
                * len
                * 0.6
                * (PI * t * rng.random_range(1.0..1.05) + phase).sin()
                * (PI * t).sin();
            Vector2::new(t * len, wobble)
                + Vector2::new(gauss(rng), gauss(rng)) * noise.jitter * len
        })
        .collect();
    place(pts, rng)
}

fn hand_shape(kind: &str, noise: Noise, rng: &mut Pcg64) -> Vec<Vector2> {
    let size = rng.random_range(80.0..400.0);
    match kind {
        "line" => hand_line(noise, rng),
        "circle" => hand_draw_closed(&sample_ellipse(size * 0.5, size * 0.5), noise, rng, false),
        "ellipse" => {
            let ratio = rng.random_range(0.25..0.65);
            hand_draw_closed(
                &sample_ellipse(size * 0.5, size * 0.5 * ratio),
                noise,
                rng,
                false,
            )
        }
        "rectangle" => {
            let ratio = rng.random_range(0.3..1.0);
            let (w, h) = (size * 0.5, size * 0.5 * ratio);
            let v = [
                Vector2::new(-w, -h),
                Vector2::new(w, -h),
                Vector2::new(w, h),
                Vector2::new(-w, h),
            ];
            hand_draw_closed(&sample_polygon(&v), noise, rng, true)
        }
        "triangle" => {
            // random triangle with all angles above 25 degrees
            loop {
                let v = [0, 1, 2].map(|_| {
                    Vector2::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
                        * size
                        * 0.5
                });
                let ok = (0..3).all(|i| {
                    let (a, b, c) = (v[(i + 2) % 3], v[i], v[(i + 1) % 3]);
                    PI - turn_angle(a, b, c) > 25_f64.to_radians()
                });
                let area =
                    ((v[1] - v[0]).x * (v[2] - v[0]).y - (v[1] - v[0]).y * (v[2] - v[0]).x).abs();
                if ok && area > size * size * 0.08 {
                    return hand_draw_closed(&sample_polygon(&v), noise, rng, true);
                }
            }
        }
        _ => unreachable!(),
    }
}

// ---------- strokes that are not shapes ----------

/// Strokes that resemble handwriting or scribbles, drawn big enough to pass the size filter.
fn non_shape(i: usize, rng: &mut Pcg64) -> Vec<Vector2> {
    let size = rng.random_range(60.0..250.0);
    let jitter = 0.004 * size;
    let mut pts: Vec<Vector2> = match i % 10 {
        // cursive loops: "eeee", "llll"
        0 => {
            let loops = rng.random_range(2..6) as f64;
            let r = size * 0.25;
            (0..400)
                .map(|j| {
                    let t = j as f64 / 400.0 * loops * 2.0 * PI;
                    Vector2::new(
                        t * size / (loops * 2.0 * PI) * 1.5 - r * t.sin(),
                        -r * t.cos(),
                    )
                })
                .collect()
        }
        // "w", "m", zigzags
        1 => {
            let n = rng.random_range(3..7);
            let v = (0..=n)
                .map(|j| {
                    Vector2::new(
                        j as f64 * size / n as f64,
                        if j % 2 == 0 { 0.0 } else { size * 0.6 },
                    )
                })
                .collect::<Vec<Vector2>>();
            let mut s = sample_polygon(&v);
            s.truncate(s.len() - (v[n] - v[0]).length() as usize / 2);
            s
        }
        // "s"
        2 => (0..200)
            .map(|j| {
                let t = j as f64 / 200.0 * 2.0 * PI;
                Vector2::new(
                    size * 0.3 * (t * 1.0).sin() * if t < PI { 1.0 } else { -1.0 },
                    -size * 0.5 + size * t / (2.0 * PI),
                )
            })
            .collect(),
        // "8"
        3 => (0..300)
            .map(|j| {
                let t = j as f64 / 300.0 * 2.0 * PI;
                Vector2::new(size * 0.3 * (2.0 * t).sin(), size * 0.5 * t.sin())
            })
            .collect(),
        // spiral / "@"
        4 => (0..400)
            .map(|j| {
                let t = j as f64 / 400.0 * rng.random_range(3.0..4.5) * PI;
                let r = size * 0.15 * (1.0 + t / (2.0 * PI));
                Vector2::new(r * t.cos(), r * t.sin())
            })
            .collect(),
        // "2"
        5 => {
            let mut s = (0..100)
                .map(|j| {
                    let t = PI + j as f64 / 100.0 * 1.3 * PI;
                    Vector2::new(size * 0.3 * t.cos(), -size * 0.3 + size * 0.3 * t.sin())
                })
                .collect::<Vec<Vector2>>();
            let last = *s.last().unwrap();
            s.extend(
                sample_polygon(&[
                    last,
                    Vector2::new(-size * 0.3, size * 0.5),
                    Vector2::new(size * 0.35, size * 0.5),
                ])
                .into_iter()
                .take(((size * 0.5 + 0.65 * size) / 2.0) as usize),
            );
            s
        }
        // "3"
        6 => (0..300)
            .map(|j| {
                let t = j as f64 / 300.0;
                let (cy, a) = if t < 0.5 {
                    (-size * 0.25, (t * 2.0) * 1.5 * PI - 0.75 * PI)
                } else {
                    (size * 0.25, ((t - 0.5) * 2.0) * 1.5 * PI - 0.75 * PI)
                };
                Vector2::new(size * 0.25 * a.cos(), cy + size * 0.25 * a.sin())
            })
            .collect(),
        // random smooth scribble (sum of sinusoids)
        7 => {
            let f = [
                rng.random_range(1.0..4.0),
                rng.random_range(1.0..4.0),
                rng.random_range(2.0..6.0),
            ];
            (0..300)
                .map(|j| {
                    let t = j as f64 / 300.0 * 2.0 * PI;
                    Vector2::new(
                        size * 0.5 * (f[0] * t).sin() + size * 0.2 * (f[2] * t).cos(),
                        size * 0.5 * (f[1] * t + 1.0).sin(),
                    )
                })
                .collect()
        }
        // "a": loop plus stem (closed loop then a tail)
        8 => {
            let mut s = sample_ellipse(size * 0.3, size * 0.35);
            let q = s.len() / 8;
            s.rotate_left(q);
            let last = s[0];
            s.extend(
                sample_polygon(&[last, last + Vector2::new(0.0, size * 0.5)])
                    .into_iter()
                    .take((size * 0.25) as usize),
            );
            s
        }
        // "D": straight side plus arc
        _ => {
            let mut s = sample_polygon(&[
                Vector2::new(0.0, size * 0.5),
                Vector2::new(0.0, -size * 0.5),
            ]);
            s.truncate(s.len() / 2);
            s.extend((0..=100).map(|j| {
                let t = -PI / 2.0 + j as f64 / 100.0 * PI;
                Vector2::new(size * 0.5 * t.cos(), size * 0.5 * t.sin())
            }));
            s
        }
    };
    for p in pts.iter_mut() {
        *p += Vector2::new(gauss(rng), gauss(rng)) * jitter;
    }
    place(pts, rng)
}

const KINDS: [&str; 5] = ["line", "circle", "ellipse", "triangle", "rectangle"];

/// Returns (correct, wrong kind, not recognized) for each kind.
fn run_shapes(
    noise: Noise,
    n: usize,
    seed: u64,
    params: RecognitionParams,
) -> Vec<(&'static str, usize, usize, usize)> {
    let mut rng = Pcg64::seed_from_u64(seed);
    KINDS
        .iter()
        .map(|&kind| {
            let (mut ok, mut wrong, mut none) = (0, 0, 0);
            for _ in 0..n {
                let stroke = hand_shape(kind, noise, &mut rng);
                match recognize(&stroke, params) {
                    // a circle recognized as an ellipse (or the other way round) is acceptable
                    Some(s) if s.kind() == kind || (kind == "circle" && s.kind() == "ellipse") => {
                        ok += 1
                    }
                    Some(_) => wrong += 1,
                    None => none += 1,
                }
            }
            (kind, ok, wrong, none)
        })
        .collect()
}

fn false_positives(n: usize, seed: u64, params: RecognitionParams) -> (usize, Vec<String>) {
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut fp = 0;
    let mut which = Vec::new();
    for i in 0..n {
        let stroke = non_shape(i, &mut rng);
        if let Some(s) = recognize(&stroke, params) {
            fp += 1;
            which.push(format!("type {} -> {}", i % 10, s.kind()));
        }
    }
    (fp, which)
}

#[test]
fn shapes_low_noise_are_recognized() {
    let res = run_shapes(LOW, 100, 1, RecognitionParams::default());
    for (kind, ok, wrong, none) in res.iter() {
        println!("low noise      {kind:10} ok {ok:3}  wrong {wrong:3}  none {none:3}");
    }
    for &(kind, ok, wrong, none) in res.iter() {
        assert!(
            ok >= 95,
            "{kind}: only {ok}/100 recognized (wrong {wrong}, none {none})"
        );
    }
}

#[test]
fn shapes_moderate_noise_are_recognized() {
    let res = run_shapes(MODERATE, 100, 2, RecognitionParams::default());
    for (kind, ok, wrong, none) in res.iter() {
        println!("moderate noise {kind:10} ok {ok:3}  wrong {wrong:3}  none {none:3}");
    }
    for &(kind, ok, wrong, none) in res.iter() {
        assert!(
            ok >= 85,
            "{kind}: only {ok}/100 recognized (wrong {wrong}, none {none})"
        );
        assert!(
            wrong <= 5,
            "{kind}: {wrong}/100 recognized as another shape"
        );
    }
}

#[test]
fn handwriting_and_scribbles_are_not_recognized() {
    let n = 500;
    let (fp, which) = false_positives(n, 3, RecognitionParams::default());
    println!("non-shapes: {fp}/{n} false positives: {which:?}");
    assert!(
        fp * 100 <= n * 3,
        "too many false positives: {fp}/{n}: {which:?}"
    );
}

#[test]
fn sensitivity_changes_the_tradeoff() {
    let strict = RecognitionParams {
        sensitivity: 0.0,
        ..Default::default()
    };
    let permissive = RecognitionParams {
        sensitivity: 1.0,
        ..Default::default()
    };
    let total = |p| {
        run_shapes(MODERATE, 40, 4, p)
            .iter()
            .map(|r| r.1)
            .sum::<usize>()
    };
    let (s, p) = (total(strict), total(permissive));
    println!("moderate noise recognized: strict {s}/200, permissive {p}/200");
    assert!(s <= p);
    let (fp_strict, _) = false_positives(200, 5, strict);
    let (fp_perm, _) = false_positives(200, 5, permissive);
    println!("false positives: strict {fp_strict}/200, permissive {fp_perm}/200");
    assert!(fp_strict <= fp_perm);
}

#[test]
fn small_strokes_are_ignored() {
    let mut rng = Pcg64::seed_from_u64(6);
    let circle = hand_draw_closed(&sample_ellipse(15.0, 15.0), LOW, &mut rng, false);
    assert_eq!(recognize(&circle, RecognitionParams::default()), None);
}

#[test]
fn recognized_geometry_is_accurate() {
    // a clean rotated rectangle
    let v = [
        Vector2::new(-100.0, -40.0),
        Vector2::new(100.0, -40.0),
        Vector2::new(100.0, 40.0),
        Vector2::new(-100.0, 40.0),
    ]
    .map(|p| rotate(p, 0.3) + Vector2::new(500.0, 300.0));
    let mut stroke = sample_polygon(&v);
    stroke.push(stroke[0]);
    match recognize(&stroke, RecognitionParams::default()) {
        Some(RecognizedShape::Rectangle {
            center,
            half_extents,
            angle,
        }) => {
            assert!(
                (center - Vector2::new(500.0, 300.0)).length() < 1.0,
                "{center:?} {half_extents:?} {angle}"
            );
            let (a, b) = (
                half_extents.x.max(half_extents.y),
                half_extents.x.min(half_extents.y),
            );
            assert!(
                (a - 100.0).abs() < 2.0 && (b - 40.0).abs() < 2.0,
                "{half_extents:?}"
            );
            // the angle is only defined modulo 90 degrees
            let d = (angle - 0.3).rem_euclid(PI / 2.0);
            assert!(d < 0.02 || d > PI / 2.0 - 0.02, "angle {angle}");
        }
        other => panic!("expected a rectangle, got {other:?}"),
    }
    // a clean circle
    let stroke = sample_ellipse(80.0, 80.0)
        .into_iter()
        .map(|p| p + Vector2::new(10.0, 20.0))
        .collect::<Vec<Vector2>>();
    match recognize(&stroke, RecognitionParams::default()) {
        Some(RecognizedShape::Circle { center, radius }) => {
            assert!((center - Vector2::new(10.0, 20.0)).length() < 1.0);
            assert!((radius - 80.0).abs() < 1.0);
        }
        other => panic!("expected a circle, got {other:?}"),
    }
}

/// Prints why strokes are not recognized. Run with `-- --ignored --nocapture`.
#[test]
#[ignore]
fn diagnose_rejections() {
    let mut rng = Pcg64::seed_from_u64(2);
    for kind in KINDS {
        let mut reasons = std::collections::BTreeMap::<String, usize>::new();
        for _ in 0..100 {
            let stroke = hand_shape(kind, MODERATE, &mut rng);
            let r = match recognize_w_reason(&stroke, RecognitionParams::default()) {
                Ok(s) => format!("ok {}", s.kind()),
                Err(e) => e.to_string(),
            };
            *reasons.entry(r).or_default() += 1;
        }
        println!("{kind}: {reasons:?}");
    }
    let mut reasons = std::collections::BTreeMap::<String, usize>::new();
    for i in 0..500 {
        let stroke = non_shape(i, &mut rng);
        let r = match recognize_w_reason(&stroke, RecognitionParams::default()) {
            Ok(s) => format!("type {} -> {}", i % 10, s.kind()),
            Err(e) => e.to_string(),
        };
        *reasons.entry(r).or_default() += 1;
    }
    println!("non-shapes: {reasons:?}");
}
