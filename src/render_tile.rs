use egui::{pos2, vec2, Color32, Id, Rect, Stroke};

use crate::{
    board::{Coordinate, OCTAL_DELTAS},
    render::{ClickMessage, InteractionMessage, TILE_CLICK_ID},
    tile::{MiniTile, Rotation, TileClickTarget, TileData},
};

fn tile_ui(
    ui: &mut egui::Ui,
    size: f32,
    tile: Option<&TileData>,
    coord: Coordinate,
    preview_tile: &Option<TileData>,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), egui::Sense::click());

    let minis = OCTAL_DELTAS.iter().chain([&(0, 0)]);

    let center = rect.center();
    let mini_size = size / 3.0;

    let get_color = |location: &Option<TileClickTarget>, tile: &TileData| {
        if tile.secondary_center.is_some() && location == &Some(TileClickTarget::Center) {
            return Color32::LIGHT_BLUE; // FIXME need to dual color here
        }
        if let Some(location) = location {
            let tile_type = tile.at(&location);
            return tile_type.get_color();
        } else {
            return MiniTile::get_color(&MiniTile::Grass);
        }
    };

    let emblem_rect = Rect::from_center_size(
        pos2(center.x + -1_f32 * mini_size, center.y + -1_f32 * mini_size),
        vec2(mini_size / 1.5, mini_size / 1.5),
    );
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let resolved_tile = tile.or(if response.hovered() {
            preview_tile.as_ref()
        } else {
            None
        });
        let is_preview = tile.is_none();

        if let Some(tile) = resolved_tile {
            for mini_coord in minis {
                let mini_rect = Rect::from_center_size(
                    pos2(
                        center.x + (mini_coord.0 as f32) * mini_size,
                        center.y + (mini_coord.1 as f32) * mini_size,
                    ),
                    vec2(mini_size, mini_size),
                );
                let target = TileClickTarget::from_octal(*mini_coord);

                let mut color = get_color(&target, tile);
                if is_preview {
                    color = color.gamma_multiply(0.5);
                }

                rect_paint(ui, mini_rect, color);

                if let Some(click_pos) = response.interact_pointer_pos() {
                    if response.clicked() && mini_rect.contains(click_pos) {
                        println!("Meeple place? {:?} {:?}", target, coord);
                        response.ctx.data_mut(|map| {
                            let id = Id::new(TILE_CLICK_ID);
                            map.insert_temp::<InteractionMessage>(
                                id,
                                InteractionMessage::Print(format!(
                                    "Meeple place? {:?} {:?}",
                                    target, coord
                                )),
                            );
                        });
                    }
                }
            }
            let emblem_color = Color32::from_rgb(133, 50, 168);

            if tile.has_emblem {
                let emblem_color = if is_preview {
                    emblem_color.gamma_multiply(0.5)
                } else {
                    emblem_color
                };
                rect_paint(ui, emblem_rect, emblem_color);
            }
        } else {
            ui.painter()
                .rect(rect, 0.0, Color32::GRAY, visuals.bg_stroke);
        }
    }

    if response.clicked() {
        response.ctx.data_mut(|map| {
            let id = Id::new(TILE_CLICK_ID);
            map.insert_temp::<InteractionMessage>(
                id,
                InteractionMessage::Click(ClickMessage {
                    coord,
                    location: TileClickTarget::Center,
                    rotation: if let Some(tile) = preview_tile {
                        tile.rotation.clone()
                    } else {
                        Rotation::None
                    },
                }),
            );
        });
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

fn rect_paint(ui: &egui::Ui, rect: Rect, color: Color32) {
    if ui.is_rect_visible(rect) {
        ui.painter().rect(rect, 0.0, color, Stroke::NONE);
    }
}

pub fn tile<'a>(
    size: f32,
    tile: Option<&'a TileData>,
    coord: Coordinate,
    preview_tile: &'a Option<TileData>,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| tile_ui(ui, size, tile, coord, preview_tile)
}
