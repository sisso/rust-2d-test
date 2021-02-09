use crate::transitions::*;
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
    transition: Box<dyn Transition>,
    next_image: ImageRef,
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
        self.transition = next_transition(
            self.screen_width,
            self.screen_height,
            self.current_image.width() as u32,
            self.current_image.height() as u32,
        );

        // block any new input
        self.ignore_keys_until = ggez::timer::ticks(ctx) + 60;
        self.change_image = false;

        Ok(())
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta_seconds = ggez::timer::delta(ctx).as_secs_f32();
        let total_seconds = ggez::timer::time_since_start(ctx).as_secs_f32();

        self.transition.update(total_seconds, delta_seconds);

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
    use crate::SLEEP_SECONDS;
    use ggez::timer::delta;
    use rand::{thread_rng, Rng};

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
            let scale_step = 0.8 + rng.next_f32() * 0.4;

            let move_x = rng.next_f32() * 40.0 - 20.0;
            let move_y = rng.next_f32() * 10.0 - 5.0;

            ScaleOutTransition {
                x,
                y,
                scale,
                scale_step,
                move_x,
                move_y,
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

        fn update(&mut self, total_time: f32, delta_time: f32) {
            self.scale_step = commons::math::lerp(self.scale_step, 1.0, delta_time * 0.1);
            self.move_x += delta_time * 10.0;
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
    //
    // #[test]
    // fn fill_image_hor_test() {
    //     let (r, mx, my) = fill_image(200, 100, 300, 100);
    //     assert_relative_eq!(1.0, r);
    //     assert_relative_eq!(-1.5, mx);
    //     assert_relative_eq!(0.0, my);
    // }
    //
    // #[test]
    // fn fill_image_ver_test() {
    //     let (r, mx, my) = fill_image(200, 100, 100, 200);
    //     assert_relative_eq!(2.0, r);
    //     assert_relative_eq!(0.0, mx);
    //     assert_relative_eq!(-2.0, my);
    // }
}
