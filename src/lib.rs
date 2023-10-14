#![feature(variant_count)]

mod utils;

use std::{mem, rc::Rc};

use fixedbitset::FixedBitSet;
use macroquad::{
    miniquad::window::screen_size,
    prelude::{animation::AnimatedSprite, *},
};
use strum::{EnumIter, IntoEnumIterator};

use animation::Animation;
use utils::FixedBitSetIterator;

pub const SCREEN_WIDTH: u32 = 272 * 5;
pub const SCREEN_HEIGHT: u32 = 160 * 5;

const DEST_WIDTH: f32 = 120. * 6.;
const DEST_WIDTH_HALF: f32 = DEST_WIDTH / 2.;
const DEST_HEIGHT: f32 = 80. * 6.;
const DEST_HEIGHT_HALF: f32 = DEST_HEIGHT / 2.;

const HOR_VELOCITY: f32 = 400.; // was 250
const JUMP_VELOCITY: f32 = 300.;
const GRAVITY: f32 = 480.;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Pose {
    Idle,
    RunningForwards,
    RunningBackwards,
    Rolling,
    Jumping,
}

struct Player {
    sprite: AnimatedSprite,
    pose: Pose,
    position: Vec2,
    velocity: Vec2,
    jumping: bool,
}

fn create_animation(name: &str, frames: u32) -> Animation {
    const ANIMATION_FPS: u32 = 11;

    Animation {
        name: name.to_owned(),
        row: 0,
        frames,
        fps: ANIMATION_FPS,
    }
}

impl Player {
    fn new() -> Self {
        let animations = &[
            create_animation("idle", 10),
            create_animation("running-forwards", 10),
            create_animation("running-backwards", 10),
            create_animation("rolling", 12),
            create_animation("juping", 3),
        ];

        const SPRITE_TILE_WIDTH: u32 = 120;
        const SPRITE_TILE_HEIGHT: u32 = 80;

        let sprite = AnimatedSprite::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT, animations, true);

        Self {
            sprite,
            pose: Pose::Idle,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            jumping: false,
        }
    }
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum BackgroundFlag {
    BackTrees,
    MiddleTrees,
    FrontTrees,
    Lights,
}

#[derive(Debug, Copy, Clone)]
pub enum Asset {
    AnimationIdle,
    AnimationRunningForwards,
    AnimationRunningBackwards,
    AnimationRolling,
    AnimationJumping,

    BackTrees,
    MiddleTrees,
    FrontTrees,
    Lights,
}

impl From<BackgroundFlag> for Asset {
    fn from(value: BackgroundFlag) -> Self {
        match value {
            BackgroundFlag::BackTrees => Asset::BackTrees,
            BackgroundFlag::MiddleTrees => Asset::MiddleTrees,
            BackgroundFlag::FrontTrees => Asset::FrontTrees,
            BackgroundFlag::Lights => Asset::Lights,
        }
    }
}

impl From<Pose> for Asset {
    fn from(value: Pose) -> Self {
        match value {
            Pose::Idle => Asset::AnimationIdle,
            Pose::RunningForwards => Asset::AnimationRunningForwards,
            Pose::RunningBackwards => Asset::AnimationRunningBackwards,
            Pose::Rolling => Asset::AnimationRolling,
            Pose::Jumping => Asset::AnimationJumping,
        }
    }
}

pub struct Game {
    player: Player,
    assets: Vec<Rc<Texture2D>>,
    background_flags: FixedBitSet,
    background_render_target: RenderTarget,
    old_screen_size: (f32, f32), // to detect resizes
}

impl Game {
    pub fn new() -> Self {
        let mut background_flags =
            FixedBitSet::with_capacity(mem::variant_count::<BackgroundFlag>());
        for flag in BackgroundFlag::iter() {
            background_flags.insert(flag as usize);
        }

        let background_render_target = render_target(SCREEN_WIDTH, SCREEN_HEIGHT);
        background_render_target
            .texture
            .set_filter(FilterMode::Nearest);

        Self {
            player: Player::new(),
            assets: Vec::with_capacity(mem::variant_count::<Asset>()),
            background_flags,
            background_render_target,
            old_screen_size: screen_size(),
        }
    }

    pub async fn load_asset(&mut self, path: &str) {
        let texture = load_texture(path).await.unwrap();
        texture.set_filter(FilterMode::Nearest);
        self.assets.push(Rc::new(texture));
    }

    fn get_asset(&self, asset: Asset) -> Rc<Texture2D> {
        self.assets[asset as usize].clone()
    }

    pub async fn run(&mut self) {
        //const MINIMUM_FRAME_TIME: f32 = 1. / 60.; // 60 fps

        loop {
            let frame_time = get_frame_time();

            self.update(frame_time);
            self.render();

            /* if frame_time < MINIMUM_FRAME_TIME {
                let time_to_sleep = (MINIMUM_FRAME_TIME - frame_time) * 1000.;
                // no tokio support for macroquad
                thread::sleep(Duration::from_millis(time_to_sleep as u64));
            } */

            next_frame().await;
        }
    }

