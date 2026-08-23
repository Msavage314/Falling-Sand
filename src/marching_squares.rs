#[derive(Debug)]

pub struct Polygon {
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug)]
pub struct Segment {
    pub start: (f32, f32),
    pub end: (f32, f32),
}

use crate::{materials::MaterialID, simulation::Grid};
use core::hash::Hash;
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::triangulation::Triangulation;
use i_triangle::float::triangulator::Triangulator;
use std::collections::HashMap;
pub fn generate_lines(grid: &Grid) -> Vec<Segment> {
    let cells: Vec<usize> = grid
        .cells
        .iter()
        .map(|c| {
            if c.material == MaterialID::Empty {
                0
            } else {
                1
            }
        })
        .collect();
    // treat anything outside grid as empty
    let sample = |x: i32, y: i32| -> usize {
        if x < 0 || y < 0 || x as usize >= grid.width || y as usize >= grid.height {
            0
        } else {
            cells[y as usize * grid.width + x as usize]
        }
    };

    let mut segments: Vec<Segment> = Vec::new();
    // Marching squares looks at each square of cells,
    // so we iterate one less in each direction.
    for x in -1..grid.width as i32 {
        for y in -1..grid.height as i32 {
            let p1 = sample(x, y);
            let p2 = sample(x + 1, y);
            let p3 = sample(x + 1, y + 1);
            let p4 = sample(x, y + 1);
            let case: u8 = (p1 | (p2 << 1) | (p3 << 2) | (p4 << 3)) as u8;

            let top = (x as f32 + 0.5, y as f32);
            let right = (x as f32 + 1.0, y as f32 + 0.5);
            let bottom = (x as f32 + 0.5, y as f32 + 1.0);
            let left = (x as f32, y as f32 + 0.5);

            match case {
                0 | 15 => {}

                1 => segments.push(Segment {
                    start: left,
                    end: top,
                }),
                14 => segments.push(Segment {
                    start: top,
                    end: left,
                }),

                2 => segments.push(Segment {
                    start: top,
                    end: right,
                }),
                13 => segments.push(Segment {
                    start: right,
                    end: top,
                }),

                3 => segments.push(Segment {
                    start: left,
                    end: right,
                }),
                12 => segments.push(Segment {
                    start: right,
                    end: left,
                }),

                4 => segments.push(Segment {
                    start: right,
                    end: bottom,
                }),
                11 => segments.push(Segment {
                    start: bottom,
                    end: right,
                }),

                5 => {
                    segments.push(Segment {
                        start: right,
                        end: top,
                    });
                    segments.push(Segment {
                        start: left,
                        end: bottom,
                    });
                }

                6 => segments.push(Segment {
                    start: top,
                    end: bottom,
                }),
                9 => segments.push(Segment {
                    start: bottom,
                    end: top,
                }),

                7 => segments.push(Segment {
                    start: left,
                    end: bottom,
                }),
                8 => segments.push(Segment {
                    start: bottom,
                    end: left,
                }),

                10 => {
                    segments.push(Segment {
                        start: top,
                        end: left,
                    });
                    segments.push(Segment {
                        start: bottom,
                        end: right,
                    });
                }

                _ => unreachable!(),
            }
        }
    }
    return segments;
}

fn point_key(p: (f32, f32)) -> (i64, i64) {
    const SCALE: f32 = 1000.0;
    return ((p.0 * SCALE).round() as i64, (p.1 * SCALE).round() as i64);
}
// computes the signed area of a set of points.
fn signed_area(points: &[(f32, f32)]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];
        area += x1 * y1 - x2 * y1;
    }
    area * 0.5
}
fn point_in_polygon(point: (f32, f32), polygon: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let n = polygon.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > point.1) != (yj > point.1))
            && (point.0 < (xj - xi) * (point.1 - yi) / (yj - yi) + xi)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn stitch_polygons(segments: &Vec<Segment>) -> Vec<Polygon> {
    // map each start point to a list of segment indices which start there
    let mut by_start: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, seg) in segments.iter().enumerate() {
        by_start.entry(point_key(seg.start)).or_default().push(i);
    }
    let mut used = vec![false; segments.len()];
    let mut polygons = Vec::new();

    // go through each segment index
    for start_idx in 0..segments.len() {
        if used[start_idx] {
            continue;
        }

        let mut points: Vec<(f32, f32)> = Vec::new();
        let first_key = point_key(segments[start_idx].start);
        let mut current = start_idx;

        loop {
            if used[current] {
                break;
            }

            used[current] = true;

            points.push(segments[current].start);

            let next_key = point_key(segments[current].end);
            if next_key == first_key {
                break; // the loop has been closed
            }

            let Some(candidates) = by_start.get(&next_key) else {
                break;
            };
            let Some(&next_idx) = candidates.iter().find(|&&i| !used[i]) else {
                break;
            };

            current = next_idx;
        }
        if points.len() >= 3 {
            polygons.push(Polygon { points });
        }
    }

    return polygons;
}

pub fn triangulate(polygons: &Vec<Polygon>) -> Vec<Triangulation<[f64; 2], u32>> {
    let mut outers = Vec::new();
    let mut holes = Vec::new();

    for polygon in polygons {
        if signed_area(&polygon.points) < 0.0 {
            outers.push(polygon);
        } else {
            holes.push(polygon);
        }
    }

    let mut triangulations = Vec::new();
    for outer in outers {
        let outer_contour: Vec<[f64; 2]> = outer
            .points
            .iter()
            .map(|p| [p.0 as f64, p.1 as f64])
            .collect();

        let mut shape = vec![outer_contour];

        for hole in &holes {
            // Test any point on the hole against the outer boundary.
            if point_in_polygon(hole.points[0], &outer.points) {
                shape.push(
                    hole.points
                        .iter()
                        .map(|p| [p.0 as f64, p.1 as f64])
                        .collect(),
                );
            }
        }

        let triangulation = shape.triangulate().to_triangulation::<u32>();
        triangulations.push(triangulation);
    }

    triangulations
}
