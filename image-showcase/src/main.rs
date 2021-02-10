use crate::transitions::*;
use approx::assert_relative_eq;
use cgmath;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::Image;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use image::RgbaImage;
use rand::{prelude, Rng};
use rexiv2::Orientation;
use std::env;
use std::ops::{Deref, Range};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const SLEEP_SECONDS: u64 = 10;
pub const IMAGE_SCALE: Range<f32> = 0.8..1.2;
pub const IMAGE_MOVE_SPEED: Range<f32> = -2.0..2.0;
pub const IMAGE_ROTATION: Range<f32> = -0.01..0.01;
pub const IMAGE_ROTATION_SPEED: Range<f32> = -0.005..0.005;

fn main() -> GameResult<()> {
    let screen_width = 1920;
    let screen_height = 1080;

    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = screen_width as f32;
    window_mode.height = screen_height as f32;

    let (mut ctx, mut event_loop) = ContextBuilder::new("Image showcase", "Someone")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let mut images = vec![];
    let args: Vec<_> = env::args().skip(1).collect();

    if args.is_empty() {
        let path = format!(
            "{}/Pictures",
            env::var("HOME").expect("user home not found")
        );
        println!("reading path {}", path);
        let path = PathBuf::from(path);
        images.extend(find_images(path.as_ref())?);
    } else {
        for path in args {
            println!("reading path {}", path);
            let path = PathBuf::from(path);
            images.extend(find_images(path.as_ref())?);
        }
    }

    let mut app = App::new(&mut ctx, screen_width, screen_height, images)?;

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
    transition: Box<dyn Transition>,
    next_image: ImageRef,
    images: Vec<PathBuf>,
    change_image: bool,
    ignore_keys_until: f32,
    screen_width: u32,
    screen_height: u32,
    next_switch: Instant,
    history: Vec<usize>,
}

impl App {
    pub fn new(
        ctx: &mut Context,
        screen_width: u32,
        screen_height: u32,
        images: Vec<PathBuf>,
    ) -> GameResult<App> {
        let current_image = load_random_image(ctx, &images)?;

        let next_image_index = choose(images.len());
        let next_image_path = &images[next_image_index];
        let next_image_ref = Arc::new(Mutex::new(None));
        load_image_async(&next_image_path, next_image_ref.clone());

        let transition = next_transition(
            screen_width,
            screen_height,
            current_image.width() as u32,
            current_image.height() as u32,
        );

        let game = App {
            current_image: current_image,
            transition: transition,
            next_image: next_image_ref,
            images,
            change_image: false,
            ignore_keys_until: 0.0,
            screen_width,
            screen_height,
            next_switch: compute_next_switch(),
            history: vec![next_image_index],
        };
        Ok(game)
    }

    fn load_next_image(&mut self, ctx: &mut Context) -> GameResult<bool> {
        // get next image if is already loaded
        let mut image_ref = self.next_image.deref().lock().unwrap();
        let image_rgb = match image_ref.take() {
            Some(image) => image,
            _ => return Ok(false),
        };

        // trigger to load next image async
        let next_image_index = choose(self.images.len());
        let next_image_path = &self.images[next_image_index];

        self.history.push(next_image_index);
        while self.history.len() > 100 {
            self.history.remove(0);
        }

        load_image_async(next_image_path, self.next_image.clone());

        // load rbga into image
        let (width, height) = image_rgb.dimensions();
        let image = Image::from_rgba8(ctx, width as u16, height as u16, &image_rgb)?;
        self.current_image = image;

        // reset positions
        self.transition = next_transition(
            self.screen_width,
            self.screen_height,
            self.current_image.width() as u32,
            self.current_image.height() as u32,
        );

        self.change_image = false;

        Ok(true)
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta_seconds = ggez::timer::delta(ctx).as_secs_f32();
        let total_seconds = ggez::timer::time_since_start(ctx).as_secs_f32();

        self.transition.update(total_seconds, delta_seconds);

        let tick = ggez::timer::ticks(ctx);
        if total_seconds > self.ignore_keys_until {
            if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Space)
                || ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Right)
            {
                println!("loading next image");
                self.change_image = true;
                self.ignore_keys_until = total_seconds + 1.0;
            }

