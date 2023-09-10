use std::{sync::mpsc::channel, thread};

use crabcassonne::{
    referee::referee_main,
    render::{Message, MyApp},
};

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (sender, receiver) = channel::<Message>();

    thread::spawn(move || referee_main(receiver));

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
