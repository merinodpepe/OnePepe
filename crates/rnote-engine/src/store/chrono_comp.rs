// Imports
use super::{StrokeKey, StrokeStore};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq)]
#[serde(rename = "stroke_layer")]
pub enum StrokeLayer {
    #[serde(rename = "user_layer", alias = "UserLayer")]
    UserLayer(u32),
    #[serde(rename = "highlighter", alias = "Highlighter")]
    Highlighter,
    #[serde(rename = "image", alias = "Image")]
    Image,
    #[serde(rename = "document", alias = "Document")]
    Document,
}

impl Default for StrokeLayer {
    fn default() -> Self {
        Self::UserLayer(0)
    }
}

impl PartialEq for StrokeLayer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UserLayer(l0), Self::UserLayer(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialOrd for StrokeLayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrokeLayer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let key = |layer: &StrokeLayer| match layer {
            StrokeLayer::Document => (0_u8, 0_u32),
            StrokeLayer::Image => (1_u8, 0_u32),
            StrokeLayer::UserLayer(layer_no) => (2_u8, *layer_no),
            StrokeLayer::Highlighter => (3_u8, 0_u32),
        };

        key(self).cmp(&key(other))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(default, rename = "chrono_component")]
pub struct ChronoComponent {
    #[serde(rename = "t")]
    t: u32,
    #[serde(rename = "layer")]
    pub layer: StrokeLayer,
}

impl Default for ChronoComponent {
    fn default() -> Self {
        Self {
            t: 0,
            layer: StrokeLayer::default(),
        }
    }
}

impl ChronoComponent {
    pub(crate) fn new(t: u32, layer: StrokeLayer) -> Self {
        Self { t, layer }
    }
}

