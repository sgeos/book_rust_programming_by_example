extern crate palette;
extern crate sdl2;
extern crate sdl2_sys;

use palette::{Hsv, LinSrgb};
use sdl2::event::Event;
use sdl2::image::{LoadSurface, InitFlag};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

const FPS: u32 = 60;
const SLEEP_SECOND: u32 = 1_000_000_000;
const TEXTURE_SIZE: u32 = 32;

#[derive(Clone, Copy)]
enum ColorBase {
    Red,
    Green,
    Blue,
}

impl ColorBase {
    fn to_hsv(self) -> Hsv {
        match self {
            ColorBase::Red => Hsv::new::<f32>(0.0, 1.0, 1.0),
            ColorBase::Green => Hsv::new::<f32>(120.0, 1.0, 1.0),
            ColorBase::Blue => Hsv::new::<f32>(240.0, 1.0, 1.0),
        }
    }
}

pub fn main() -> Result<(), Box<Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem.window("Tetris", 800, 600).position_centered().opengl().build()?;
    let mut canvas = window.into_canvas().target_texture().present_vsync().build()?;
    let texture_creator: TextureCreator<_> = canvas.texture_creator();
    let mut square_texture: Texture = create_texture_square(&texture_creator, TEXTURE_SIZE)?;
    let mut background_surface: Surface = Surface::from_file("assets/background.png")?;
    unsafe {
      sdl2_sys::SDL_ConvertSurfaceFormat((&mut background_surface).as_ptr() as *mut sdl2_sys::SDL_Surface, sdl2_sys::SDL_PIXELFORMAT_RGBA8888 as u32, 0);
    }

    let mut done = false;
    let mut frame: u64 = 0;
    let mut event_pump = sdl_context.event_pump()?;
    let background_color_base = ColorBase::Red;
    let mut square_color_base = ColorBase::Green;
    let hsv_delta = Hsv::new::<f32>(60.0 / FPS as f32, 0.0, 0.0); // hue degrees per second
    let mut hsv_offset = Hsv::new::<f32>(0.0, 0.0, 0.0); // hue degrees per second
    while let false = done {
        frame = frame.wrapping_add(1);
        if 0 == frame % u64::from(FPS) {
            square_color_base = match square_color_base {
                ColorBase::Red => ColorBase::Green,
                ColorBase::Green => ColorBase::Blue,
                ColorBase::Blue => ColorBase::Red,
            };
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => done = true,
                _ => (),
            }
        }
        hsv_offset = hsv_offset + hsv_delta;
        canvas.set_draw_color(hsv_to_color(background_color_base.to_hsv() + hsv_offset));
        canvas.clear();
        canvas.with_texture_canvas(&mut square_texture,
            |texture| {
                texture.set_draw_color(hsv_to_color(square_color_base.to_hsv() + hsv_offset));
                texture.clear();
            }
        )?;
        //canvas.copy(&background_texture, None, None)?;
        canvas.copy(&square_texture, None, Rect::new(0, 0, TEXTURE_SIZE, TEXTURE_SIZE))?;
        canvas.present();
        sleep(Duration::new(0, SLEEP_SECOND / FPS));
    }

    Ok(())
}

fn create_hue_shifted_texture_from_surface<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    surface: &'a Surface
) -> Result<Texture<'a>, sdl2::render::TextureValueError> {
    let width = surface.width();
    let height = surface.height();
    let pitch = surface.pitch();
    let src_pixels: *const u8 = (*surface.raw()).pixels as *const u8;
    let texture = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, width, height)?;
    let dst_pixels: *mut u8 = (*texture.raw()).pixels as *mut u8;
    if src_pixels.is_null() || dst_pixels.is_null() {
      return sdl2::render::TextureValueError::SdlError("No pixel data.");
    } else if pitch < 3 {
      return sdl2::render::TextureValueError::SdlError("Unexpected pitch.");
    }
    //for i in 0..width*height*pitch {
    //}
    texture
}

fn create_texture_square<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    size: u32
) -> Result<Texture<'a>, sdl2::render::TextureValueError> {
    texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, size, size)
}

fn hsv_to_color(hsv: Hsv) -> Color {
    let rgb = LinSrgb::from(hsv);
    Color::RGBA((255.0 * rgb.red) as u8, (255.0 * rgb.green) as u8, (255.0 * rgb.blue) as u8, 255)
}

