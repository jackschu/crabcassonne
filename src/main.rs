use eframe::egui;
use egui::{pos2, vec2, Color32, Id, Pos2, Rect, Stroke, Ui};

fn main() -> Result<(), eframe::Error> {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Crabcassonne",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    zoom: usize,
}

const SUBTILE_ID: &str = "subtile";

impl Default for MyApp {
    fn default() -> Self {
        Self { zoom: 80 }
    }
}

pub fn board(ui: &mut Ui, columns: usize, rows: usize, roundness: f32, spacing: egui::Vec2) {
    let big_rect = ui.max_rect();
    let top = big_rect.top();
    let left = big_rect.left();

    let bottom = big_rect.bottom();
    let right = big_rect.right();

    let h = (bottom - top) / (rows as f32);
    let w = (right - left) / (columns as f32);

    let border_color = Color32::RED;
    ui.painter().rect(
        big_rect,
        0.0,
        Color32::TRANSPARENT,
        Stroke::new(1.0, border_color),
    );

    for r in 0..rows {
        for c in 0..columns {
            let pos = pos2(left + (c as f32) * w, top + (r as f32) * h);
            let rect = Rect::from_min_size(pos, vec2(w, h));
            ui.painter().rect(
                rect,
                0.,
                Color32::from_gray(64),
                Stroke::new(1., Color32::WHITE),
            );
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
        //        let rect = rect.expand(visuals.expansion);
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

pub fn tile(size: f32, on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| tile_ui(ui, size, on)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crabcassone");
            ui.add(egui::Slider::new(&mut self.zoom, 40..=160).text("age"));
            // if ui.button("Click each year").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
            // ui.vertical(|ui| {
            //     board(ui, self.age, self.age, 0.2, vec2(20.0, 20.0));
            // });
            let grid_rows = 30;
            let grid_cols = 30;
            let grid = egui::Grid::new("some_unique_id").spacing(vec2(10.0, 10.0));
            // grid = grid.striped(true);

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
                                        println!("{:?}", val);
                                    }
                                    map.remove::<String>(subtile_id);
                                })
                            }
                            ui.end_row();
                        }
                    });
                });

            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
        });
    }
}
