use anyhow::anyhow;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

use chip8_core::*;
use std::{env, fs, process};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

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

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

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

        emu.tick();
        canvas.clear();
        canvas.present();
    }

    Ok(())
}