/// Systems that are related to their chronological ordering.
impl StrokeStore {
    pub(crate) fn update_chrono_to_last(&mut self, key: StrokeKey) {
        if let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) {
            self.chrono_counter += 1;
            Arc::make_mut(chrono_comp).t = self.chrono_counter;
        }
    }

    /// Returns the keys in chronological order, as in first: gets drawn first, last: gets drawn last.
    pub(crate) fn keys_sorted_chrono(&self) -> Vec<StrokeKey> {
        let mut keys = self.stroke_components.keys().collect::<Vec<StrokeKey>>();
        self.sort_keys_chrono(&mut keys);
        keys
    }

    pub(crate) fn keys_sorted_chrono_intersecting_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        let mut keys = self.key_tree.keys_intersecting_bounds(bounds);
        self.sort_keys_chrono(&mut keys);
        keys
    }

    pub(crate) fn keys_sorted_chrono_in_bounds(&self, bounds: Aabb) -> Vec<StrokeKey> {
        let mut keys = self.key_tree.keys_in_bounds(bounds);
        self.sort_keys_chrono(&mut keys);
        keys
    }

    pub(super) fn sort_keys_chrono(&self, keys: &mut [StrokeKey]) {
        let chrono_components = &self.chrono_components;
        keys.par_sort_unstable_by(|&first, &second| {
            if let Some(first_chrono) = chrono_components.get(first)
                && let Some(second_chrono) = chrono_components.get(second)
            {
                let layer_order = first_chrono.layer.cmp(&second_chrono.layer);

                if layer_order != std::cmp::Ordering::Equal {
                    layer_order
                } else {
                    first_chrono.t.cmp(&second_chrono.t)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
}

impl StrokeStore {
    /// OnePepe: the layer of a stroke.
    pub(crate) fn stroke_layer(&self, key: StrokeKey) -> Option<StrokeLayer> {
        self.chrono_components.get(key).map(|c| c.layer)
    }
}

/// OnePepe: changes to the stacking order (z-order) of strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZOrderChange {
    /// Above all other strokes.
    BringToFront,
    /// Below all other strokes.
    SendToBack,
    /// Above the next overlapping stroke.
    RaiseOne,
    /// Below the previous overlapping stroke.
    LowerOne,
}

/// OnePepe: systems that change the stacking order.
impl StrokeStore {
    /// Change the stacking order of the given strokes.
    ///
    /// Images and PDF pages live on layers below the ink. Reordering them moves them to the ink layer,
    /// so they can be placed above or between ink strokes. Returns true if something changed.
    pub(crate) fn change_z_order(&mut self, keys: &[StrokeKey], change: ZOrderChange) -> bool {
        let keys = keys
            .iter()
            .copied()
            .filter(|&k| self.chrono_components.contains_key(k))
            .collect::<Vec<StrokeKey>>();
        if keys.is_empty() {
            return false;
        }
        let selection_bounds = self.bounds_for_strokes(&keys);

        // Move images and pdf pages to the ink layer. They were drawn below all ink, so they start at its bottom.
        let mut migrated = keys
            .iter()
            .copied()
            .filter(|&k| {
                self.chrono_components
                    .get(k)
                    .is_some_and(|c| matches!(c.layer, StrokeLayer::Image | StrokeLayer::Document))
            })
            .collect::<Vec<StrokeKey>>();
        self.sort_keys_chrono(&mut migrated);
        for &key in migrated.iter() {
            if let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) {
                Arc::make_mut(chrono_comp).layer = StrokeLayer::UserLayer(0);
            }
        }

        let mut layers = keys
            .iter()
            .filter_map(|&k| self.chrono_components.get(k).map(|c| c.layer))
            .collect::<Vec<StrokeLayer>>();
        layers.sort();
        layers.dedup();

        let mut changed = !migrated.is_empty();
        for layer in layers {
            let mut order = self
                .chrono_components
                .iter()
                .filter(|(k, c)| c.layer == layer && !migrated.contains(k))
                .map(|(k, _)| k)
                .collect::<Vec<StrokeKey>>();
            self.sort_keys_chrono(&mut order);
            if layer == StrokeLayer::UserLayer(0) {
                order = [migrated.clone(), order].concat();
            }
            let is_selected = |k: &StrokeKey| keys.contains(k);
            let overlaps = |k: StrokeKey| {
                !is_selected(&k)
                    && !self.trashed(k).unwrap_or(true)
                    && match (selection_bounds, self.bounds_for_strokes(&[k])) {
                        (Some(a), Some(b)) => a.intersects(&b),
                        _ => false,
                    }
            };

            let (selected, others): (Vec<StrokeKey>, Vec<StrokeKey>) =
                order.iter().copied().partition(is_selected);
            let new_order = match change {
                ZOrderChange::BringToFront => [others, selected].concat(),
                ZOrderChange::SendToBack => [selected, others].concat(),
                ZOrderChange::RaiseOne => {
                    let top = order.iter().rposition(is_selected).unwrap_or(0);
                    match order.iter().skip(top + 1).copied().find(|&k| overlaps(k)) {
                        Some(target) => {
                            let pos = others.iter().position(|&k| k == target).unwrap() + 1;
                            [&others[..pos], &selected[..], &others[pos..]].concat()
                        }
                        None => [others, selected].concat(),
                    }
                }
                ZOrderChange::LowerOne => {
                    let bottom = order.iter().position(is_selected).unwrap_or(0);
                    match order[..bottom].iter().rev().copied().find(|&k| overlaps(k)) {
                        Some(target) => {
                            let pos = others.iter().position(|&k| k == target).unwrap();
                            [&others[..pos], &selected[..], &others[pos..]].concat()
                        }
                        None => [selected, others].concat(),
                    }
                }
            };

            if new_order != order {
                changed = true;
            }
            // Renumber the whole layer, so the order is unique and stays above older counters.
            for key in new_order {
                self.chrono_counter += 1;
                if let Some(chrono_comp) = Arc::make_mut(&mut self.chrono_components).get_mut(key) {
                    Arc::make_mut(chrono_comp).t = self.chrono_counter;
                }
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strokes::{ShapeStroke, Stroke};
    use p2d::math::Vector2;
    use rnote_compose::Style;
    use rnote_compose::shapes::{Line, Shape};

    fn line_stroke(x0: f64, x1: f64) -> Stroke {
        Stroke::ShapeStroke(ShapeStroke::new(
            Shape::Line(Line::new(Vector2::new(x0, 0.0), Vector2::new(x1, 10.0))),
            Style::default(),
        ))
    }

    /// Store with four overlapping strokes a, b, c, d (drawn in that order) and one far away stroke e.
    fn store_abcde() -> (StrokeStore, [StrokeKey; 5]) {
        let mut store = StrokeStore::default();
        let a = store.insert_stroke(line_stroke(0.0, 10.0), None);
        let b = store.insert_stroke(line_stroke(0.0, 10.0), None);
        let c = store.insert_stroke(line_stroke(0.0, 10.0), None);
        let e = store.insert_stroke(line_stroke(1000.0, 1010.0), None);
        let d = store.insert_stroke(line_stroke(0.0, 10.0), None);
        (store, [a, b, c, d, e])
    }

    #[test]
    fn bring_to_front_and_send_to_back() {
        let (mut store, [a, b, c, d, e]) = store_abcde();
        assert!(store.change_z_order(&[b], ZOrderChange::BringToFront));
        assert_eq!(store.keys_sorted_chrono(), vec![a, c, e, d, b]);
        assert!(store.change_z_order(&[d], ZOrderChange::SendToBack));
        assert_eq!(store.keys_sorted_chrono(), vec![d, a, c, e, b]);
        // nothing to change
        assert!(!store.change_z_order(&[d], ZOrderChange::SendToBack));
    }

    #[test]
    fn raise_and_lower_skip_strokes_that_do_not_overlap() {
        let (mut store, [a, b, c, d, e]) = store_abcde();
        assert!(store.change_z_order(&[c], ZOrderChange::RaiseOne));
        // c jumps over the far away stroke e, above d
        assert_eq!(store.keys_sorted_chrono(), vec![a, b, e, d, c]);
        assert!(store.change_z_order(&[c], ZOrderChange::LowerOne));
        assert_eq!(store.keys_sorted_chrono(), vec![a, b, e, c, d]);
        assert!(store.change_z_order(&[c], ZOrderChange::LowerOne));
        assert_eq!(store.keys_sorted_chrono(), vec![a, c, b, e, d]);
    }

    #[test]
    fn multiple_selected_keep_relative_order() {
        let (mut store, [a, b, c, d, e]) = store_abcde();
        assert!(store.change_z_order(&[c, a], ZOrderChange::BringToFront));
        assert_eq!(store.keys_sorted_chrono(), vec![b, e, d, a, c]);
    }

    #[test]
    fn images_can_be_brought_above_ink() {
        let (mut store, [a, b, c, d, e]) = store_abcde();
        let img = store.insert_stroke(line_stroke(0.0, 10.0), Some(StrokeLayer::Image));
        let pdf = store.insert_stroke(line_stroke(0.0, 10.0), Some(StrokeLayer::Document));
        // pdf pages and images are drawn below the ink
        assert_eq!(store.keys_sorted_chrono(), vec![pdf, img, a, b, c, e, d]);
        assert!(store.change_z_order(&[img], ZOrderChange::BringToFront));
        assert_eq!(store.keys_sorted_chrono(), vec![pdf, a, b, c, e, d, img]);
        assert!(store.change_z_order(&[pdf], ZOrderChange::RaiseOne));
        assert_eq!(store.keys_sorted_chrono(), vec![a, pdf, b, c, e, d, img]);
    }

    #[test]
    fn selecting_does_not_change_order() {
        let (mut store, [a, b, c, d, e]) = store_abcde();
        store.change_z_order(&[d], ZOrderChange::SendToBack);
        store.set_selected_keys(&[d], true);
        store.set_selected_keys(&[d], false);
        assert_eq!(store.keys_sorted_chrono(), vec![d, a, b, c, e]);
    }
}
