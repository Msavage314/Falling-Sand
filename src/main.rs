use bitflags::bitflags;
use egui_macroquad::egui;
use macroquad::color::Color;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq )]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked
        const RISES = 1<<4;

        const MELTABLE = 1<<5; // Destoryed by lava

        const SAND = Self::FALLS.bits() | Self::GRANULAR.bits();
        const LIQUID = Self::FALLS.bits() | Self::FLOWS.bits() | Self::GRANULAR.bits();
        const GAS = Self::RISES.bits() |Self::FLOWS.bits();
    }
}

/// Stores a material ID that represents a specific material
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Eq, Hash)]
pub enum MaterialID {
    Empty,
    Sand,
    Stone,
    Water,
    Slime,
    Salt,
    SaltWater,
    Lava,
    Steam,
    Dirt,
    Snow,
}

impl MaterialID {
    /// Returns a `MaterialProperties` struct of the properties associated with the materialID
    ///
    /// # Examples
    /// ```
    /// let x = MaterialID::Empty
    /// println!("{}",x.properties().desnsity)
    /// ```
    pub fn properties(self) -> &'static MaterialProperties {
        return &MATERIAL_TABLE[self as usize];
    }
}
static MATERIAL_TABLE: LazyLock<[MaterialProperties; 11]> = LazyLock::new(|| {
    [
        /* Empty */
        MaterialProperties {
            behavior: Behavior::empty(),
            density: 0.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            flow_distance: 0,
            lava_resistance: 0.0,
        },
        /* Sand  */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.5,
            color: Color::new(0.96, 0.82, 0.45, 1.0),
            flow_distance: 0,
            lava_resistance: 0.8,
        },
        /* Stone */
        MaterialProperties {
            behavior: Behavior::STATIC | Behavior::MELTABLE,
            density: 3.0,
            color: Color::new(0.5, 0.5, 0.5, 1.0),
            flow_distance: 0,
            lava_resistance: 0.05,
        },
        /* Water */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.0,
            color: Color::new(0.1, 0.45, 0.82, 1.0),
            flow_distance: 5,
            lava_resistance: 1.0,
        },
        /* Slime */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.3,
            color: Color::new(0.8, 0.3, 0.8, 1.0),
            flow_distance: 3,
            lava_resistance: 1.0,
        },
        /* Salt */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.5,
            color: Color::new(0.9, 0.9, 1.0, 1.0),
            flow_distance: 3,
            lava_resistance: 0.1,
        },
        /* Salt Water */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.1,
            color: Color::new(0.43, 0.77, 0.8, 1.0),
            flow_distance: 3,
            lava_resistance: 1.0,
        },
        /* Lava */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.1,
            color: Color::new(0.95, 0.7, 0.0, 1.0),
            flow_distance: 1,
            lava_resistance: 1.0,
        },
        /* Steam */
        MaterialProperties {
            behavior: Behavior::GAS,
            density: 0.1,
            color: Color::new(0.9, 0.9, 0.9, 1.0),
            flow_distance: 10,
            lava_resistance: 1.0,
        },
        /* Dirt */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.7,
            color: Color::new(0.35, 0.23, 0.16, 1.0),
            flow_distance: 10,
            lava_resistance: 0.4,
        },
        /* Snow */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.7,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            flow_distance: 10,
            lava_resistance: 1.0,
        },
    ]
});

#[derive(Debug)]
pub struct MaterialProperties {
    pub behavior: Behavior,
    pub density: f32,
    pub color: Color,

    pub flow_distance: u8,
    pub lava_resistance: f32,
}

/// A reactant can be either a material E.g. Water and Salt or a behavior e.g. Acid and anything with behavior Corrodable
#[derive(Debug, Clone, Copy)]
pub enum Reactant {
    Material(MaterialID),
    Behavior(Behavior),
}
impl Reactant {
    fn matches(self, mat: MaterialID) -> bool {
        match self {
            Reactant::Material(id) => id == mat,
            Reactant::Behavior(flag) => mat.properties().behavior.contains(flag),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ReactionOutcome {
    pub chance_fn: fn(self_mat: MaterialID, other_mat: MaterialID) -> f32,
    pub result: MaterialID,
}

pub struct Reaction {
    pub a: Reactant,
    pub b: Reactant,

    pub output_a: Option<ReactionOutcome>,
    pub output_b: Option<ReactionOutcome>,

    pub chance: f32,
}

static REACTIONS: &[Reaction] = &[
    Reaction {
        a: Reactant::Material(MaterialID::Salt),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: (MaterialID::SaltWater),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.5,
            result: (MaterialID::Water),
        }),
        chance: 0.1,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Lava),
        b: Reactant::Material(MaterialID::Water),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 0.5,
            result: (MaterialID::Stone),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: (MaterialID::Steam),
        }),

        chance: 0.5,
    },
    Reaction {
        a: Reactant::Material(MaterialID::Steam),
        b: Reactant::Material(MaterialID::Steam),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| 0.1,
            result: (MaterialID::Water),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 1.0,
            result: (MaterialID::Empty),
        }),

        chance: 0.01,
    },
    Reaction {
        a: Reactant::Behavior(Behavior::MELTABLE),
        b: Reactant::Material(MaterialID::Lava),

        output_a: Some(ReactionOutcome {
            chance_fn: |a, b| a.properties().lava_resistance,
            result: (MaterialID::Lava),
        }),
        output_b: Some(ReactionOutcome {
            chance_fn: |a, b| 0.2,
            result: (MaterialID::Empty),
        }),
        chance: 0.05,
    },
];

pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<MaterialID>,
    image: Image,
    texture: Texture2D,
    updated: Vec<bool>,
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
            updated: vec![false; width * height],
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
        if cell_a.properties().behavior.contains(Behavior::STATIC)
            || cell_b.properties().behavior.contains(Behavior::STATIC)
        {
            return false;
        }

        let diff = cell_a.properties().density - cell_b.properties().density;

        if diff <= 0.0 {
            return false;
        }

        let chance = (diff / cell_a.properties().density.clamp(0.0, 1.0));

        return macroquad::rand::gen_range(0.0, 1.0) < chance;
    }

    fn try_flow(&mut self, cell: MaterialID, x: i32, y: i32, dir: i32, max_dist: i32) -> bool {
        let mut target = None;

        for d in 1..=max_dist {
            let nx = x + dir * d;
            let other = self.get(nx, y);

            if self.can_density_swap(cell, other) {
                target = Some(nx);
            } else {
                break;
            }
        }
        if let Some(nx) = target {
            self.swap_cells(x, y, nx, y);
            return true;
        } else {
            return false;
        }
    }
    fn mark_updated(&mut self, x: i32, y: i32) {
        let idx = y as usize * self.width + x as usize;
        self.updated[idx] = true
    }
    fn swap_cells(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let a = self.get(x1, y1);
        let b = self.get(x2, y2);

        self.set(x1, y1, b);
        self.set(x2, y2, a);

        self.mark_updated(x1, y1);
        self.mark_updated(x2, y2);
    }
    fn get_neighbors(&mut self, x: i32, y: i32) -> [(i32, i32); 4] {
        let neighbors = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        return neighbors.map(|c| (c.0 + x, c.1 + y));
    }
    fn update_cell(&mut self, x: i32, y: i32) {
        let idx = y as usize * self.width + x as usize;
        if self.updated[idx] {
            return;
        }
        let cell = self.get(x, y);
        let properties = cell.properties();

        for (cx, cy) in self.get_neighbors(x, y) {
            let other = self.get(cx, cy);
            for reaction in REACTIONS {
                if reaction.a.matches(cell)
                    && reaction.b.matches(other)
                    && macroquad::rand::gen_range(0.0, 1.0) < reaction.chance
                {
                    let mut changed = false;
                    if let Some(oa) = reaction.output_a {
                        if macroquad::rand::gen_range(0.0, 1.0) < (oa.chance_fn)(cell, other) {
                            self.set(x, y, oa.result);
                            changed = true;
                        }
                    }
                    if let Some(ob) = reaction.output_b {
                        if macroquad::rand::gen_range(0.0, 1.0) < (ob.chance_fn)(cell, other) {
                            self.set(cx, cy, ob.result);
                            changed = true;
                        }
                    }

                    if changed {
                        self.mark_updated(x, y);
                        self.mark_updated(cx, cy);
                        return;
                    }
                }
            }
        }

        if properties.behavior.contains(Behavior::FALLS) {
            if self.can_density_swap(self.get(x, y), self.get(x, y + 1)) {
                self.swap_cells(x, y, x, y + 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::RISES) {
            if self.can_density_swap(self.get(x, y), self.get(x, y - 1)) {
                self.swap_cells(x, y, x, y - 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::GRANULAR) {
            if macroquad::rand::gen_range(0, 2) == 1 {
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
            } else {
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
            }
        }
        if properties.behavior.contains(Behavior::FLOWS) {
            let flow = macroquad::rand::gen_range(0, properties.flow_distance * 2) as i32;
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
        self.updated.fill(false);

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
    pub fn total_alive(&mut self) -> i32 {
        let mut count = 0;
        for cell in &self.cells {
            if *cell != MaterialID::Empty {
                count += 1;
            }
        }
        return count;
    }
    pub fn count_by_material(&mut self) -> HashMap<MaterialID, i32> {
        let mut counts = HashMap::new();
        for cell in &self.cells {
            if *cell != MaterialID::Empty {
                *counts.entry(*cell).or_insert(0) += 1
            }
        }
        return counts;
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
        } else if is_key_pressed(KeyCode::Key4) {
            active = MaterialID::Slime
        } else if is_key_pressed(KeyCode::Key5) {
            active = MaterialID::Salt
        } else if is_key_pressed(KeyCode::Key6) {
            active = MaterialID::Lava
        } else if is_key_pressed(KeyCode::Key7) {
            active = MaterialID::Steam
        } else if is_key_pressed(KeyCode::Key8) {
            active = MaterialID::Dirt
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

        // if frame_count % 100 == 0 {
        //     println!("{}", g.total_alive())
        // }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", get_fps()));
                ui.separator();
                ui.label(format!("Total cells alive = {}", g.total_alive()));

                let counts = g.count_by_material();
                for id in MaterialID::iter() {
                    if id != MaterialID::Empty {
                        let count = counts.get(&id).copied().unwrap_or(0);
                        ui.label(format!("{:?}: {}", id, count));
                    }
                }
            });
        });
        egui_macroquad::draw();

        next_frame().await;
    }
}
