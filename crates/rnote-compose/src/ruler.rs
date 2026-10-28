//! OnePepe: a virtual ruler, like the one in OneNote.
//!
//! The ruler can be moved by dragging its body and rotated by dragging its rotation handle.
//! A stroke that starts near one of its long edges is stuck to that edge and becomes straight.
//!
//! Everything here is in document coordinates and independent of the UI.

// Imports
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// One of the two long edges of the ruler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulerEdge {
    /// The edge on the side of the negative normal.
    First,
    /// The edge on the side of the positive normal.
    Second,
}

/// What a position hits on the ruler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerHit {
    /// The rotation handle.
    RotateHandle,
    /// Near one of the long edges, where strokes are snapped.
    Edge(RulerEdge),
    /// The body of the ruler, where it can be dragged.
    Body,
    /// Nothing.
    None,
}

/// A virtual ruler.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ruler {
    /// Center of the ruler.
    pub center: Vector2,
    /// Rotation angle in radians.
    pub angle: f64,
    /// Length along its direction.
    pub length: f64,
    /// Width across its direction.
    pub width: f64,
    /// OnePepe: infinite mode, a line across the whole canvas (the length is ignored).
    #[serde(default)]
    pub infinite: bool,
    /// OnePepe: while rotating, snap to steps of 15° (with Shift). Not saved.
    #[serde(skip)]
    pub snap_to_steps: bool,
}

impl Default for Ruler {
    fn default() -> Self {
        Self {
            center: Vector2::ZERO,
            angle: 0.0,
            length: 800.0,
            width: 80.0,
            infinite: false,
            snap_to_steps: false,
        }
    }
}

impl Ruler {
    /// Angles are snapped to multiples of this (in degrees) when close enough.
    pub const ANGLE_SNAP_STEP_DEG: f64 = 15.0;
    /// The maximum angle difference (in degrees) for snapping.
    pub const ANGLE_SNAP_TOLERANCE_DEG: f64 = 1.5;

    /// Unit vector along the ruler.
    pub fn direction(&self) -> Vector2 {
        Vector2::new(self.angle.cos(), self.angle.sin())
    }

    /// Unit vector across the ruler.
    pub fn normal(&self) -> Vector2 {
        Vector2::new(-self.angle.sin(), self.angle.cos())
    }

    /// Position in ruler coordinates: (along, across), relative to the center.
    pub fn to_local(&self, pos: Vector2) -> Vector2 {
        let d = pos - self.center;
        Vector2::new(d.dot(self.direction()), d.dot(self.normal()))
    }

    /// Position in document coordinates from ruler coordinates.
    pub fn from_local(&self, local: Vector2) -> Vector2 {
        self.center + self.direction() * local.x + self.normal() * local.y
    }

    fn edge_offset(&self, edge: RulerEdge) -> f64 {
        match edge {
            RulerEdge::First => -self.width * 0.5,
            RulerEdge::Second => self.width * 0.5,
        }
    }

    /// The center of the rotation handle.
    pub fn rotate_handle_center(&self) -> Vector2 {
        if self.infinite {
            // an infinite ruler has no end: the handle is at its center
            return self.center;
        }
        self.from_local(Vector2::new(self.length * 0.5 - self.width * 0.5, 0.0))
    }

    /// OnePepe: the part of the ruler to draw for a view: an infinite ruler, as a finite one across the view
    /// (its center stays on the line, so the ticks do not move); a finite one, as it is.
    pub fn visible_part(&self, view_corners: [Vector2; 4]) -> Ruler {
        if !self.infinite {
            return *self;
        }
        let along = view_corners.map(|c| self.to_local(c).x);
        let min = along.iter().cloned().fold(f64::INFINITY, f64::min) - self.width;
        let max = along.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + self.width;
        // the visible part is centered on a multiple of the tick spacing, so the ticks stay in place
        let mid = ((min + max) * 0.5 / Self::TICK_STEP).round() * Self::TICK_STEP;
        Ruler {
            center: self.from_local(Vector2::new(mid, 0.0)),
            length: (max - min) + 2.0 * (mid - (min + max) * 0.5).abs(),
            infinite: false,
            ..*self
        }
    }

    /// OnePepe: distance between ticks (design guide: 8 px).
    pub const TICK_STEP: f64 = 8.0;
    /// OnePepe: distance between numbers (design guide: 80 px).
    pub const NUMBER_STEP: f64 = 80.0;

