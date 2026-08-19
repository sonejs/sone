use kurbo::{BezPath, Point, Vec2};

#[derive(Debug, Clone, Copy, Default)]
pub struct CornerRadii {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

#[derive(Debug, Clone, Copy)]
struct PathParams {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    p: f64,
    arc_section_length: f64,
    corner_radius: f64,
}

fn r4(v: f64) -> f64 {
    // figma-squircle emits toFixed(4) into an SVG string; match that rounding
    // so control points are bit-identical to the TS output.
    (v * 10_000.0).round() / 10_000.0
}

fn path_params_for_corner(
    corner_radius: f64,
    mut corner_smoothing: f64,
    preserve_smoothing: bool,
    budget: f64,
) -> PathParams {
    let mut p = (1.0 + corner_smoothing) * corner_radius;
    if !preserve_smoothing {
        let max_smoothing = budget / corner_radius - 1.0;
        corner_smoothing = corner_smoothing.min(max_smoothing);
        p = p.min(budget);
    }
    let arc_measure = 90.0 * (1.0 - corner_smoothing);
    let arc_section_length = (arc_measure / 2.0).to_radians().sin() * corner_radius * 2f64.sqrt();
    let angle_alpha = (90.0 - arc_measure) / 2.0;
    let p3_to_p4 = corner_radius * (angle_alpha / 2.0).to_radians().tan();
    let angle_beta = 45.0 * corner_smoothing;
    let c = p3_to_p4 * angle_beta.to_radians().cos();
    let d = c * angle_beta.to_radians().tan();
    let mut b = (p - arc_section_length - c - d) / 3.0;
    let mut a = 2.0 * b;
    if preserve_smoothing && p > budget {
        let p1_to_p3_max = budget - d - arc_section_length - c;
        let min_a = p1_to_p3_max / 6.0;
        let max_b = p1_to_p3_max - min_a;
        b = b.min(max_b);
        a = p1_to_p3_max - b;
        p = p.min(budget);
    }
    PathParams {
        a,
        b,
        c,
        d,
        p,
        arc_section_length,
        corner_radius,
    }
}

const ADJACENTS: [[(usize, bool); 2]; 4] = [
    // topLeft: (topRight, horizontal), (bottomLeft, vertical)
    [(1, true), (3, false)],
    // topRight: (topLeft, horizontal), (bottomRight, vertical)
    [(0, true), (2, false)],
    // bottomRight: (bottomLeft, horizontal), (topRight, vertical)
    [(3, true), (1, false)],
    // bottomLeft: (bottomRight, horizontal), (topLeft, vertical)
    [(2, true), (0, false)],
];

/// figma-squircle's `distributeAndNormalize`, indexed TL, TR, BR, BL.
fn distribute_and_normalize(radii: [f64; 4], width: f64, height: f64) -> ([f64; 4], [f64; 4]) {
    let mut budgets = [-1.0f64; 4];
    let mut radius_map = radii;

    // JS sorts Object.entries in insertion order TL, TR, BL, BR by radius desc.
    let mut order = [0usize, 1, 3, 2];
    order.sort_by(|&x, &y| {
        radius_map[y]
            .partial_cmp(&radius_map[x])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for &corner in &order {
        let radius = radius_map[corner];
        let mut budget = f64::INFINITY;
        for &(adj, horizontal) in &ADJACENTS[corner] {
            let adj_radius = radius_map[adj];
            let candidate = if radius == 0.0 && adj_radius == 0.0 {
                0.0
            } else {
                let side = if horizontal { width } else { height };
                if budgets[adj] >= 0.0 {
                    side - budgets[adj]
                } else {
                    radius / (radius + adj_radius) * side
                }
            };
            if candidate < budget {
                budget = candidate;
            }
        }
        budgets[corner] = budget;
        radius_map[corner] = radius.min(budget);
    }
    (radius_map, budgets)
}

fn arc_to(path: &mut BezPath, from: Point, to: Point, radius: f64) {
    if radius <= 0.0 || (to - from).hypot() < 1e-9 {
        path.line_to(to);
        return;
    }
    let svg = kurbo::SvgArc {
        from,
        to,
        radii: Vec2::new(radius, radius),
        x_rotation: 0.0,
        large_arc: false,
        sweep: true,
    };
    match kurbo::Arc::from_svg_arc(&svg) {
        Some(arc) => {
            arc.to_cubic_beziers(0.1, |p1, p2, p3| path.curve_to(p1, p2, p3));
        }
        None => path.line_to(to),
    }
}

fn rel_curve(path: &mut BezPath, cur: &mut Point, d1: (f64, f64), d2: (f64, f64), d3: (f64, f64)) {
    let p1 = Point::new(cur.x + r4(d1.0), cur.y + r4(d1.1));
    let p2 = Point::new(cur.x + r4(d2.0), cur.y + r4(d2.1));
    let p3 = Point::new(cur.x + r4(d3.0), cur.y + r4(d3.1));
    path.curve_to(p1, p2, p3);
    *cur = p3;
}

fn rel_line(path: &mut BezPath, cur: &mut Point, dx: f64, dy: f64) {
    let p = Point::new(cur.x + r4(dx), cur.y + r4(dy));
    path.line_to(p);
    *cur = p;
}

fn rel_arc(path: &mut BezPath, cur: &mut Point, dx: f64, dy: f64, radius: f64) {
    let to = Point::new(cur.x + r4(dx), cur.y + r4(dy));
    arc_to(path, *cur, to, radius);
    *cur = to;
}

/// Squircle outline for a `width` x `height` box at the origin.
pub fn squircle_path(
    width: f64,
    height: f64,
    radii: CornerRadii,
    corner_smoothing: f64,
    preserve_smoothing: bool,
) -> BezPath {
    let r = [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ];

    let params: [PathParams; 4] = if r[0] == r[1] && r[1] == r[2] && r[2] == r[3] {
        let budget = width.min(height) / 2.0;
        let cr = r[0].min(budget);
        let p = path_params_for_corner(cr, corner_smoothing, preserve_smoothing, budget);
        [p; 4]
    } else {
        let (radii_n, budgets) = distribute_and_normalize(r, width, height);
        [
            path_params_for_corner(radii_n[0], corner_smoothing, preserve_smoothing, budgets[0]),
            path_params_for_corner(radii_n[1], corner_smoothing, preserve_smoothing, budgets[1]),
            path_params_for_corner(radii_n[2], corner_smoothing, preserve_smoothing, budgets[2]),
            path_params_for_corner(radii_n[3], corner_smoothing, preserve_smoothing, budgets[3]),
        ]
    };
    let (tl, tr, br, bl) = (params[0], params[1], params[2], params[3]);

    let mut path = BezPath::new();
    let mut cur = Point::new(r4(width - tr.p), 0.0);
    path.move_to(cur);

    // top-right
    if tr.corner_radius != 0.0 {
        let (a, b, c, d) = (tr.a, tr.b, tr.c, tr.d);
        rel_curve(&mut path, &mut cur, (a, 0.0), (a + b, 0.0), (a + b + c, d));
        rel_arc(
            &mut path,
            &mut cur,
            tr.arc_section_length,
            tr.arc_section_length,
            tr.corner_radius,
        );
        rel_curve(&mut path, &mut cur, (d, c), (d, b + c), (d, a + b + c));
    } else {
        rel_line(&mut path, &mut cur, tr.p, 0.0);
    }

    let p = Point::new(width, r4(height - br.p));
    path.line_to(p);
    cur = p;

    // bottom-right
    if br.corner_radius != 0.0 {
        let (a, b, c, d) = (br.a, br.b, br.c, br.d);
        rel_curve(&mut path, &mut cur, (0.0, a), (0.0, a + b), (-d, a + b + c));
        rel_arc(
            &mut path,
            &mut cur,
            -br.arc_section_length,
            br.arc_section_length,
            br.corner_radius,
        );
        rel_curve(
            &mut path,
            &mut cur,
            (-c, d),
            (-(b + c), d),
            (-(a + b + c), d),
        );
    } else {
        rel_line(&mut path, &mut cur, 0.0, br.p);
    }

    let p = Point::new(r4(bl.p), height);
    path.line_to(p);
    cur = p;

    // bottom-left
    if bl.corner_radius != 0.0 {
        let (a, b, c, d) = (bl.a, bl.b, bl.c, bl.d);
        rel_curve(
            &mut path,
            &mut cur,
            (-a, 0.0),
            (-(a + b), 0.0),
            (-(a + b + c), -d),
        );
        rel_arc(
            &mut path,
            &mut cur,
            -bl.arc_section_length,
            -bl.arc_section_length,
            bl.corner_radius,
        );
        rel_curve(
            &mut path,
            &mut cur,
            (-d, -c),
            (-d, -(b + c)),
            (-d, -(a + b + c)),
        );
    } else {
        rel_line(&mut path, &mut cur, -bl.p, 0.0);
    }

    let p = Point::new(0.0, r4(tl.p));
    path.line_to(p);
    cur = p;

    // top-left
    if tl.corner_radius != 0.0 {
        let (a, b, c, d) = (tl.a, tl.b, tl.c, tl.d);
        rel_curve(
            &mut path,
            &mut cur,
            (0.0, -a),
            (0.0, -(a + b)),
            (d, -(a + b + c)),
        );
        rel_arc(
            &mut path,
            &mut cur,
            tl.arc_section_length,
            -tl.arc_section_length,
            tl.corner_radius,
        );
        rel_curve(&mut path, &mut cur, (c, -d), (b + c, -d), (a + b + c, -d));
    } else {
        rel_line(&mut path, &mut cur, 0.0, -tl.p);
    }

    path.close_path();
    path
}

/// Straight-bevel corners — the `corner: "cut"` shape.
pub fn cut_corner_path(width: f64, height: f64, r: CornerRadii) -> BezPath {
    let mut path = BezPath::new();
    path.move_to((r.top_left, 0.0));
    path.line_to((width - r.top_right, 0.0));
    path.line_to((width, r.top_right));
    path.line_to((width, height - r.bottom_right));
    path.line_to((width - r.bottom_right, height));
    path.line_to((r.bottom_left, height));
    path.line_to((0.0, height - r.bottom_left));
    path.line_to((0.0, r.top_left));
    path.close_path();
    path
}

/// Clamp each entry to `[0, max_radius / 2]`, as `parseRadius` does.
pub fn parse_radius(radius: &[f64], max_radius: f64) -> Vec<f64> {
    radius
        .iter()
        .map(|r| r.max(0.0).min(max_radius / 2.0))
        .collect()
}

/// CSS-style 1/2/4 value expansion used by `createSmoothRoundRect`.
pub fn corner_radii(radius: &[f64]) -> CornerRadii {
    match radius.len() {
        0 => CornerRadii::default(),
        1 => CornerRadii {
            top_left: radius[0],
            top_right: radius[0],
            bottom_right: radius[0],
            bottom_left: radius[0],
        },
        2 => CornerRadii {
            top_left: radius[0],
            bottom_right: radius[0],
            top_right: radius[1],
            bottom_left: radius[1],
        },
        _ => CornerRadii {
            top_left: radius[0],
            top_right: radius.get(1).copied().unwrap_or(0.0),
            bottom_right: radius.get(2).copied().unwrap_or(0.0),
            bottom_left: radius.get(3).copied().unwrap_or(0.0),
        },
    }
}

/// The rounded-rect / squircle / cut-corner outline used for every box.
pub fn box_outline(
    width: f64,
    height: f64,
    radius: &[f64],
    corner_smoothing: Option<f64>,
    shape_cut: bool,
) -> BezPath {
    let clamped = parse_radius(radius, width.min(height));
    let radii = corner_radii(&clamped);
    if shape_cut {
        return cut_corner_path(width, height, radii);
    }
    let cs = corner_smoothing.unwrap_or(0.0).clamp(0.0, 1.0);
    squircle_path(width, height, radii, cs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    #[test]
    fn radius_is_clamped() {
        assert_eq!(parse_radius(&[100.0, -5.0], 40.0), vec![20.0, 0.0]);
    }

    #[test]
    fn radii_shorthand() {
        let one = corner_radii(&[8.0]);
        assert_eq!((one.top_left, one.bottom_left), (8.0, 8.0));
        let two = corner_radii(&[8.0, 2.0]);
        assert_eq!(
            (
                two.top_left,
                two.top_right,
                two.bottom_right,
                two.bottom_left
            ),
            (8.0, 2.0, 8.0, 2.0)
        );
        let four = corner_radii(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            (
                four.top_left,
                four.top_right,
                four.bottom_right,
                four.bottom_left
            ),
            (1.0, 2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn zero_radius_is_a_rectangle() {
        let p = box_outline(100.0, 50.0, &[0.0], None, false);
        let bbox = p.bounding_box();
        assert!((bbox.width() - 100.0).abs() < 1e-6);
        assert!((bbox.height() - 50.0).abs() < 1e-6);
        // 4 lines + close, no curves
        assert!(!p
            .elements()
            .iter()
            .any(|e| matches!(e, kurbo::PathEl::CurveTo(..))));
    }

    #[test]
    fn rounded_box_stays_in_bounds() {
        let p = box_outline(200.0, 120.0, &[24.0], Some(0.6), false);
        let bbox = p.bounding_box();
        assert!(bbox.x0 >= -0.01 && bbox.y0 >= -0.01);
        assert!(bbox.x1 <= 200.01 && bbox.y1 <= 120.01);
        assert!(p
            .elements()
            .iter()
            .any(|e| matches!(e, kurbo::PathEl::CurveTo(..))));
    }

    #[test]
    fn cut_corners_are_octagonal() {
        let p = box_outline(100.0, 100.0, &[10.0], None, true);
        let lines = p
            .elements()
            .iter()
            .filter(|e| matches!(e, kurbo::PathEl::LineTo(..)))
            .count();
        assert_eq!(lines, 7);
    }

    #[test]
    fn asymmetric_radii_are_distributed() {
        let p = box_outline(100.0, 100.0, &[50.0, 0.0, 50.0, 0.0], Some(0.0), false);
        let bbox = p.bounding_box();
        assert!(bbox.x1 <= 100.01 && bbox.y1 <= 100.01);
    }
}
