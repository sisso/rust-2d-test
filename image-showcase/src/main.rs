use cgmath;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::{graphics, Context, ContextBuilder, GameResult};

fn main() -> GameResult<()> {
    let resource_dir = std::path::PathBuf::from("/home/sisso/Desktop");
    //     if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
    //     let mut path = std::path::PathBuf::from(manifest_dir);
    //     path.push("resources");
    //     path
    // } else {
    //     std::path::PathBuf::from("./resources")
    // };

    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = 1900.0;
    window_mode.height = 1024.0;

    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .add_resource_path(resource_dir)
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(&mut ctx)?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => {
            println!("Exited cleanly.");
            Ok(())
        }
        Err(e) => {
            println!("Error occured: {}", e);
            Err(e)
        }
    }
}

struct MyGame {
    image: graphics::Image,
    point: cgmath::Point2<f32>,
    scale: cgmath::Vector2<f32>,
    rotation: f32,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> GameResult<MyGame> {
        let image = graphics::Image::new(ctx, "/20200410_103353.jpg")?;
        let point = cgmath::Point2::new(0.0, 0.0);
        let scale = cgmath::Vector2::new(0.25, 0.25);
        let rotation = 0.0;
        let game = MyGame {
            image,
            point,
            scale,
            rotation,
        };
        Ok(game)
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        self.point.x += 1.0;
        self.scale.x += 0.001;
        self.scale.y += 0.001;
        self.rotation += 0.001;
        // graphics::draw(ctx, &self.image, (self.point.clone(),))?;
        graphics::draw(
            ctx,
            &self.image,
            graphics::DrawParam::new()
                .dest(self.point.clone())
                .rotation(self.rotation)
                .scale(self.scale),
        )?;
        graphics::present(ctx)
    }
}
