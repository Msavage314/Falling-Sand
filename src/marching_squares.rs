pub struct Segment {
    pub start: (f32, f32),
    pub end: (f32, f32),
}

use crate::{materials::MaterialID, simulation::Grid};
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

    let mut segments: Vec<Segment> = Vec::new();
    // Marching squares looks at each square of cells,
    // so we iterate one less in each direction.
    for x in 0..grid.width - 1 {
        for y in 0..grid.height - 1 {
            let p1 = cells[y * grid.width + x];
            let p2 = cells[y * grid.width + (x + 1)];
            let p3 = cells[(y + 1) * grid.width + (x + 1)];
            let p4 = cells[(y + 1) * grid.width + x];

            let case: u8 = (p1 | (p2 << 1) | (p3 << 2) | (p4 << 3)) as u8;

            let top = (x as f32 + 0.5, y as f32);
            let right = (x as f32 + 1.0, y as f32 + 0.5);
            let bottom = (x as f32 + 0.5, y as f32 + 1.0);
            let left = (x as f32, y as f32 + 0.5);

            match case {
                0 | 15 => {}

                1 | 14 => {
                    segments.push(Segment {
                        start: left,
                        end: top,
                    });
                }

                2 | 13 => {
                    segments.push(Segment {
                        start: top,
                        end: right,
                    });
                }

                3 | 12 => {
                    segments.push(Segment {
                        start: left,
                        end: right,
                    });
                }

                4 | 11 => {
                    segments.push(Segment {
                        start: right,
                        end: bottom,
                    });
                }

                5 => {
                    // Ambiguous case
                    segments.push(Segment {
                        start: left,
                        end: top,
                    });

                    segments.push(Segment {
                        start: right,
                        end: bottom,
                    });
                }

                6 | 9 => {
                    segments.push(Segment {
                        start: top,
                        end: bottom,
                    });
                }

                7 | 8 => {
                    segments.push(Segment {
                        start: left,
                        end: bottom,
                    });
                }

                10 => {
                    // Ambiguous case
                    segments.push(Segment {
                        start: top,
                        end: right,
                    });

                    segments.push(Segment {
                        start: bottom,
                        end: left,
                    });
                }

                _ => unreachable!(),
            }
        }
    }
    return segments;
}
