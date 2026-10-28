// OnePepe: a whole section (all the pages of a folder) exported as a single PDF, for printing or studying.

// Imports
use super::snapshot::Snapshotable;
use super::{Engine, EngineSnapshot};
use anyhow::Context;

/// Exports the pages (title and `.rnote` bytes, in this order) as one PDF. Every page of every document becomes a
/// PDF page with the format size of its document; empty documents are skipped.
pub async fn export_pages_as_one_pdf(
    title: String,
    pages: Vec<(String, Vec<u8>)>,
) -> anyhow::Result<Vec<u8>> {
    const PT_IN_INCH: f64 = 72.0;
    let surface = cairo::PdfSurface::for_stream(595.0, 842.0, Vec::<u8>::new())
        .context("Creating Pdf target surface failed.")?;
    surface
        .set_metadata(cairo::PdfMetadata::Title, title.as_str())
        .context("Set pdf surface title metadata failed.")?;
    let mut n_pages = 0;
    {
        let cx = cairo::Context::new(&surface).context("Creating cairo context failed.")?;
        for (name, bytes) in pages {
            let snapshot = match EngineSnapshot::load_from_rnote_bytes(bytes).await {
                Ok(snapshot) => snapshot,
                Err(e) => {
                    tracing::warn!("Skipping page `{name}` that could not be read, Err: {e:?}");
                    continue;
                }
            };
            // only the document and the strokes: no screen rendering for this temporary engine
            let mut engine = Engine::default();
            engine.document = snapshot.document.extract_snapshot_data();
            let _ = engine.store.import_from_snapshot(&snapshot);
            // background rendering jobs of the strokes may still report back after this engine is gone: keep their
            // channel open for a moment instead of logging send errors
            if let Some(tasks_rx) = engine.take_engine_tasks_rx() {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    drop(tasks_rx);
                });
            }
            if engine.store.stroke_keys_as_rendered().is_empty() {
                continue;
            }
            let prefs = engine.config.read().export_prefs.doc_export_prefs;
            let format_size = engine.document.config.format.size();
            let dpi = engine.document.config.format.dpi();
            for page_content in engine.extract_pages_content(prefs.page_order) {
                let Some(page_bounds) = page_content.bounds() else {
                    continue;
                };
                surface
                    .set_size(
                        format_size[0] * (PT_IN_INCH / dpi),
                        format_size[1] * (PT_IN_INCH / dpi),
                    )
                    .context("Setting the pdf page size failed.")?;
                cx.save()?;
                cx.scale(PT_IN_INCH / dpi, PT_IN_INCH / dpi);
                cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);
                page_content.draw_to_cairo(
                    &cx,
                    prefs.with_background,
                    prefs.with_pattern,
                    prefs.optimize_printing,
                    0.0,
                    Engine::STROKE_EXPORT_IMAGE_SCALE,
                )?;
                cx.restore()?;
                cx.show_page()
                    .map_err(|e| anyhow::anyhow!("Showing pdf page failed, Err: {e:?}"))?;
                n_pages += 1;
            }
        }
    }
    if n_pages == 0 {
        return Err(anyhow::anyhow!(
            "All the pages are empty, nothing to export."
        ));
    }
    let data = *surface
        .finish_output_stream()
        .map_err(|e| anyhow::anyhow!("Finishing outputstream failed, Err: {e:?}"))?
        .downcast::<Vec<u8>>()
        .map_err(|e| anyhow::anyhow!("Downcasting output stream failed, Err: {e:?}"))?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn two_documents_in_one_pdf() {
        let mut a = Engine::default();
        let _ = a.insert_table(2, 2);
        let mut b = Engine::default();
        let _ = b.insert_title_and_date("Tema 2");
        let _ = b.record(Instant::now());
        let empty = Engine::default();
        let bytes = |e: &Engine| {
            futures::executor::block_on(e.save_as_rnote_bytes("x.rnote".into()))
                .unwrap()
                .unwrap()
        };
        let pages = vec![
            ("a".to_string(), bytes(&a)),
            ("vacía".to_string(), bytes(&empty)),
            ("b".to_string(), bytes(&b)),
            ("rota".to_string(), b"no es un rnote".to_vec()),
        ];
        let pdf =
            futures::executor::block_on(export_pages_as_one_pdf("Sección".into(), pages)).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        let only_empty = vec![("vacía".to_string(), bytes(&empty))];
        assert!(
            futures::executor::block_on(export_pages_as_one_pdf("x".into(), only_empty)).is_err()
        );
    }
}
