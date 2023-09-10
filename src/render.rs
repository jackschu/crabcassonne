use std::sync::mpsc::Sender;

use eframe::egui;
use egui::{pos2, vec2, Id, Rect};

pub enum Message {
    PrintMessage(String),
}

pub struct MyApp {
    zoom: usize,
    pub output_channel: Sender<Message>,
}

const SUBTILE_ID: &str = "subtile";

impl MyApp {
    pub fn create(output_channel: Sender<Message>) -> Self {
        Self {
            zoom: 80,
            output_channel,
        }
    }
}

fn tile_ui(ui: &mut egui::Ui, size: f32, on: &mut bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(vec2(size, size), egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, *on);
        ui.painter()
            .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);
    }

    struct SquareDef {
        pub dx: i8,
        pub dy: i8,
        pub id: String,
    }
    let square_defns = [
        SquareDef {
            dx: 0,
            dy: 0,
            id: "center".to_owned(),
        },
        SquareDef {
            dx: -1,
            dy: 0,
            id: "left".to_owned(),
        },
        SquareDef {
            dx: 1,
            dy: 0,
            id: "right".to_owned(),
        },
        SquareDef {
            dx: 0,
            dy: -1,
            id: "top".to_owned(),
        },
        SquareDef {
            dx: 0,
            dy: 1,
            id: "bottom".to_owned(),
        },
    ];

    let center = rect.center();
    let mini_size = size / 3.0;

    for def in square_defns {
        let mini_rect = Rect::from_center_size(
            pos2(
                center.x + (def.dx as f32) * mini_size,
                center.y + (def.dy as f32) * mini_size,
            ),
            vec2(mini_size, mini_size),
        );
        let mini_response = rect_button(ui, mini_rect, ui.id().with(def.id.as_str()));
        if mini_response.clicked() {
            response.ctx.data_mut(|map| {
                let id = Id::new(SUBTILE_ID);
                map.insert_temp::<String>(id, def.id);
            });
        }
    }

    return response;
}

fn rect_button(ui: &mut egui::Ui, rect: Rect, id: Id) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    if ui.is_rect_visible(rect) {
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);
    }
    response
}

fn tile(size: f32, on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| tile_ui(ui, size, on)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crabcassone");
            ui.add(egui::Slider::new(&mut self.zoom, 40..=160).text("age"));

            let grid_rows = 30;
            let grid_cols = 30;
            let grid = egui::Grid::new("some_unique_id").spacing(vec2(10.0, 10.0));

            // grid = grid.min_row_height(size);
            // grid = grid.min_col_width(size);
            // grid = grid.max_col_width(size);
            egui::ScrollArea::both()
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    grid.show(ui, |ui| {
                        for r in 0..grid_rows {
                            for c in 0..grid_cols {
                                let mut on = false;
                                let response = ui
                                    .push_id((r, c), |ui| ui.add(tile(self.zoom as f32, &mut on)))
                                    .inner;
                                response.ctx.data_mut(|map| {
                                    let subtile_id = Id::new(SUBTILE_ID);
                                    let maybe_val = map.get_temp::<String>(subtile_id);
                                    if let Some(val) = maybe_val {
                                        self.output_channel
                                            .send(Message::PrintMessage(val))
                                            .unwrap();
                                    }
                                    map.remove::<String>(subtile_id);
                                })
                            }
                            ui.end_row();
                        }
                    });
                });
        });
    }
}
