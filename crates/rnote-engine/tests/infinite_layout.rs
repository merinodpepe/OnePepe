// OnePepe: comprobación automática de las hojas infinitas.
//
// Simula lo que hace la interfaz al desplazarse y hacer zoom (mover la cámara y
// expandir el documento) y comprueba que la zona visible queda siempre dentro del
// documento, es decir, que nunca se "choca" con un borde.

use p2d::math::Vector2;
use parry2d_f64 as p2d;
use rnote_engine::Engine;
use rnote_engine::document::Layout;

/// Comprueba que el documento contiene por completo la zona visible de la cámara.
fn assert_viewport_inside_doc(engine: &Engine, ctx: &str) {
    let vp = engine.camera.viewport();
    let doc = &engine.document;
    let eps = 1e-6;
    assert!(
        vp.mins.x >= doc.x - eps
            && vp.mins.y >= doc.y - eps
            && vp.maxs.x <= doc.x + doc.width + eps
            && vp.maxs.y <= doc.y + doc.height + eps,
        "{ctx}: la zona visible {vp:?} se sale del documento (x={}, y={}, w={}, h={})",
        doc.x,
        doc.y,
        doc.width,
        doc.height
    );
}

fn infinite_engine() -> Engine {
    let mut engine = Engine::default();
    let _ = engine.camera_set_size(Vector2::new(1600.0, 1000.0));
    let _ = engine.set_doc_layout(Layout::Infinite);
    engine
}

/// Desplaza la cámara paso a paso en una dirección, como al arrastrar con el dedo o el lápiz.
fn pan(engine: &mut Engine, dir: Vector2, steps: usize, step_len: f64) {
    for i in 0..steps {
        let new_offset = engine.camera.offset() + dir * step_len;
        let _ = engine.camera_set_offset_expand(new_offset);
        assert_viewport_inside_doc(engine, &format!("desplazamiento {dir:?}, paso {i}"));
        // La cámara no debe quedarse bloqueada: el desplazamiento pedido se aplica entero.
        let diff = (engine.camera.offset() - new_offset).length();
        assert!(
            diff < 1e-6,
            "desplazamiento {dir:?}, paso {i}: la cámara se ha quedado bloqueada (diferencia {diff})"
        );
    }
}

#[test]
fn infinite_layout_is_default() {
    assert_eq!(Engine::default().document.config.layout, Layout::Infinite);
}

#[test]
fn infinite_layout_pans_in_all_directions() {
    let mut engine = infinite_engine();
    assert_viewport_inside_doc(&engine, "inicio");
    for dir in [
        Vector2::new(1.0, 0.0),
        Vector2::new(-1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, -1.0),
        Vector2::new(1.0, 1.0).normalize(),
        Vector2::new(-1.0, -1.0).normalize(),
    ] {
        // 300 pasos de 200 px: unas 60 000 unidades, bastante más que varias páginas.
        pan(&mut engine, dir, 300, 200.0);
    }
}

#[test]
fn infinite_layout_survives_zoom_and_pan() {
    let mut engine = infinite_engine();
    for zoom in [0.2, 0.5, 1.0, 2.5, 6.0, 0.15] {
        let _ = engine.camera.zoom_to(zoom);
        let _ = engine.doc_expand_autoexpand();
        assert_viewport_inside_doc(&engine, &format!("zoom {zoom}"));
        pan(&mut engine, Vector2::new(-1.0, 0.0), 50, 400.0);
        pan(&mut engine, Vector2::new(0.0, -1.0), 50, 400.0);
        pan(&mut engine, Vector2::new(1.0, 1.0).normalize(), 100, 400.0);
    }
}

#[test]
fn infinite_layout_shrinks_back_when_empty() {
    // Tras alejarse mucho y volver, el documento puede reducirse al ajustarlo al contenido,
    // pero debe seguir conteniendo la zona visible.
    let mut engine = infinite_engine();
    pan(&mut engine, Vector2::new(1.0, 0.0), 200, 500.0);
    let _ = engine.camera_set_offset_expand(Vector2::ZERO);
    let _ = engine.doc_resize_autoexpand();
    assert_viewport_inside_doc(&engine, "vuelta al origen");
}
