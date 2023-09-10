use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui::{pos2, vec2, Color32, Id, Rect};

use crate::referee::{Board, PlacedTile};

pub enum Message {
    PrintMessage(String),
}

pub struct MyApp {
    zoom: usize,
    board: Board,
    pub output_channel: Sender<Message>,
    pub board_channel: Receiver<Board>,
}

const SUBTILE_ID: &str = "subtile";

impl MyApp {
    pub fn create(output_channel: Sender<Message>, board_channel: Receiver<Board>) -> Self {
        Self {
            zoom: 80,
            board: Board::default(),
            output_channel,
            board_channel,
        }
    }
}

fn tile_ui(ui: &mut egui::Ui, size: f32, tile: &Option<PlacedTile>) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), egui::Sense::click());

    let default_color = Color32::GRAY;

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect(rect, 0.0, default_color, visuals.bg_stroke);
    }

    let get_color = |location: usize| {
        if let Some(place_tile) = tile {
            return place_tile.data[location].getColor();
        }
        return default_color;
    };

    struct SquareDef {
        pub dx: i8,
        pub dy: i8,
        pub id: String,
        pub color: Color32,
    }
    let square_defns = [
        SquareDef {
            dx: 0,
            dy: 0,
            id: "center".to_owned(),
            color: get_color(2),
        },
        SquareDef {
            dx: -1,
            dy: 0,
            id: "left".to_owned(),
            color: get_color(1),
        },
        SquareDef {
            dx: 1,
            dy: 0,
            id: "right".to_owned(),
            color: get_color(3),
        },
        SquareDef {
            dx: 0,
            dy: -1,
            id: "top".to_owned(),
            color: get_color(0),
        },
        SquareDef {
            dx: 0,
            dy: 1,
            id: "bottom".to_owned(),
            color: get_color(4),
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
        let mini_response = rect_button(ui, mini_rect, ui.id().with(def.id.as_str()), def.color);
        if mini_response.clicked() {
            response.ctx.data_mut(|map| {
                let id = Id::new(SUBTILE_ID);
                map.insert_temp::<String>(id, def.id);
            });
        }
    }

    return response;
}

fn rect_button(ui: &mut egui::Ui, rect: Rect, id: Id, color: Color32) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    if ui.is_rect_visible(rect) {
        let rect = rect.expand(visuals.expansion);
        ui.painter().rect(rect, 0.0, color, visuals.bg_stroke);
    }
    response
}

fn tile(size: f32, tile: &Option<PlacedTile>) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| tile_ui(ui, size, tile)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        loop {
            match self.board_channel.try_recv() {
                Ok(board) => self.board = board,
                Err(_) => {
                    break;
                }
            }
        }

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
                                    .push_id((r, c), |ui| {
                                        ui.add(tile(self.zoom as f32, self.board.at(r, c)))
                                    })
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