    fn update(&mut self, frame_time: f32) {
        self.player.sprite.set_animation(self.player.pose as usize);
        self.player.sprite.update();

        let can_change_pose = self.player.pose == Pose::Idle || self.player.sprite.is_last_frame();
        if can_change_pose {
            use KeyCode::*;

            if is_key_down(Right) || is_key_down(D) {
                self.player.pose = Pose::RunningForwards;
                self.player.velocity.x = HOR_VELOCITY;
            } else if is_key_down(Left) || is_key_down(A) {
                self.player.pose = Pose::RunningBackwards;
                self.player.velocity.x = -HOR_VELOCITY;
            } else if is_key_down(Down) || is_key_down(S) {
                self.player.pose = Pose::Rolling;
            } else if is_key_down(Space) {
                if !self.player.jumping {
                    self.player.pose = Pose::Jumping;
                    self.player.velocity = vec2(0., JUMP_VELOCITY);
                    self.player.jumping = true;
                }
            } else if self.player.pose != Pose::Jumping {
                self.player.pose = Pose::Idle;
                self.player.velocity.x = 0.;
            }
        }

        static KEY_MAP: [(KeyCode, BackgroundFlag); 4] = [
            (KeyCode::Kp0, BackgroundFlag::Lights),
            (KeyCode::Kp1, BackgroundFlag::BackTrees),
            (KeyCode::Kp2, BackgroundFlag::MiddleTrees),
            (KeyCode::Kp3, BackgroundFlag::FrontTrees),
        ];

        for (key_code, flag) in KEY_MAP {
            if is_key_pressed(key_code) {
                self.background_flags.toggle(flag as usize);
            }
        }

        self.player.position += self.player.velocity * frame_time;
        self.player.velocity.y -= GRAVITY * frame_time;

        if self.player.position.y <= 0. {
            // reached the ground
            self.player.position.y = 0.;
            self.player.velocity.y = 0.;
            self.player.jumping = false;
            // FIXME: why bother checking?
            if self.player.pose == Pose::Jumping {
                self.player.pose = Pose::Idle;
            }
        }

        let screen_size = screen_size();
        if self.old_screen_size != screen_size {
            self.background_render_target.delete();
            self.background_render_target =
                render_target(screen_size.0 as u32, screen_size.1 as u32);
        }
    }

    fn render(&mut self) {
        clear_background(BLACK);
        self.render_background();

        let texture = &self.get_asset(Asset::from(self.player.pose));
        let (dest_x, dest_y) = (
            screen_width() / 2. - DEST_WIDTH_HALF,
            screen_height() / 2. - DEST_HEIGHT_HALF - self.player.position.y,
        );

        draw_texture_ex(
            texture,
            dest_x,
            dest_y,
            WHITE,
            DrawTextureParams {
                source: Some(self.player.sprite.frame().source_rect),
                dest_size: Some(vec2(DEST_WIDTH, DEST_HEIGHT)),
                ..Default::default()
            },
        );

        self.render_fps();
    }

    fn render_background(&mut self) {
        if self.background_flags.is_empty() {
            return;
        }

        let (screen_width, screen_height) = screen_size();

        // draw to offscreen framebuffer
        set_camera(&Camera2D {
            render_target: Some(self.background_render_target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., screen_width, screen_height))
        });

        clear_background(BLACK);

        for flag in FixedBitSetIterator::new(&self.background_flags) {
            let texture_layer = &self.assets[Asset::from(flag) as usize];

            draw_texture_ex(
                texture_layer,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(screen_size().into()),
                    ..Default::default()
                },
            );
        }

        // draw to screen
        set_default_camera();

        let params = DrawTextureParams {
            flip_y: true,
            ..Default::default()
        };

        // draw background twice, to make it 'scrollable'
        let pos_x = self.player.position.x;
        let background_offset_x1 = -pos_x % screen_width;
        let background_offset_x2 = -pos_x % screen_width + screen_width * pos_x.signum();

        draw_texture_ex(
            &self.background_render_target.texture,
            background_offset_x1,
            0.,
            WHITE,
            params.clone(),
        );
        draw_texture_ex(
            &self.background_render_target.texture,
            background_offset_x2,
            0.,
            WHITE,
            params,
        );
    }

    fn render_fps(&self) {
        let fps = get_fps();

        let color = match fps {
            (29..=15) => ORANGE,
            (0..15) => RED,
            _ => LIME,
        };

        draw_text_ex(
            &format!("{fps:2}"),
            10.,
            20.,
            TextParams {
                font_size: 30,
                color,
                ..Default::default()
            },
        );
    }
}
