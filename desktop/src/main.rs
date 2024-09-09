use anyhow::anyhow;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};

use chip8_core::*;
use std::{env, fs, process};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const TICKS_PER_FRAME: usize = 10;

#[inline]
fn clear_screen(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
}

// TODO: consider dropping Result here?
fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) -> Result<(), anyhow::Error> {
    clear_screen(canvas);
    let screen_buffer = emu.get_display();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            // Convert our 1D array's index into a 2D (x,y) position
            let x = SCALE * (i % SCREEN_WIDTH) as u32;
            let y = SCALE * (i / SCREEN_WIDTH) as u32;

            // Draw a rectangle at (x,y), scaled up by our SCALE value
            let rect = Rect::new(x as i32, y as i32, SCALE, SCALE);
            canvas.fill_rect(rect).map_err(|e| anyhow!(e))?;
        }
    }

    canvas.present();
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: cargo run path/to/game");
        process::exit(1);
    }

    let game_data = fs::read(args.get(1).unwrap())?;

    // init emulator
    let mut emu = Emu::new();
    emu.load(&game_data);

    // init display
    let sdl_context = sdl2::init().map_err(|e| anyhow!(e))?;
    let video_subsystem = sdl_context.video().map_err(|e| anyhow!(e))?;

    let window = video_subsystem
        .window("CHIP-8 EMU", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    clear_screen(&mut canvas);

    let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow!(e))?;
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'game_loop,
                _ => {
                    println!("{event:#?}");
                }
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }
        emu.tick_timers();

        draw_screen(&emu, &mut canvas)?;
    }

    Ok(())
}
