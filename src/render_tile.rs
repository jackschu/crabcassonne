use egui::{
    epaint::{CircleShape, RectShape},
    pos2, vec2, Color32, Id, Rect, Shape, Stroke,
};

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
    is_placing_meeple: bool,
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
            let tile_type = tile.at(location);
            tile_type.get_color()
        } else {
            MiniTile::get_color(&MiniTile::Grass)
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
            let meeple_map = tile.get_meeple_locations();
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

                if let Some(target) = target {
                    if let Some(meeple_owner) = meeple_map.get(&target) {
                        meeple_paint(ui, mini_rect, meeple_owner.get_color());
                    } else if let Some(click_pos) = response.interact_pointer_pos() {
                        if is_placing_meeple && mini_rect.contains(click_pos) && response.clicked() {
                            response.ctx.data_mut(|map| {
                                let id = Id::new(TILE_CLICK_ID);
                                map.insert_temp::<InteractionMessage>(
                                    id,
                                    InteractionMessage::Click(ClickMessage {
                                        location: target,
                                        rotation: Rotation::None,
                                        coord,
                                    }),
                                );
                            });
                        }
                    } else if let Some(hover_pos) = response.hover_pos() {
                        if is_placing_meeple && mini_rect.contains(hover_pos) {
                            meeple_paint(ui, mini_rect, Color32::TRANSPARENT);
                        }
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

    if response.clicked() && !is_placing_meeple {
        response.ctx.data_mut(|map| {
            let id = Id::new(TILE_CLICK_ID);
            if map.get_temp::<InteractionMessage>(id).is_some() {
                return;
            }
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

fn rect_paint(ui: &egui::Ui, rect: Rect, color: Color32) {
    if ui.is_rect_visible(rect) {
        ui.painter().rect(rect, 0.0, color, Stroke::NONE);
    }
}

fn meeple_paint(ui: &egui::Ui, rect: Rect, color: Color32) {
    if ui.is_rect_visible(rect) {
        let l = rect.height();
        let outline = Stroke::new(1.0, Color32::BLACK);

        let meeple_head = Shape::Circle(CircleShape {
            center: rect.center() - vec2(0.0, l / 6.0),
            radius: l / 4.5,
            fill: color,
            stroke: outline,
        });
        let body_rect = rect.shrink2(vec2(l / 3.0, l / 6.0));

        let meeple_body = Shape::Rect(RectShape {
            rect: body_rect,
            rounding: 0.0.into(),
            stroke: outline,
            fill: color,
        });
        ui.painter().add(meeple_body);
        ui.painter().add(meeple_head);
    }
}

pub fn tile<'a>(
    size: f32,
    tile: Option<&'a TileData>,
    coord: Coordinate,
    preview_tile: &'a Option<TileData>,
    is_placing_meeple: bool,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| tile_ui(ui, size, tile, coord, preview_tile, is_placing_meeple)
}