    /// The radius of the rotation handle.
    pub fn rotate_handle_radius(&self) -> f64 {
        self.width * 0.3
    }

    /// The four corners of the ruler.
    pub fn corners(&self) -> [Vector2; 4] {
        let (hl, hw) = (self.length * 0.5, self.width * 0.5);
        [
            self.from_local(Vector2::new(-hl, -hw)),
            self.from_local(Vector2::new(hl, -hw)),
            self.from_local(Vector2::new(hl, hw)),
            self.from_local(Vector2::new(-hl, hw)),
        ]
    }

    /// What the position hits. `snap_distance` is how close to an edge a position must be to be snapped.
    pub fn hit(&self, pos: Vector2, snap_distance: f64) -> RulerHit {
        if (pos - self.rotate_handle_center()).length() <= self.rotate_handle_radius() {
            return RulerHit::RotateHandle;
        }
        let local = self.to_local(pos);
        if !self.infinite && local.x.abs() > self.length * 0.5 + snap_distance {
            return RulerHit::None;
        }
        for edge in [RulerEdge::First, RulerEdge::Second] {
            if (local.y - self.edge_offset(edge)).abs() <= snap_distance {
                return RulerHit::Edge(edge);
            }
        }
        if local.y.abs() < self.width * 0.5 && (self.infinite || local.x.abs() <= self.length * 0.5)
        {
            return RulerHit::Body;
        }
        RulerHit::None
    }

    /// Projects the position onto the line of the given edge.
    pub fn project_to_edge(&self, pos: Vector2, edge: RulerEdge) -> Vector2 {
        let local = self.to_local(pos);
        self.from_local(Vector2::new(local.x, self.edge_offset(edge)))
    }

    /// The angle in degrees, in the range [0, 180), as it is shown to the user.
    ///
    /// The y axis of the document points down, so the angle is negated to count counter-clockwise.
    pub fn angle_degrees(&self) -> f64 {
        let deg = (-self.angle).to_degrees().rem_euclid(180.0);
        // avoid showing 180 for values that are almost 0
        // + 0.0 turns -0 into 0 (shown as "-0°" otherwise)
        if (deg - 180.0).abs() < 1e-9 {
            0.0
        } else {
            deg + 0.0
        }
    }

    /// Sets the angle, snapping it to multiples of [`Self::ANGLE_SNAP_STEP_DEG`] when close.
    pub fn set_angle_snapped(&mut self, angle: f64) {
        let deg = angle.to_degrees();
        let snapped = (deg / Self::ANGLE_SNAP_STEP_DEG).round() * Self::ANGLE_SNAP_STEP_DEG;
        self.angle =
            if self.snap_to_steps || (deg - snapped).abs() <= Self::ANGLE_SNAP_TOLERANCE_DEG {
                snapped.to_radians()
            } else {
                angle
            };
    }
}

/// The state of an interaction of the pen with the ruler, from pen down until pen up.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RulerInteraction {
    /// No pen down.
    #[default]
    Idle,
    /// Drawing without the ruler.
    Free,
    /// Drawing stuck to an edge.
    Snapping(RulerEdge),
    /// Moving the ruler. Stores the offset from the pen to the ruler center.
    Dragging(Vector2),
    /// Rotating the ruler. Stores the angle between the pen and the ruler at the start.
    Rotating(f64),
}

impl RulerInteraction {
    /// Handles a pen down event (also repeated while moving).
    ///
    /// Returns the position the pen should draw at, or `None` if the event was used for the ruler and must not draw.
    pub fn pen_down(
        &mut self,
        ruler: &mut Ruler,
        pos: Vector2,
        snap_distance: f64,
    ) -> Option<Vector2> {
        if *self == RulerInteraction::Idle {
            *self = match ruler.hit(pos, snap_distance) {
                RulerHit::RotateHandle => {
                    let v = pos - ruler.center;
                    RulerInteraction::Rotating(v.y.atan2(v.x) - ruler.angle)
                }
                RulerHit::Edge(edge) => RulerInteraction::Snapping(edge),
                RulerHit::Body => RulerInteraction::Dragging(ruler.center - pos),
                RulerHit::None => RulerInteraction::Free,
            };
        }
        match *self {
            RulerInteraction::Idle | RulerInteraction::Free => Some(pos),
            RulerInteraction::Snapping(edge) => Some(ruler.project_to_edge(pos, edge)),
            RulerInteraction::Dragging(offset) => {
                ruler.center = pos + offset;
                None
            }
            RulerInteraction::Rotating(grab_angle) => {
                let v = pos - ruler.center;
                if v.length() > f64::EPSILON {
                    ruler.set_angle_snapped(v.y.atan2(v.x) - grab_angle);
                }
                None
            }
        }
    }

