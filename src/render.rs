use macroquad::prelude::*;

use crate::{simulation, stains::StainKind, ui::compute_grid_dest_rect};

pub struct GridRenderer {
    image: Image,
    texture: Texture2D,
}
impl GridRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        let image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        return GridRenderer { image, texture };
    }
    pub fn update_texture(&mut self, grid: &simulation::Grid) {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let cell = grid.get(x as i32, y as i32);
                let base_color = cell.color;
                let final_color = match cell.stain {
                    Some(stain) if stain.kind == StainKind::Burning => Color::new(
                        base_color.r * 0.3 + 0.95 * 0.7,
                        base_color.g * 0.3 + 0.4 * 0.7,
                        base_color.b * 0.3,
                        1.0,
                    ),
                    Some(stain) if stain.kind == StainKind::Wet => {
                        let darken = 1.0 - 0.3 * stain.intensity;
                        Color::new(
                            base_color.r * darken,
                            base_color.g * darken,
                            base_color.b * darken,
                            1.0,
                        )
                    }
                    Some(stain) if stain.kind == StainKind::Slimy => {
                        let new = blend_colors(base_color, Color::new(0.8, 0.3, 0.8, 1.0));
                        Color::new(new.r * 0.9, new.g * 0.9, new.b * 0.9, new.a)
                    }
                    Some(stain) if stain.kind == StainKind::Toxic => {
                        Color::from_rgba(148, 247, 0, 255)
                    }
                    _ => base_color,
                };
                self.image.set_pixel(x as u32, y as u32, final_color);
            }
        }
        for particle in &grid.particles {
            let x = particle.pos.x.floor();
            let y = particle.pos.y.floor();
            if !(x < 0.0 || y < 0.0 || x as usize >= grid.width || y as usize >= grid.height) {
                self.image.set_pixel(
                    x as u32,
                    y as u32,
                    (particle.material.properties().color)(x as i32, y as i32),
                );
            }
        }
        self.texture.update(&self.image);
    }
    pub fn draw(&mut self, grid: &simulation::Grid) {
        self.update_texture(grid);
        let (x, y, w, h) = compute_grid_dest_rect(grid.width, grid.height);
        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        )
    }
}
fn blend_colors(back: Color, front: Color) -> Color {
    let out_a = front.a + back.a * (1.0 - front.a);
    if out_a <= 0.0 {
        return Color::new(0.0, 0.0, 0.0, 0.0);
    }

    let out_r = (front.r * front.a + back.r * back.a * (1.0 - front.a)) / out_a;
    let out_g = (front.g * front.a + back.g * back.a * (1.0 - front.a)) / out_a;
    let out_b = (front.b * front.a + back.b * back.a * (1.0 - front.a)) / out_a;

    Color::new(out_r, out_g, out_b, out_a)
}
