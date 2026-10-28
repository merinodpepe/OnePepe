// Imports
use crate::Image;
use crate::Svg;
use anyhow::Context;
use p2d::bounding_volume::Aabb;
use p2d::math::Vector2;
use rnote_compose::Color;
use serde::{Deserialize, Serialize};
use svg::Node;
use svg::node::element;

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "pattern_style")]
pub enum PatternStyle {
    #[serde(rename = "none")]
    None = 0,
    #[serde(rename = "lines")]
    Lines,
    #[serde(rename = "grid")]
    Grid,
    #[serde(rename = "dots")]
    Dots,
    #[serde(rename = "isometric_grid")]
    IsometricGrid,
    #[serde(rename = "isometric_dots")]
    IsometricDots,
}

impl Default for PatternStyle {
    fn default() -> Self {
        Self::Dots
    }
}

impl TryFrom<u32> for PatternStyle {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("PatternStyle try_from::<u32>() for value {} failed", value)
        })
    }
}

/// 3_f64.sqrt()
const SQRT_THREE: f64 = 1.7320508075688772;
/// 3_f64.sqrt() / 2_f64
const HALF_SQRT_THREE: f64 = SQRT_THREE / 2_f64;
/// 3_f64.sqrt() / 4_f64
const QUARTER_SQRT_THREE: f64 = SQRT_THREE / 4_f64;

fn gen_hline_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_hline_pattern";

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", bounds.extents()[0])
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", line_offset)
                    .set("x2", bounds.extents()[0])
                    .set("y2", line_offset),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_grid_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_grid_pattern";

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", 0_f64)
                    .set("y1", line_offset)
                    .set("x2", column_spacing)
                    .set("y2", line_offset),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset)
                    .set("y2", row_spacing),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_dots_pattern(
    bounds: Aabb,
    row_spacing: f64,
    column_spacing: f64,
    color: Color,
    dots_width: f64,
) -> svg::node::element::Element {
    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_dots_pattern";

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", column_spacing)
            .set("height", row_spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Rectangle::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("x", 0_f64)
                    .set("y", 0_f64)
                    .set("width", dots_width)
                    .set("height", dots_width)
                    .set("rx", dots_width / 3.0)
                    .set("ry", dots_width / 3.0),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));
    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn calc_width_iso_pattern(spacing: f64) -> f64 {
    spacing * SQRT_THREE
}

fn gen_iso_grid_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    line_width: f64,
) -> svg::node::element::Element {
    // spacing: side length of the equilateral triangle
    // pattern_width: two times the height of the equilateral triangle

    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_iso_grid_pattern";
    let pattern_width = calc_width_iso_pattern(spacing);

    let line_offset = line_width * 0.5;

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", pattern_width)
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset + pattern_width)
                    .set("y2", spacing),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", spacing)
                    .set("x2", line_offset + pattern_width)
                    .set("y2", 0_f64),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset + pattern_width * 0.5)
                    .set("y1", 0_f64)
                    .set("x2", line_offset + pattern_width * 0.5)
                    .set("y2", spacing),
            )
            .add(
                element::Line::new()
                    .set("stroke-width", line_width)
                    .set("stroke", color.to_css_color_attr())
                    .set("x1", line_offset)
                    .set("y1", 0_f64)
                    .set("x2", line_offset)
                    .set("y2", spacing),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));

    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

fn gen_iso_dots_pattern(
    bounds: Aabb,
    spacing: f64,
    color: Color,
    hexagon_height: f64,
) -> svg::node::element::Element {
    // spacing: side length of the equilateral triangle
    // pattern_width: two times the height of the equilateral triangle

    let pattern_id = rnote_compose::utils::svg_random_id_prefix() + "_bg_iso_dots_pattern";
    let pattern_width = calc_width_iso_pattern(spacing);

    let hexagon_path = |x_offset: f64, y_offset: f64| {
        element::path::Data::new()
            .move_to((
                x_offset + QUARTER_SQRT_THREE * hexagon_height,
                y_offset + hexagon_height,
            ))
            .line_to((
                x_offset + HALF_SQRT_THREE * hexagon_height,
                y_offset + 0.75 * hexagon_height,
            ))
            .line_to((
                x_offset + HALF_SQRT_THREE * hexagon_height,
                y_offset + 0.25 * hexagon_height,
            ))
            .line_to((x_offset + QUARTER_SQRT_THREE * hexagon_height, y_offset))
            .line_to((x_offset, y_offset + 0.25 * hexagon_height))
            .line_to((x_offset, y_offset + 0.75 * hexagon_height))
            .close()
    };

    let pattern = element::Definitions::new().add(
        element::Pattern::new()
            .set("id", pattern_id.as_str())
            .set("x", 0_f64)
            .set("y", 0_f64)
            .set("width", pattern_width)
            .set("height", spacing)
            .set("patternUnits", "userSpaceOnUse")
            .set("patternContentUnits", "userSpaceOnUse")
            .add(
                element::Path::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("d", hexagon_path(0.0, 0.0)),
            )
            .add(
                element::Path::new()
                    .set("stroke", "none")
                    .set("fill", color.to_css_color_attr())
                    .set("d", hexagon_path(pattern_width * 0.5, spacing * 0.5)),
            ),
    );

    let mut rect = element::Rectangle::new().set("fill", format!("url(#{pattern_id})"));
    rect.assign("x", format!("{}px", bounds.mins[0]));
    rect.assign("y", format!("{}px", bounds.mins[1]));
    rect.assign("width", format!("{}px", bounds.extents()[0]));
    rect.assign("height", format!("{}px", bounds.extents()[1]));

    let group = element::Group::new().add(pattern).add(rect);
    group.into()
}

