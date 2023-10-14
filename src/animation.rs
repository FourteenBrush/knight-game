use std::rc::Rc;

use macroquad::prelude::*;

pub struct Animation {
    texture_atlas: Rc<Texture2D>,
    frame_dimension: Vec2,
    frame_count: u32,
    frame_idx: u32,
}

impl Animation {
    pub fn new(texture_atlas: Rc<Texture2D>,  frame_count: u32) -> Self {
        let frame_dimension = vec2(
            texture_atlas.width() / frame_count as f32,
            texture_atlas.height()
        );
        Self {
            texture_atlas,
            frame_dimension,
            frame_count,
            frame_idx: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame_idx = (self.frame_idx + 1) % self.frame_count;
    }

    pub fn is_fully_played(&self) -> bool {
        self.frame_idx == self.frame_count - 1
    }

    pub fn render(&self, dest: Rect) {
        let source = Rect::new(
            self.frame_dimension.x * self.frame_idx as f32, 0.,
            self.frame_dimension.x, self.frame_dimension.y
        );

        draw_texture_ex(&self.texture_atlas, dest.x, dest.y, WHITE, DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(dest.w, dest.h)),
            ..Default::default()
        });
    }
}