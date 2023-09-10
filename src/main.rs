use std::{sync::mpsc::channel, thread};

use crabcassonne::{
    referee::referee_main,
    render::{InteractionMessage, MyApp, RenderMessage},
};

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (input_sender, input_receiver) = channel::<RenderMessage>();
    let (sender, receiver) = channel::<InteractionMessage>();

    thread::spawn(move || referee_main(receiver, input_sender));

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Crabcassonne",
        options,
        Box::new(|_cc| Box::new(MyApp::create(sender, input_receiver))),
    )
    .unwrap();
}