/// OnePepe: a vertical margin line on every page, like the red line of a notebook. It is not part of the .rnote
/// format: OnePepe keeps it in the hidden file of the page.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarginLine {
    /// Distance from the left edge of each page.
    pub x: f64,
    pub color: Color,
    /// The width of the pages, to repeat the line on every column of pages.
    pub page_width: f64,
}

impl MarginLine {
    pub const WIDTH: f64 = 1.0;
    pub const COLOR_DEFAULT: Color = Color {
        r: 0.9,
        g: 0.45,
        b: 0.5,
        a: 1.0,
    };

    /// The x positions of the line inside `bounds`.
    pub fn positions(&self, bounds: Aabb) -> Vec<f64> {
        if self.page_width <= 0.0 {
            return vec![];
        }
        let first = ((bounds.mins[0] - self.x) / self.page_width).floor() as i64;
        let last = ((bounds.maxs[0] - self.x) / self.page_width).ceil() as i64;
        (first..=last)
            .map(|k| k as f64 * self.page_width + self.x)
            .filter(|x| *x >= bounds.mins[0] && *x <= bounds.maxs[0])
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "background")]
pub struct Background {
    #[serde(rename = "color")]
    pub color: Color,
    #[serde(rename = "pattern")]
    pub pattern: PatternStyle,
    #[serde(
        rename = "pattern_size",
        with = "rnote_compose::serialize::glam_vector2_dp3"
    )]
    pub pattern_size: Vector2,
    #[serde(rename = "pattern_color")]
    pub pattern_color: Color,
    /// OnePepe: the width of the lines of the pattern (default 0.5). Not saved in the .rnote file: OnePepe keeps it
    /// in the hidden file of the page.
    #[serde(skip)]
    pub onepepe_line_width: Option<f64>,
    /// OnePepe: the margin line. Not saved in the .rnote file either.
    #[serde(skip)]
    pub onepepe_margin: Option<MarginLine>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Self::COLOR_DEFAULT,
            pattern: PatternStyle::default(),
            pattern_size: Self::PATTERN_SIZE_DEFAULT,
            pattern_color: Self::PATTERN_COLOR_DEFAULT,
            onepepe_line_width: None,
            onepepe_margin: None,
        }
    }
}

impl Background {
    const LINE_WIDTH: f64 = 0.5;
    const DOTS_WIDTH: f64 = 1.5;
    const HEXAGON_HEIGHT: f64 = 2.0;

    const TILE_MAX_SIZE: f64 = 128.0;
    const COLOR_DEFAULT: Color = Color::WHITE;
    const PATTERN_SIZE_DEFAULT: Vector2 = Vector2::new(32.0, 32.0);
    const PATTERN_COLOR_DEFAULT: Color = Color {
        r: 0.8,
        g: 0.9,
        b: 1.0,
        a: 1.0,
    };

    /// Calculates the tile size as multiple of pattern_size with max size TITLE_MAX_SIZE
    pub(crate) fn tile_size(&self) -> Vector2 {
        let pattern_size = match self.pattern {
            PatternStyle::None => Vector2::new(Self::TILE_MAX_SIZE, Self::TILE_MAX_SIZE),
            PatternStyle::Lines => Vector2::new(Self::TILE_MAX_SIZE, self.pattern_size[1]),
            PatternStyle::IsometricGrid | PatternStyle::IsometricDots => Vector2::new(
                calc_width_iso_pattern(self.pattern_size[1]),
                self.pattern_size[1],
            ),
            _ => self.pattern_size,
        };
        let tile_factor = Vector2::splat(Self::TILE_MAX_SIZE) / pattern_size;
        let tile_width = if tile_factor[0] > 1.0 {
            tile_factor[0].floor() * pattern_size[0]
        } else {
            pattern_size[0]
        };
        let tile_height = if tile_factor[1] > 1.0 {
            tile_factor[1].floor() * pattern_size[1]
        } else {
            pattern_size[1]
        };

        Vector2::new(tile_width, tile_height)
    }

    /// Generate the background svg, without Xml header or Svg root.
    pub(crate) fn gen_svg(
        &self,
        bounds: Aabb,
        with_pattern: bool,
        optimize_printing: bool,
    ) -> Result<Svg, anyhow::Error> {
        self.gen_svg_inner(bounds, with_pattern, optimize_printing, true)
    }

