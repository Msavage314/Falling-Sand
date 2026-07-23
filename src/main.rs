use bitflags::bitflags;
use macroquad::color::Color;
use macroquad::prelude::*;

bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq)]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked

        const SAND = Self::FALLS.bits() | Self::GRANULAR.bits();
        const LIQUID = Self::FALLS.bits() | Self::FLOWS.bits() | Self::GRANULAR.bits();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialID {
    Empty,
    Sand,
    Stone,
    Water,
}

impl MaterialID {
    pub fn properties(self) -> &'static MaterialProperties {
        return &MATERIAL_TABLE[self as usize];
    }
}
static MATERIAL_TABLE: [MaterialProperties; 4] = [
    /* Empty */
    MaterialProperties {
        behavior: Behavior::empty(),
        density: 0.0,
        color: Color::new(0.0, 0.0, 0.0, 1.0),
        flow_distance: 0,
    },
    /* Sand  */
    MaterialProperties {
        behavior: Behavior::SAND,
        density: 1.5,
        color: Color::new(0.96, 0.82, 0.45, 1.0),
        flow_distance: 0,
    },
    /* Stone */
    MaterialProperties {
        behavior: Behavior::STATIC,
        density: 3.0,
        color: Color::new(0.5, 0.5, 0.5, 1.0),
        flow_distance: 0,
    },
    /* Water */
    MaterialProperties {
        behavior: Behavior::LIQUID,
        density: 1.0,
        color: Color::new(0.1, 0.45, 0.82, 1.0),
        flow_distance: 5,
    },
];

#[derive(Debug)]
pub struct MaterialProperties {
    pub behavior: Behavior,
    pub density: f32,
    pub color: Color,

    pub flow_distance: u8,
}

pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<MaterialID>,
    image: Image,
    texture: Texture2D,
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        return Grid {
            width: width,
            height: height,
            cells: vec![MaterialID::Empty; width * height],
            image,
            texture,
        };
    }
    pub fn get(&self, x: i32, y: i32) -> MaterialID {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return MaterialID::Stone;
        }

        return self.cells[y as usize * self.width + x as usize];
    }
    pub fn set(&mut self, x: i32, y: i32, value: MaterialID) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.cells[y as usize * self.width + x as usize] = value
    }
    fn can_density_swap(&mut self, cell_a: MaterialID, cell_b: MaterialID) -> bool {
        if cell_a.properties().density > cell_b.properties().density {
            return !cell_a.properties().behavior.contains(Behavior::STATIC)
                && !cell_b.properties().behavior.contains(Behavior::STATIC);
        } else {
            return false;
        }
    }
    fn try_flow(&mut self, cell: MaterialID, x: i32, y: i32, dir: i32, max_dist: i32) -> bool {
        let mut target = None;

        for d in 1..=max_dist {
            let nx = x + dir * d;
            let other = self.get(nx, y);

            if other == MaterialID::Empty {
                target = Some(nx);
            } else {
                break;
            }
        }

        if let Some(nx) = target {
            self.set(nx, y, cell);
            self.set(x, y, MaterialID::Empty);
            return true;
        } else {
            return false;
        }
    }
    fn update_cell(&mut self, x: i32, y: i32) {
        let cell = self.get(x, y);
        let properties = cell.properties();

        if properties.behavior.contains(Behavior::FALLS) {
            if self.can_density_swap(self.get(x, y), self.get(x, y + 1)) {
                let other = self.get(x, y + 1);
                self.set(x, y + 1, cell);
                self.set(x, y, other);
                return;
            }
        }
        if properties.behavior.contains(Behavior::GRANULAR) {
            if macroquad::rand::gen_range(0, 2) == 1 {
                let other = self.get(x + 1, y + 1);

                if self.can_density_swap(cell, other) {
                    self.set(x + 1, y + 1, cell);
                    self.set(x, y, other);
                    return;
                }
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.set(x - 1, y + 1, cell);
                    self.set(x, y, other);
                    return;
                }
            } else {
                if self.get(x - 1, y + 1) == MaterialID::Empty {
                    self.set(x - 1, y + 1, cell);
                    self.set(x, y, MaterialID::Empty);
                    return;
                } else if self.get(x + 1, y + 1) == MaterialID::Empty {
                    self.set(x + 1, y + 1, cell);
                    self.set(x, y, MaterialID::Empty);
                    return;
                }
            }
        }
        if properties.behavior.contains(Behavior::FLOWS) {
            let flow = properties.flow_distance as i32;
            let left_first = macroquad::rand::gen_range(0, 2) == 0;

            if left_first {
                if self.try_flow(cell, x, y, -1, flow) {
                    return;
                }
                if self.try_flow(cell, x, y, 1, flow) {
                    return;
                }
            } else {
                if self.try_flow(cell, x, y, 1, flow) {
                    return;
                }
                if self.try_flow(cell, x, y, -1, flow) {
                    return;
                }
            }
        }

        // if !properties.behavior.contains(Behavior::STATIC) {
        //     let cell_b = self.get(x, y + 1);
        //     if self.get(x, y).properties().density > self.get(x, y + 1).properties().density {
        //         self.set(x, y, cell_b);
        //         self.set(x, y + 1, cell);
        //         return;
        //     }
        // }
    }
    pub fn update(&mut self, left: bool) {
        for y in (0..self.height).rev() {
            if left {
                for x in (0..self.width).rev() {
                    self.update_cell(x as i32, y as i32);
                }
            } else {
                for x in (0..self.width) {
                    self.update_cell(x as i32, y as i32);
                }
            }
        }
    }
    pub fn update_texture(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let id = self.cells[y * self.width + x];
                self.image
                    .set_pixel(x as u32, y as u32, id.properties().color);
            }
        }

        self.texture.update(&self.image);
    }

    pub fn draw(&mut self) {
        self.update_texture();
        draw_texture_ex(
            &self.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        )
    }
    pub fn draw_brush(&mut self, cx: i32, cy: i32, radius: i32, material: MaterialID) {
        let r2 = radius * radius;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.set(cx + x, cy + y, material)
                }
            }
        }
    }
}
fn screen_to_grid(grid: &Grid, screen_x: f32, screen_y: f32) -> (i32, i32) {
    let grid_x = (screen_x / screen_width() * grid.width as f32) as i32;
    let grid_y = (screen_y / screen_height() * grid.height as f32) as i32;
    (grid_x, grid_y)
}

#[macroquad::main("Falling Sand")]
async fn main() {
    let mut g = Grid::new(200, 150);
    let mut frame_count = 0;
    g.set(50, 0, MaterialID::Sand);
    let mut active = MaterialID::Sand;
    let mut radius = 1;
    loop {
        clear_background(BLACK);

        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (gx, gy) = screen_to_grid(&g, mx, my);
            g.draw_brush(gx, gy, radius, active);
        }
        if is_key_pressed(KeyCode::Key0) {
            active = MaterialID::Empty
        } else if is_key_pressed(KeyCode::Key1) {
            active = MaterialID::Sand
        } else if is_key_pressed(KeyCode::Key2) {
            active = MaterialID::Stone
        } else if is_key_pressed(KeyCode::Key3) {
            active = MaterialID::Water
        }

        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            radius += 1;
        }

        // Decrease
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            radius = (radius - 1).max(1);
        }

        frame_count += 1;

        g.update(frame_count % 2 == 0);
        g.draw();

        next_frame().await;
    }
}