            if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::Left) {
                println!("loading next image");
                self.ignore_keys_until = total_seconds + 1.0;
            }
        }

        if self.next_switch <= Instant::now() {
            println!("time switch");
            self.change_image = true;
        }

        if self.change_image {
            if self.load_next_image(ctx)? {
                self.next_switch = compute_next_switch();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let pos = self.transition.pos();
        graphics::draw(
            ctx,
            &self.current_image,
            graphics::DrawParam::new()
                .dest(cgmath::Point2::new(pos.0, pos.1))
                .rotation(self.transition.rotation())
                .scale(cgmath::Vector2::new(1.0, 1.0) * self.transition.scale()),
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
    rng.gen_range(0..len)
}

fn load_image_async(path: &Path, image_ref: ImageRef) {
    let path = path.to_path_buf();

    thread::spawn(move || {
        let start_trace = Instant::now();
        println!("loading {}", path.to_string_lossy());
        let buffer = std::fs::read(&path).expect("fail to load image");
        let meta = rexiv2::Metadata::new_from_buffer(&buffer).unwrap();
        let mut dynamic_img = image::load_from_memory(&buffer).unwrap();
        match meta.get_orientation() {
            Orientation::Rotate180 => dynamic_img = dynamic_img.rotate180(),
            Orientation::Rotate90 => dynamic_img = dynamic_img.rotate90(),
            Orientation::Rotate270 => dynamic_img = dynamic_img.rotate270(),
            _ => (),
        }
        let img = dynamic_img.to_rgba8();
        *image_ref.lock().unwrap() = Some(img);
        println!(
            "loaded {} in {}ms",
            path.to_string_lossy(),
            start_trace.elapsed().as_millis()
        );
    });
}

fn load_image(ctx: &mut Context, path: &Path) -> GameResult<Image> {
    let img = image::open(path).unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    Image::from_rgba8(ctx, width as u16, height as u16, &img)
}

fn fit_image(screen_width: u32, screen_height: u32, image_width: u32, image_height: u32) -> f32 {
    let ration_w = screen_width as f32 / image_width as f32;
    let ration_h = screen_height as f32 / image_height as f32;
    ration_w.min(ration_h)
}

fn next_transition(
    screen_width: u32,
    screen_height: u32,
    image_width: u32,
    image_height: u32,
) -> Box<dyn Transition> {
    Box::new(ScaleOutTransition::new(
        screen_width,
        screen_height,
        image_width,
        image_height,
    ))
}

mod transitions {
    use super::*;
    use ggez::timer::delta;
    use rand::{thread_rng, Rng, RngCore};

    pub trait Transition {
        fn pos(&self) -> (f32, f32) {
            (0.0, 0.0)
        }
        fn scale(&self) -> f32 {
            1.0
        }
        fn rotation(&self) -> f32 {
            0.0
        }
        fn update(&mut self, total_time: f32, delta_time: f32) {}
    }

    pub struct FitTransition {
        x: f32,
        y: f32,
        scale: f32,
    }

    impl FitTransition {
        pub fn new(
            screen_width: u32,
            screen_height: u32,
            image_width: u32,
            image_height: u32,
        ) -> Self {
            let ration_w = screen_width as f32 / image_width as f32;
            let ration_h = screen_height as f32 / image_height as f32;
            let scale = ration_w.min(ration_h);
            let x = (screen_width as f32 - image_width as f32 * scale) / 2.0;
            let y = (screen_height as f32 - image_height as f32 * scale) / 2.0;
            FitTransition {
                x: x,
                y: y,
                scale: scale,
            }
        }
    }

    impl Transition for FitTransition {
        fn pos(&self) -> (f32, f32) {
            (self.x, self.y)
        }

        fn scale(&self) -> f32 {
            self.scale
        }
    }

    pub struct ScaleOutTransition {
        x: f32,
        y: f32,
        scale: f32,
        scale_step: f32,
        move_x: f32,
        move_y: f32,
        rotation: f32,
        rotation_speed: f32,
    }

    impl ScaleOutTransition {
        pub fn new(
            screen_width: u32,
            screen_height: u32,
            image_width: u32,
            image_height: u32,
        ) -> Self {
            let mut rng = thread_rng();

            let ration_w = screen_width as f32 / image_width as f32;
            let ration_h = screen_height as f32 / image_height as f32;
            let scale = ration_w.min(ration_h);
            let x = (screen_width as f32 - image_width as f32 * scale) / 2.0;
            let y = (screen_height as f32 - image_height as f32 * scale) / 2.0;

            ScaleOutTransition {
                x,
                y,
                scale,
                scale_step: rng.gen_range(IMAGE_SCALE),
                move_x: rng.gen_range(IMAGE_MOVE_SPEED),
                move_y: 0.0,
                rotation: rng.gen_range(IMAGE_ROTATION),
                rotation_speed: rng.gen_range(IMAGE_ROTATION_SPEED),
            }
        }
    }

    impl Transition for ScaleOutTransition {
        fn pos(&self) -> (f32, f32) {
            (self.x + self.move_x, self.y + self.move_y)
        }

        fn scale(&self) -> f32 {
            self.scale * self.scale_step
        }

        fn rotation(&self) -> f32 {
            self.rotation
        }

        fn update(&mut self, total_time: f32, delta_time: f32) {
            self.scale_step = commons::math::lerp(self.scale_step, 1.0, delta_time * 0.1);
            self.x += self.move_x * delta_time;
            self.rotation += delta_time * self.rotation_speed;
        }
    }
}

fn compute_next_switch() -> Instant {
    Instant::now() + Duration::new(SLEEP_SECONDS, 0)
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn fit_image_test() {
    //     assert_relative_eq!(0.5, fit_image(200, 100, 300, 200));
    //     assert_relative_eq!(1.0 / 3.0, fit_image(200, 100, 200, 300));
    // }
}
