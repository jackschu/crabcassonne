use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use eframe::egui;
use egui::{vec2, Id};

use crate::{
    board::{BoardData, ConcreteBoard, Coordinate},
    referee::Player,
    render_tile,
    tile::{Rotation, TileClickTarget, TileData},
};

#[derive(Clone)]
pub struct ClickMessage {
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub location: TileClickTarget,
}

#[derive(Clone)]
pub enum InteractionMessage {
    Print(String),
    Click(ClickMessage),
    CancelMeeple,
}

pub struct RenderState {
    pub preview_tile: Option<TileData>,
    pub board: ConcreteBoard,
    pub turn_order: Vec<Player>,
    pub is_placing_meeple: bool,
    pub current_player: Player,
    pub player_meeples: HashMap<Player, u8>,
    pub player_scores: HashMap<Player, u32>,
}

pub enum RenderMessage {
    RefereeSync(RenderState),
}

pub struct MyApp {
    zoom: usize,
    render_state: Option<RenderState>,
    pub output_channel: Sender<InteractionMessage>,
    pub input_channel: Receiver<RenderMessage>,
}

pub const TILE_CLICK_ID: &str = "subtile";

impl MyApp {
    pub fn create(
        output_channel: Sender<InteractionMessage>,
        board_channel: Receiver<RenderMessage>,
    ) -> Self {
        Self {
            zoom: 80,
            render_state: None,
            output_channel,
            input_channel: board_channel,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.input_channel.try_recv() {
            match message {
                RenderMessage::RefereeSync(state) => self.render_state = Some(state),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crabcassone");
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.zoom, 40..=160).text("zoom"));
                ui.separator();
                ui.label("Press R to rotate");
                ui.separator();
                ui.label("Press X to skip meeple placement");
            });
            if let Some(state) = &self.render_state {
                let mut score_map: HashMap<Player, u8> = HashMap::from([]);
                let score_data = state.board.get_all_scoring_data();
                for data in score_data {
                    for player in &data.scoring_players {
                        if let Some(current) = score_map.get_mut(player) {
                            *current += data.points;
                        } else {
                            score_map.insert(player.clone(), data.points);
                        }
                    }
                }
                for player in &state.turn_order {
                    ui.horizontal(|ui| {
                        ui.monospace(format!("Player: {}", player,));
                        ui.separator();
                        ui.monospace(format!(
                            "Score: {:03}",
                            state.player_scores.get(player).unwrap_or(&0),
                        ));
                        ui.separator();
                        ui.monospace(format!(
                            "Meeples: {}",
                            state.player_meeples.get(player).unwrap_or(&0)
                        ));
                        ui.separator();
                        ui.monospace(format!(
                            "Standing points: {:03}",
                            score_map.get(player).unwrap_or(&0)
                        ));
                    });
                }
                ui.horizontal(|ui| {
                    ui.strong(format!("Current Player: {}", state.current_player));
                });
            }

            let events = ui.input(|i| i.events.clone());
            for event in &events {
                #[allow(clippy::single_match)] // may expand
                match event {
                    egui::Event::Key {
                        key,
                        pressed,
                        modifiers: _modifiers,
                        repeat,
                    } => {
                        if *pressed && !repeat {
                            match key {
                                egui::Key::R => {
                                    if let Some(preview_tile) = self
                                        .render_state
                                        .as_mut()
                                        .and_then(|state| state.preview_tile.as_mut())
                                    {
                                        preview_tile.rotate_right()
                                    }
                                }
                                egui::Key::X => self
                                    .output_channel
                                    .send(InteractionMessage::CancelMeeple)
                                    .unwrap(),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            let grid = egui::Grid::new("some_unique_id").spacing(vec2(10.0, 10.0));

            if let Some(state) = &self.render_state {
                let ((min_row, max_row), (min_col, max_col)) = state.board.boundaries();
                egui::ScrollArea::both()
                    .drag_to_scroll(true)
                    .show(ui, |ui| {
                        grid.show(ui, |ui| {
                            for r in (min_row - 1)..(max_row + 1 + 1) {
                                for c in (min_col - 1)..(max_col + 1 + 1) {
                                    let coord = (r, c);
                                    let response = ui
                                        .push_id(coord, |ui| {
                                            ui.add(render_tile::tile(
                                                self.zoom as f32,
                                                state.board.at(&coord),
                                                coord,
                                                if state.is_placing_meeple {
                                                    &None
                                                } else {
                                                    &state.preview_tile
                                                },
                                                state.is_placing_meeple,
                                                state.current_player.clone(),
                                            ))
                                        })
                                        .inner;
                                    response.ctx.data_mut(|map| {
                                        let subtile_id = Id::new(TILE_CLICK_ID);
                                        let maybe_val =
                                            map.get_temp::<InteractionMessage>(subtile_id);
                                        if let Some(val) = maybe_val {
                                            self.output_channel.send(val).unwrap();
                                        }
                                        map.remove::<InteractionMessage>(subtile_id);
                                    })
                                }
                                ui.end_row();
                            }
                        });
                    });
            };
        });
    }
}