    /// Handles a pen up event. Returns the position to draw at, like [`Self::pen_down`], and resets the state.
    pub fn pen_up(
        &mut self,
        ruler: &mut Ruler,
        pos: Vector2,
        snap_distance: f64,
    ) -> Option<Vector2> {
        let res = if *self == RulerInteraction::Idle {
            // an up without a down: never start dragging here
            Some(pos)
        } else {
            self.pen_down(ruler, pos, snap_distance)
        };
        *self = RulerInteraction::Idle;
        res
    }

    /// Whether the ruler is currently being moved or rotated.
    pub fn is_modifying_ruler(&self) -> bool {
        matches!(
            self,
            RulerInteraction::Dragging(_) | RulerInteraction::Rotating(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    const SNAP: f64 = 15.0;

    fn ruler(angle_deg: f64) -> Ruler {
        Ruler {
            center: Vector2::new(100.0, 200.0),
            angle: angle_deg.to_radians(),
            length: 600.0,
            width: 60.0,
            infinite: false,
            snap_to_steps: false,
        }
    }

    /// Distance of a point to the line of an edge.
    fn dist_to_edge(r: &Ruler, p: Vector2, edge: RulerEdge) -> f64 {
        (r.to_local(p).y - r.edge_offset(edge)).abs()
    }

    #[test]
    fn infinite_ruler_has_no_ends() {
        let mut r = ruler(0.0);
        r.infinite = true;
        // far beyond the length: still the edge and the body
        assert_eq!(
            r.hit(Vector2::new(5000.0, 170.0), SNAP),
            RulerHit::Edge(RulerEdge::First)
        );
        assert_eq!(r.hit(Vector2::new(-5000.0, 200.0), SNAP), RulerHit::Body);
        assert_eq!(r.hit(r.center, SNAP), RulerHit::RotateHandle);
        // the part to draw covers the view, with ticks in place
        let view = [
            Vector2::new(1000.0, 0.0),
            Vector2::new(3000.0, 0.0),
            Vector2::new(3000.0, 500.0),
            Vector2::new(1000.0, 500.0),
        ];
        let part = r.visible_part(view);
        assert!(!part.infinite);
        let local_center = r.to_local(part.center).x;
        assert!((local_center / Ruler::TICK_STEP).fract().abs() < 1e-9);
        assert!(
            r.to_local(part.from_local(Vector2::new(-part.length * 0.5, 0.0)))
                .x
                <= 1000.0 - 100.0
        );
        assert!(
            r.to_local(part.from_local(Vector2::new(part.length * 0.5, 0.0)))
                .x
                >= 3000.0 - 100.0
        );
    }

    #[test]
    fn snap_to_steps_with_shift() {
        let mut r = ruler(0.0);
        r.set_angle_snapped(20f64.to_radians());
        assert!((r.angle.to_degrees() - 20.0).abs() < 1e-9);
        r.snap_to_steps = true;
        r.set_angle_snapped(20f64.to_radians());
        assert!((r.angle.to_degrees() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn local_coordinates_roundtrip() {
        let r = ruler(33.0);
        let p = Vector2::new(-12.0, 345.0);
        assert_relative_eq!(r.from_local(r.to_local(p)), p, epsilon = 1e-9);
    }

    #[test]
    fn hit_regions() {
        let r = ruler(0.0);
        // edges are at y = 170 and y = 230
        assert_eq!(
            r.hit(Vector2::new(0.0, 160.0), SNAP),
            RulerHit::Edge(RulerEdge::First)
        );
        assert_eq!(
            r.hit(Vector2::new(0.0, 240.0), SNAP),
            RulerHit::Edge(RulerEdge::Second)
        );
        assert_eq!(r.hit(Vector2::new(0.0, 200.0), SNAP), RulerHit::Body);
        assert_eq!(r.hit(Vector2::new(0.0, 100.0), SNAP), RulerHit::None);
        assert_eq!(r.hit(Vector2::new(1000.0, 170.0), SNAP), RulerHit::None);
        assert_eq!(
            r.hit(r.rotate_handle_center(), SNAP),
            RulerHit::RotateHandle
        );
    }

    #[test]
    fn wobbly_stroke_along_edge_becomes_straight() {
        for angle in [0.0, 30.0, 45.0, 90.0, 137.0] {
            let r0 = ruler(angle);
            let mut r = r0;
            let mut interaction = RulerInteraction::default();
            // a wobbly stroke slightly outside the first edge
            let points = (0..50)
                .map(|i| {
                    let along = -250.0 + i as f64 * 10.0;
                    let across = -30.0 - 6.0 + 5.0 * (i as f64 * 0.7).sin();
                    r0.from_local(Vector2::new(along, across))
                })
                .collect::<Vec<Vector2>>();
            let mut drawn = Vec::new();
            for (i, &p) in points.iter().enumerate() {
                let res = if i == points.len() - 1 {
                    interaction.pen_up(&mut r, p, SNAP)
                } else {
                    interaction.pen_down(&mut r, p, SNAP)
                };
                drawn.push(res.expect("drawing near the edge must not be consumed"));
            }
            for p in drawn {
                assert!(dist_to_edge(&r, p, RulerEdge::First) < 1e-9);
            }
            assert_eq!(r, r0, "the ruler must not move while drawing");
            assert_eq!(interaction, RulerInteraction::Idle);
        }
    }

    #[test]
    fn stroke_stays_snapped_after_leaving_the_snap_zone() {
        let mut r = ruler(0.0);
        let mut interaction = RulerInteraction::default();
        let start = interaction
            .pen_down(&mut r, Vector2::new(0.0, 235.0), SNAP)
            .unwrap();
        assert_relative_eq!(start.y, 230.0);
        // far away from the edge, but the stroke started snapped
        let p = interaction
            .pen_down(&mut r, Vector2::new(50.0, 300.0), SNAP)
            .unwrap();
        assert_relative_eq!(p, Vector2::new(50.0, 230.0));
    }

    #[test]
    fn free_stroke_is_not_modified() {
        let mut r = ruler(0.0);
        let mut interaction = RulerInteraction::default();
        let p0 = Vector2::new(0.0, 100.0);
        assert_eq!(interaction.pen_down(&mut r, p0, SNAP), Some(p0));
        // crossing the ruler later does not snap
        let p1 = Vector2::new(0.0, 172.0);
        assert_eq!(interaction.pen_down(&mut r, p1, SNAP), Some(p1));
    }

    #[test]
    fn drag_moves_the_ruler() {
        let mut r = ruler(0.0);
        let mut interaction = RulerInteraction::default();
        assert_eq!(
            interaction.pen_down(&mut r, Vector2::new(10.0, 200.0), SNAP),
            None
        );
        assert!(interaction.is_modifying_ruler());
        assert_eq!(
            interaction.pen_down(&mut r, Vector2::new(60.0, 260.0), SNAP),
            None
        );
        assert_eq!(
            interaction.pen_up(&mut r, Vector2::new(60.0, 260.0), SNAP),
            None
        );
        assert_relative_eq!(r.center, Vector2::new(150.0, 260.0));
        assert_eq!(interaction, RulerInteraction::Idle);
    }

    #[test]
    fn rotate_handle_rotates_and_snaps() {
        let mut r = ruler(0.0);
        let mut interaction = RulerInteraction::default();
        let handle = r.rotate_handle_center();
        let center = r.center;
        let radius = (handle - center).length();
        assert_eq!(interaction.pen_down(&mut r, handle, SNAP), None);
        // rotate to 40 degrees: not snapped
        let a = 40.0_f64.to_radians();
        interaction.pen_down(
            &mut r,
            center + Vector2::new(a.cos(), a.sin()) * radius,
            SNAP,
        );
        assert_relative_eq!(r.angle, a, epsilon = 1e-9);
        // rotate to 44.2 degrees: snapped to 45
        let a = 44.2_f64.to_radians();
        interaction.pen_up(
            &mut r,
            center + Vector2::new(a.cos(), a.sin()) * radius,
            SNAP,
        );
        assert_relative_eq!(r.angle, PI / 4.0, epsilon = 1e-9);
    }

    #[test]
    fn angle_shown_to_user() {
        assert_relative_eq!(ruler(0.0).angle_degrees(), 0.0);
        // y points down: a negative angle is counter-clockwise on screen
        assert_relative_eq!(ruler(-30.0).angle_degrees(), 30.0, epsilon = 1e-9);
        assert_relative_eq!(ruler(30.0).angle_degrees(), 150.0, epsilon = 1e-9);
        assert_relative_eq!(ruler(-180.0).angle_degrees(), 0.0, epsilon = 1e-9);
    }
}
