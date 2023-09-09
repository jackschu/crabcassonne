use eframe::egui;
use egui::{pos2, vec2, Color32, Pos2, Rect, Stroke, Ui};

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
    name: String,
    age: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 80,
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
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);
    }

    let center = rect.center();
    let mini_size = size / 3.0;
    let mini_rect = Rect::from_center_size(center, vec2(mini_size, mini_size));
    let parent_id = ui.id();

    let mini_response = ui.interact(mini_rect, parent_id.with("middle"), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let mini_visuals = ui.style().interact_selectable(&mini_response, *on);
        let mini_rect = mini_rect.expand(mini_visuals.expansion);
        ui.painter()
            .rect(mini_rect, 0.0, mini_visuals.bg_fill, mini_visuals.bg_stroke);
    }

    return response;
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

pub fn tile(size: f32, on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| tile_ui(ui, size, on)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crabcassone");
            ui.add(egui::Slider::new(&mut self.age, 40..=160).text("age"));
            // if ui.button("Click each year").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
            // ui.vertical(|ui| {
            //     board(ui, self.age, self.age, 0.2, vec2(20.0, 20.0));
            // });
            let grid_rows = 3;
            let grid_cols = 3;
            let grid = egui::Grid::new("some_unique_id");
            // grid = grid.striped(true);

            // grid = grid.min_row_height(size);
            // grid = grid.min_col_width(size);
            // grid = grid.max_col_width(size);

            grid.show(ui, |ui| {
                for r in 0..grid_rows {
                    for c in 0..grid_cols {
                        let mut on = false;
                        ui.push_id((r, c), |ui| ui.add(tile(self.age as f32, &mut on)));
                    }
                    ui.end_row();
                }
            });

            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
        });
    }
}
