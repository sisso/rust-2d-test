use cgmath;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::Image;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use rand::Rng;
use std::env;
use std::path::{Path, PathBuf};

fn main() -> GameResult<()> {
    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = 1900.0;
    window_mode.height = 1024.0;

    let (mut ctx, mut event_loop) = ContextBuilder::new("Image showcase", "Someone")
        // .add_resource_path(resource_dir)
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    // let root_path = if env::vars().find(|k,v| )args().len() < 1 {
    //     env::home_dir().expect("user folder not found")
    // }

    // TODO: require a proper argument parser
    let root_path = match env::args().nth(1) {
        Some(path) => path,
        None => format!(
            "{}/Pictures",
            env::var("HOME").expect("user home not found")
        ),
    };

    let mut app = App::new(&mut ctx, PathBuf::from(root_path).as_ref())?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
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

struct App {
    current_image: graphics::Image,
    next_image: graphics::Image,
    point: cgmath::Point2<f32>,
    scale: cgmath::Vector2<f32>,
    rotation: f32,
    images: Vec<PathBuf>,
    swtich_delay: bool,
}

impl App {
    pub fn new(ctx: &mut Context, root: &Path) -> GameResult<App> {
        let images = find_images(root)?;
        let current_image = load_random_image(ctx, &images)?;
        let next_image = load_random_image(ctx, &images)?;

        let point = cgmath::Point2::new(0.0, 0.0);
        let scale = cgmath::Vector2::new(0.25, 0.25);
        let rotation = 0.0;
        let game = App {
            current_image: current_image,
            next_image: next_image,
            point,
            scale,
            rotation,
            images,
            swtich_delay: false,
        };
        Ok(game)
    }

    fn load_next_image(&mut self, ctx: &mut Context) -> GameResult<()> {
        std::mem::swap(&mut self.current_image, &mut self.next_image);
        self.next_image = load_random_image(ctx, &self.images)?;
        self.point = cgmath::Point2::new(0.0, 0.0);
        self.scale = cgmath::Vector2::new(0.25, 0.25);
        self.rotation = 0.0;
        self.swtich_delay = true;
        Ok(())
    }
}

fn load_random_image(ctx: &mut Context, images: &Vec<PathBuf>) -> GameResult<Image> {
    let index = images[choose(images.len())].as_ref();
    load_image(ctx, index)
}

fn choose(len: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0, len)
}

fn load_image(ctx: &mut Context, path: &Path) -> GameResult<Image> {
    let buffer = std::fs::read(path)?;
    let img = image::load_from_memory(&buffer).unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    Image::from_rgba8(ctx, width as u16, height as u16, &img)
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.swtich_delay && ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
            self.load_next_image(ctx)?;
        } else {
            self.swtich_delay = false;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        self.point.x += 1.0;
        self.scale.x += 0.0001;
        self.scale.y += 0.0001;
        self.rotation += 0.0001;
        // graphics::draw(ctx, &self.image, (self.point.clone(),))?;
        graphics::draw(
            ctx,
            &self.current_image,
            graphics::DrawParam::new()
                .dest(self.point.clone())
                .rotation(self.rotation)
                .scale(self.scale),
        )?;
        graphics::present(ctx)
    }
}

fn find_images(path: &Path) -> GameResult<Vec<PathBuf>> {
    let mut files = list_files_at(path)?;
    files.retain(|f| {
        let file = f.to_string_lossy();
        file.ends_with(".png") || file.ends_with(".jpg")
    });
    Ok(files)
}

fn list_files_at(root_path: &Path) -> GameResult<Vec<PathBuf>> {
    let mut result = vec![];
    let list: std::fs::ReadDir = std::fs::read_dir(root_path)?;
    for entry in list {
        let path = entry?.path();
        if path.is_dir() {
            result.extend(list_files_at(&path)?);
        } else {
            result.push(path);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_images_recursive() {
        let iterator = find_images(&std::path::PathBuf::from("/home/sisso/Pictures")).unwrap();
        for file in iterator {
            println!("{:?}", file);
        }

        assert!(false);
    }
}
