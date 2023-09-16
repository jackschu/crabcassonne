use egui::{pos2, vec2, Color32, Id, Rect, Stroke};

use crate::{
    board::Coordinate,
    render::{ClickMessage, SUBTILE_ID},
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

    let default_color = match tile {
        Some(_) => MiniTile::get_color(&MiniTile::Grass),
        None => Color32::GRAY,
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

    let get_color =
        |location: &TileClickTarget, tile: Option<&TileData>, transparent_grass: bool| {
            if let Some(place_tile) = tile {
                if place_tile.secondary_center.is_some() && location == &TileClickTarget::Center {
                    return Color32::LIGHT_BLUE; // FIXME need to dual color here
                }
                let tile_type = place_tile.at(location);
                if transparent_grass && tile_type == &MiniTile::Grass {
                    return Color32::TRANSPARENT;
                }
                return tile_type.get_color();
            }
            default_color
        };

    let emblem_rect = Rect::from_center_size(
        pos2(center.x + -1_f32 * mini_size, center.y + -1_f32 * mini_size),
        vec2(mini_size / 1.5, mini_size / 1.5),
    );
    let emblem_color = Color32::from_rgb(133, 50, 168);
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let preview_tile = if response.hovered() && tile.is_none() {
            preview_tile
        } else {
            &None
        };
        if let Some(tile) = preview_tile {
            let grass = MiniTile::get_color(&MiniTile::Grass);
            ui.painter()
                .rect(rect, 0.0, grass.gamma_multiply(0.5), visuals.bg_stroke);
            for def in &square_defns {
                let mini_rect = Rect::from_center_size(
                    pos2(
                        center.x + (def.dx as f32) * mini_size,
                        center.y + (def.dy as f32) * mini_size,
                    ),
                    vec2(mini_size, mini_size),
                );
                rect_paint(
                    ui,
                    mini_rect,
                    get_color(&def.target, Some(tile), true).gamma_multiply(0.5),
                );
            }
            if tile.has_emblem {
                rect_paint(ui, emblem_rect, emblem_color.gamma_multiply(0.5));
            }
        } else {
            ui.painter()
                .rect(rect, 0.0, default_color, visuals.bg_stroke);
        }
    }

    if response.clicked() {
        response.ctx.data_mut(|map| {
            let id = Id::new(SUBTILE_ID);
            map.insert_temp::<ClickMessage>(
                id,
                ClickMessage {
                    coord,
                    location: TileClickTarget::Center,
                    rotation: if let Some(tile) = preview_tile {
                        tile.rotation.clone()
                    } else {
                        Rotation::None
                    },
                },
            );
        });
    }

    if tile.is_none() {
        return response;
    }

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
            get_color(&def.target, tile, false),
        );
        if mini_response.clicked() {
            response.ctx.data_mut(|map| {
                let id = Id::new(SUBTILE_ID);
                map.insert_temp::<ClickMessage>(
                    id,
                    ClickMessage {
                        coord,
                        location: def.target,
                        rotation: Rotation::None,
                    },
                );
            });
        }
    }
    if tile.map(|tile| tile.has_emblem).unwrap_or(false) {
        rect_paint(ui, emblem_rect, emblem_color);
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

fn rect_paint(ui: &mut egui::Ui, rect: Rect, color: Color32) {
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
