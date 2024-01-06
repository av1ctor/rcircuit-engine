use three_rs::math::vector3::Vector3;
use crate::{object::Object, player::Player};

pub trait Renderer {
    fn clear(&mut self);
    fn present(&mut self);
    fn draw_line (&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: (u8, u8, u8));
    fn draw_rect (&mut self, x: f32, y: f32, w: f32, h: f32, color: (u8, u8, u8));
    fn fill_rect (&mut self, x: f32, y: f32, w: f32, h: f32, color: (u8, u8, u8));
    fn draw_pixel (&mut self, x: f32, y: f32, color: (u8, u8, u8));
    fn draw_player (&mut self, player: &Player, pov: &Vector3);
    fn draw_key (&mut self, obj: &Object, pov: &Vector3);
    fn draw_door (&mut self, obj: &Object, pov: &Vector3);
    fn draw_trap (&mut self, obj: &Object, pov: &Vector3);
    fn draw_enemy (&mut self, obj: &Object, pov: &Vector3);
}

