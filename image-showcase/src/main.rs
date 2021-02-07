use approx::assert_relative_eq;
use cgmath;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::Image;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use image::RgbaImage;
use rand::Rng;
use std::env;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SLEEP_SECONDS: u64 = 4;

fn main() -> GameResult<()> {
    let screen_width = 1900;
    let screen_height = 1024;

    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = screen_width as f32;
    window_mode.height = screen_height as f32;

    let (mut ctx, mut event_loop) = ContextBuilder::new("Image showcase", "Someone")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let root_path = match env::args().nth(1) {
        Some(path) => path,
        None => format!(
            "{}/Pictures",
            env::var("HOME").expect("user home not found")
        ),
    };

    let mut app = App::new(
        &mut ctx,
        PathBuf::from(root_path).as_ref(),
        screen_width,
        screen_height,
    )?;

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

type ImageRef = Arc<Mutex<Option<RgbaImage>>>;

struct App {
    current_image: graphics::Image,
    next_image: ImageRef,
    point: cgmath::Point2<f32>,
    scale: cgmath::Vector2<f32>,
    rotation: f32,
    images: Vec<PathBuf>,
    change_image: bool,
    ignore_keys_until: usize,
    screen_width: u32,
    screen_height: u32,
    next_switch: Instant,
}

impl App {
    pub fn new(
        ctx: &mut Context,
        root: &Path,
        screen_width: u32,
        screen_height: u32,
    ) -> GameResult<App> {
        let images = find_images(root)?;

        let current_image = load_random_image(ctx, &images)?;

        let next_image_path = &images[choose(images.len())];
        let next_image_ref = Arc::new(Mutex::new(None));
        load_image_async(next_image_path, next_image_ref.clone());

        let point = cgmath::Point2::new(0.0, 0.0);
        let scale = cgmath::Vector2::new(0.25, 0.25);
        let rotation = 0.0;

        let game = App {
            current_image: current_image,
            next_image: next_image_ref,
            point,
            scale,
            rotation,
            images,
            change_image: false,
            ignore_keys_until: 0,
            screen_width,
            screen_height,
            next_switch: compute_next_switch(),
        };
        Ok(game)
    }

    fn load_next_image(&mut self, ctx: &mut Context) -> GameResult<()> {
        // get next image if is already loaded
        let mut image_ref = self.next_image.deref().lock().unwrap();
        let image_rgb = match image_ref.take() {
            Some(image) => image,
            _ => return Ok(()),
        };

        // trigger to load next image async
        let next_image_path = &self.images[choose(self.images.len())];
        load_image_async(next_image_path, self.next_image.clone());

        // load rbga into image
        let (width, height) = image_rgb.dimensions();
        let image = Image::from_rgba8(ctx, width as u16, height as u16, &image_rgb)?;
        self.current_image = image;

        // reset positions
        self.point = cgmath::Point2::new(0.0, 0.0);
        // self.scale = cgmath::Vector2::new(0.25, 0.25);
        self.scale = cgmath::Vector2::new(1.0, 1.0)
            * fit_image(self.screen_width, self.screen_height, width, height);
        self.rotation = 0.0;

        // block any new input
        self.ignore_keys_until = ggez::timer::ticks(ctx) + 60;
        self.change_image = false;

        Ok(())
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let tick = ggez::timer::ticks(ctx);
        if tick > self.ignore_keys_until
            && ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Space)
        {
            println!("loading next image");
            self.change_image = true;
        }

        if self.next_switch <= Instant::now() {
            self.next_switch = compute_next_switch();
            println!("time swtich");
            self.change_image = true;
        }

        if self.change_image {
            self.load_next_image(ctx)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // graphics::clear(ctx, graphics::BLACK);
        // self.point.x += 1.0;
        // self.scale.x += 0.0001;
        // self.scale.y += 0.0001;
        // self.rotation += 0.0001;
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

fn load_random_image(ctx: &mut Context, images: &Vec<PathBuf>) -> GameResult<Image> {
    let index = images[choose(images.len())].as_ref();
    load_image(ctx, index)
}

fn choose(len: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0, len)
}

fn load_image_async(path: &Path, image_ref: ImageRef) {
    let path = path.to_path_buf();

    thread::spawn(move || {
        // TODO: test ImageReader::open("myimage.png")?.decode()
        let start_trace = Instant::now();
        println!("loading {}", path.to_string_lossy());
        let buffer = std::fs::read(&path).expect("fail to load image");
        let img = image::load_from_memory(&buffer).unwrap().to_rgba8();
        *image_ref.lock().unwrap() = Some(img);
        println!(
            "loaded {} in {}ms",
            path.to_string_lossy(),
            start_trace.elapsed().as_millis()
        );
    });
}

fn load_image(ctx: &mut Context, path: &Path) -> GameResult<Image> {
    let buffer = std::fs::read(path)?;
    let img = image::load_from_memory(&buffer).unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    Image::from_rgba8(ctx, width as u16, height as u16, &img)
}

fn fit_image(screen_width: u32, screen_height: u32, image_width: u32, image_height: u32) -> f32 {
    // let fill = true;
    // let width = screen_width;
    // let height = screen_height;
    // let nwidth = image_width;
    // let nheight = image_height;
    // @see github.com-1ecc6299db9ec823/image-0.23.13/src/math/utils.rs:34

    let ration_w = screen_width as f32 / image_width as f32;
    let ration_h = screen_height as f32 / image_height as f32;
    ration_w.min(ration_h)
}

fn compute_next_switch() -> Instant {
    Instant::now() + Duration::new(SLEEP_SECONDS, 0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_images_recursive_test() {
        let iterator = find_images(&std::path::PathBuf::from("/home/sisso/Pictures")).unwrap();
        for file in iterator {
            println!("{:?}", file);
        }

        assert!(false);
    }

    #[test]
    fn fit_image_test() {
        assert_relative_eq!(0.5, fit_image(200, 100, 300, 200));
        assert_relative_eq!(1.0 / 3.0, fit_image(200, 100, 200, 300));
    }
}