    /// OnePepe: the line width of the pattern.
    pub fn line_width(&self) -> f64 {
        self.onepepe_line_width.unwrap_or(Self::LINE_WIDTH)
    }

    /// `with_margin`: the margin line is drawn for whole pages, never in the repeated tile of the screen.
    fn gen_svg_inner(
        &self,
        bounds: Aabb,
        with_pattern: bool,
        optimize_printing: bool,
        with_margin: bool,
    ) -> Result<Svg, anyhow::Error> {
        let (color, pattern_color) = if optimize_printing {
            if self.color.luma() > 0.5 {
                // original background color is bright, don't invert pattern color
                (Color::WHITE, self.pattern_color)
            } else {
                // original background color is dark, invert pattern color
                (
                    Color::WHITE,
                    self.pattern_color.to_inverted_brightness_color(),
                )
            }
        } else {
            (self.color, self.pattern_color)
        };

        // background color
        let mut color_rect = element::Rectangle::new().set("fill", color.to_css_color_attr());
        color_rect.assign("x", format!("{}px", bounds.mins[0]));
        color_rect.assign("y", format!("{}px", bounds.mins[1]));
        color_rect.assign("width", format!("{}px", bounds.extents()[0]));
        color_rect.assign("height", format!("{}px", bounds.extents()[1]));

        let mut svg_group = element::Group::new();
        svg_group = svg_group.add(color_rect);

        if with_pattern {
            match self.pattern {
                PatternStyle::None => {}
                PatternStyle::Lines => {
                    svg_group = svg_group.add(gen_hline_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        self.line_width(),
                    ));
                }
                PatternStyle::Grid => {
                    svg_group = svg_group.add(gen_grid_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        pattern_color,
                        self.line_width(),
                    ));
                }
                PatternStyle::Dots => {
                    svg_group = svg_group.add(gen_dots_pattern(
                        bounds,
                        self.pattern_size[1],
                        self.pattern_size[0],
                        pattern_color,
                        Self::DOTS_WIDTH,
                    ));
                }
                PatternStyle::IsometricGrid => {
                    svg_group = svg_group.add(gen_iso_grid_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        self.line_width(),
                    ));
                }
                PatternStyle::IsometricDots => {
                    svg_group = svg_group.add(gen_iso_dots_pattern(
                        bounds,
                        self.pattern_size[1],
                        pattern_color,
                        Self::HEXAGON_HEIGHT,
                    ));
                }
            }
        }

        if with_pattern
            && with_margin
            && let Some(margin) = self.onepepe_margin
        {
            for x in margin.positions(bounds) {
                let line = element::Line::new()
                    .set("x1", x)
                    .set("y1", bounds.mins[1])
                    .set("x2", x)
                    .set("y2", bounds.maxs[1])
                    .set("stroke", margin.color.to_css_color_attr())
                    .set("stroke-width", MarginLine::WIDTH);
                svg_group = svg_group.add(line);
            }
        }

        let svg_data = rnote_compose::utils::svg_node_to_string(&svg_group)
            .context("Converting Svg group node to String failed.")?;

        Ok(Svg { svg_data, bounds })
    }

    pub(crate) fn gen_tile_image(&self, image_scale: f64) -> Result<Image, anyhow::Error> {
        let tile_bounds = Aabb::new(Vector2::ZERO, self.tile_size());
        self.gen_svg_inner(tile_bounds, true, false, false)?
            .gen_image(image_scale)
    }

    pub(crate) fn draw_to_cairo(
        &self,
        cx: &cairo::Context,
        bounds: Aabb,
        with_pattern: bool,
        optimize_printing: bool,
    ) -> anyhow::Result<()> {
        self.gen_svg(bounds, with_pattern, optimize_printing)?
            .draw_to_cairo(cx)
    }
}

#[cfg(test)]
mod margin_tests {
    use super::*;

    #[test]
    fn margin_on_every_page_column() {
        let margin = MarginLine {
            x: 80.0,
            color: MarginLine::COLOR_DEFAULT,
            page_width: 800.0,
        };
        let xs = margin.positions(Aabb::new(
            Vector2::new(-900.0, 0.0),
            Vector2::new(1700.0, 10.0),
        ));
        assert_eq!(xs, vec![-720.0, 80.0, 880.0, 1680.0]);
        let mut background = Background {
            onepepe_margin: Some(margin),
            onepepe_line_width: Some(2.0),
            pattern: PatternStyle::Lines,
            ..Default::default()
        };
        let page = Aabb::new(Vector2::ZERO, Vector2::new(800.0, 1000.0));
        let svg = background.gen_svg(page, true, false).unwrap().svg_data;
        assert!(svg.contains("x1=\"80\""));
        assert!(svg.contains("stroke-width=\"2\""));
        // never in the repeated tile of the screen, and never in the .rnote file
        background.pattern = PatternStyle::None;
        assert!(
            !background
                .gen_svg_inner(page, true, false, false)
                .unwrap()
                .svg_data
                .contains("<line")
        );
        let json = serde_json::to_string(&background).unwrap();
        assert!(!json.contains("onepepe"));
    }
}
