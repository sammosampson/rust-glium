#[macro_use]
extern crate glium;
extern crate image;
pub mod triangle;
pub mod animated_triangle;
pub mod rotating_triangle_with_matrix;
pub mod textured_wall;
pub mod sdf_circle;
pub mod full_sdf_rect_circle_text_render;

pub fn main() {
    full_sdf_rect_circle_text_render::run();
}