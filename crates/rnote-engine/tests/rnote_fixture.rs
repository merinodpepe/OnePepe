// OnePepe: writes a small synthetic .rnote file ("2+3=") with the brush, used to test the stroke extractor of the
// math prototype (tools/onepepe-math).
//
// Run: ONEPEPE_WRITE_FIXTURE=path/to/file.rnote cargo test -p rnote-engine --test rnote_fixture

use p2d::math::Vector2;
use parry2d_f64 as p2d;
use rnote_compose::penevent::PenEvent;
use rnote_compose::penpath::Element;
use rnote_engine::Engine;
use std::collections::HashSet;
use std::time::{Duration, Instant};

fn draw(engine: &mut Engine, points: &[Vector2]) {
    let t0 = Instant::now();
    for (i, p) in points.iter().enumerate() {
        let element = Element::new(*p, 0.6);
        let modifier_keys = HashSet::new();
        let event = if i == points.len() - 1 {
            PenEvent::Up {
                element,
                modifier_keys,
            }
        } else {
            PenEvent::Down {
                element,
                modifier_keys,
            }
        };
        let _ = engine.handle_pen_event(event, None, t0 + Duration::from_millis(8 * i as u64));
    }
}

fn polyline(pts: &[(f64, f64)], offset: (f64, f64)) -> Vec<Vector2> {
    let mut out = Vec::new();
    for w in pts.windows(2) {
        for j in 0..10 {
            let t = j as f64 / 10.0;
            out.push(Vector2::new(
                offset.0 + w[0].0 + (w[1].0 - w[0].0) * t,
                offset.1 + w[0].1 + (w[1].1 - w[0].1) * t,
            ));
        }
    }
    let last = pts[pts.len() - 1];
    out.push(Vector2::new(offset.0 + last.0, offset.1 + last.1));
    out
}

#[test]
fn write_fixture() {
    let mut engine = Engine::default();
    // "2"
    draw(
        &mut engine,
        &polyline(
            &[
                (0.0, 10.0),
                (10.0, 0.0),
                (20.0, 10.0),
                (0.0, 40.0),
                (22.0, 40.0),
            ],
            (100.0, 100.0),
        ),
    );
    // "+"
    draw(
        &mut engine,
        &polyline(&[(0.0, 20.0), (24.0, 20.0)], (140.0, 100.0)),
    );
    draw(
        &mut engine,
        &polyline(&[(12.0, 8.0), (12.0, 32.0)], (140.0, 100.0)),
    );
    // "3"
    draw(
        &mut engine,
        &polyline(
            &[
                (0.0, 2.0),
                (18.0, 4.0),
                (6.0, 20.0),
                (20.0, 30.0),
                (0.0, 40.0),
            ],
            (180.0, 100.0),
        ),
    );
    // "="
    draw(
        &mut engine,
        &polyline(&[(0.0, 15.0), (24.0, 15.0)], (220.0, 100.0)),
    );
    draw(
        &mut engine,
        &polyline(&[(0.0, 27.0), (24.0, 27.0)], (220.0, 100.0)),
    );

    let bytes = futures::executor::block_on(engine.save_as_rnote_bytes("fixture.rnote".into()))
        .unwrap()
        .unwrap();
    assert!(!bytes.is_empty());
    if let Ok(path) = std::env::var("ONEPEPE_WRITE_FIXTURE") {
        std::fs::write(path, bytes).unwrap();
    }
}

/// OnePepe: the "new page" action of the workspace browser writes an empty engine to a file.
/// It must load again.
#[test]
fn empty_page_roundtrip() {
    let engine = Engine::default();
    let bytes = futures::executor::block_on(engine.save_as_rnote_bytes("Página 1.rnote".into()))
        .unwrap()
        .unwrap();
    let snapshot = futures::executor::block_on(
        rnote_engine::engine::EngineSnapshot::load_from_rnote_bytes(bytes),
    )
    .unwrap();
    assert_eq!(
        snapshot.document.config.layout,
        engine.document.config.layout
    );
}
