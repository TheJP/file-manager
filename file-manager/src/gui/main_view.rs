use eframe::{
    egui::{CentralPanel, Context, Image, TopBottomPanel, Ui},
    epaint::vec2,
};
use egui_extras::{Size, StripBuilder};

use super::FileManagerApp;

impl FileManagerApp {
    pub(crate) fn update_main_view(&mut self, ctx: &Context) {
        if !self.meta_window_open {
            self.main_view_handle_input(ctx);
        }

        TopBottomPanel::top(eframe::egui::Id::new("top_panel")).show(ctx, |ui| {
            self.top_panel(ui);
        });

        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| self.add_image(ctx, ui));
                });
        });

        TopBottomPanel::bottom(eframe::egui::Id::new("bottom_panel")).show(ctx, |ui| {
            self.bottom_panel(ui);
        });
    }

    fn top_panel(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            let people = ui
                .button("People")
                .on_hover_text("Add or Remove People (Hotkey: 1)");
            if people.clicked() {
                self.open_meta_window();
            }

            // TODO: Add more meta types.
            let _ = ui
                .button("Events")
                .on_hover_text("Add or Remove People (Hotkey: 1)");
        });
    }

    fn bottom_panel(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(20.0))
                .size(Size::remainder())
                .size(Size::exact(20.0))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        let left_button = &ui
                            .button("<")
                            .on_hover_text("Navigate to Previous Image (Hotkey: Left)");

                        if left_button.clicked() {
                            self.images.back();
                        }
                    });
                    strip.cell(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            let file_name = self.current_file_name();
                            ui.label(file_name);
                        });
                    });
                    strip.cell(|ui| {
                        let right_button = &ui
                            .button(">")
                            .on_hover_text("Navigate to Next Image (Hotkey: Right)");
                        if right_button.clicked() {
                            self.images.forward();
                        }
                    });
                })
        });
    }

    fn add_image(&mut self, ctx: &Context, ui: &mut Ui) {
        let Some(image) = self.images.current_image().transpose() else {
            return;
        };

        let Ok(image) = image else {
            // TODO: Improve error handling.
            eprintln!("Encountered error while loading image: {}", image.err().unwrap());
            return;
        };

        let size = ui.available_size_before_wrap().max(vec2(100.0, 100.0));
        let aspect_x = size.y * (image.width() as f32) / (image.height() as f32);
        let fit = (aspect_x <= size.x)
            .then_some(vec2(aspect_x, size.y))
            .unwrap_or_else(|| {
                vec2(
                    size.x,
                    size.x * (image.height() as f32) / (image.width() as f32),
                )
            });

        ui.vertical_centered(|ui| ui.add(Image::new(image.texture_id(ctx), fit)));
    }
}
