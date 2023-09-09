use std::{sync::mpsc::channel, thread};

use crabcassonne::render::MyApp;

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (sender, receiver) = channel::<String>();

    thread::spawn(move || loop {
        let message = receiver.recv().unwrap();
        println!("recv {}", message);
    });

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Crabcassonne",
        options,
        Box::new(|_cc| Box::new(MyApp::create(sender))),
    );
}
