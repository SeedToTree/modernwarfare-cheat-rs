use memlib::logger::MinimalLogger;
use memlib::memory;
use memlib::system;
use memlib::overlay;

use log::*;
use std::error::Error;

mod sdk;
mod hacks;
mod config;
mod gui;

pub const PROCESS_NAME: &str = "ModernWarfare.exe";
pub const CHEAT_TICKRATE: u64 = 50;

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn run() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    MinimalLogger::init(LOG_LEVEL)?;

    // Create a handle to the game
    let handle = memory::Handle::from_boxed_interface(
        Box::new(memory::handle_interfaces::driver_handle::DriverProcessHandle::attach(PROCESS_NAME)?)
    );

    // Init system by connecting to RPC running on guest
    info!("Connecting to system host");
    // system::connect(&"192.168.122.129:9800".parse().unwrap()).unwrap();
    system::init().unwrap();

    // Init the overlay
    // let overlay = Box::new(overlay::looking_glass::LookingGlassOverlay::new(
    //     "/tmp/overlay-pipe",
    //     false,
    //     6
    // ).expect("Failed to create overlay"));
    let overlay = Box::new(overlay::null_overlay::NullOverlay);

    // Create a game struct from the handle
    let game = sdk::Game::new(handle)?;

    // Run the hack loop
    hacks::hack_loop(game, overlay)?;

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => {
            info!("Exiting cheat");
            0
        }
        Err(err) => {
            error!("{}", err);
            1
        }
    })
}
