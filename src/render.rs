use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui::{pos2, vec2, Color32, Id, Rect};

use crate::referee::{Board, MiniTile, TileClickTarget, TileData, BOARD_DIM};

#[derive(Clone)]
pub struct ClickMessage {
    pub row: usize,
    pub column: usize,
    pub location: TileClickTarget,
}
pub enum Message {
    Print(String),
    Click(ClickMessage),
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

fn tile_ui(
    ui: &mut egui::Ui,
    size: f32,
    tile: Option<&TileData>,
    row: usize,
    column: usize,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), egui::Sense::click());

    let default_color = match tile {
        Some(_) => MiniTile::get_color(&MiniTile::Grass),
        None => Color32::GRAY,
    };

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect(rect, 0.0, default_color, visuals.bg_stroke);
    }

    if response.clicked() {
        response.ctx.data_mut(|map| {
            let id = Id::new(SUBTILE_ID);
            map.insert_temp::<ClickMessage>(
                id,
                ClickMessage {
                    row,
                    column,
                    location: TileClickTarget::Center,
                },
            );
        });
    }

    if tile.is_none() {
        return response;
    }

    let get_color = |location: &TileClickTarget| {
        if let Some(place_tile) = tile {
            if let Some(_) = place_tile.secondary_center {
                if location == &TileClickTarget::Center {
                    return Color32::LIGHT_BLUE; // FIXME need to dual color here
                }
            }
            return place_tile.at(location).get_color();
        }
        default_color
    };

    struct SquareDef {
        pub dx: i8,
        pub dy: i8,
        pub target: TileClickTarget,
    }
    let square_defns = [
        SquareDef {
            dx: 0,
            dy: 0,
            target: TileClickTarget::Center,
        },
        SquareDef {
            dx: -1,
            dy: 0,
            target: TileClickTarget::Left,
        },
        SquareDef {
            dx: 1,
            dy: 0,
            target: TileClickTarget::Right,
        },
        SquareDef {
            dx: 0,
            dy: -1,
            target: TileClickTarget::Top,
        },
        SquareDef {
            dx: 0,
            dy: 1,
            target: TileClickTarget::Bottom,
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
        let mini_response = rect_button(
            ui,
            mini_rect,
            ui.id().with(&def.target),
            get_color(&def.target),
        );
        if mini_response.clicked() {
            response.ctx.data_mut(|map| {
                let id = Id::new(SUBTILE_ID);
                map.insert_temp::<ClickMessage>(
                    id,
                    ClickMessage {
                        row,
                        column,
                        location: def.target,
                    },
                );
            });
        }
    }

    response
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

fn tile<'a>(
    size: f32,
    tile: Option<&'a TileData>,
    row: usize,
    column: usize,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| tile_ui(ui, size, tile, row, column)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(board) = self.board_channel.try_recv() {
            self.board = board
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crabcassone");
            ui.add(egui::Slider::new(&mut self.zoom, 40..=160).text("age"));

            let grid_rows = BOARD_DIM;
            let grid_cols = BOARD_DIM;
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
                                let response = ui
                                    .push_id((r, c), |ui| {
                                        ui.add(tile(self.zoom as f32, self.board.at(r, c), r, c))
                                    })
                                    .inner;
                                response.ctx.data_mut(|map| {
                                    let subtile_id = Id::new(SUBTILE_ID);
                                    let maybe_val = map.get_temp::<ClickMessage>(subtile_id);
                                    if let Some(val) = maybe_val {
                                        self.output_channel.send(Message::Click(val)).unwrap();
                                    }
                                    map.remove::<ClickMessage>(subtile_id);
                                })
                            }
                            ui.end_row();
                        }
                    });
                });
        });
    }
}
