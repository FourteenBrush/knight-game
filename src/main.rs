use game::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use macroquad::window::Conf;

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    game.load_asset("assets/_Idle.png").await;
    game.load_asset("assets/_Run.png").await;
    game.load_asset("assets/_Run_Left.png").await;
    game.load_asset("assets/_Roll.png").await;
    game.load_asset("assets/_Jump.png").await;

    game.load_asset("assets/back-trees.png").await;
    game.load_asset("assets/middle-trees.png").await;
    game.load_asset("assets/front-trees.png").await;
    game.load_asset("assets/lights.png").await;

    game.run().await;
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Demo".to_owned(),
        window_width: SCREEN_WIDTH as _,
        window_height: SCREEN_HEIGHT as _,
        ..Default::default()
    }
}
