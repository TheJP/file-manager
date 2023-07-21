use approximate_string_matcher::MatchResult;
use eframe::{
    egui::{Context, Layout, RichText, TextEdit, Ui, Window},
    epaint::Color32,
};
use egui_extras::{Column, TableBuilder};

use super::{FileManagerApp, MetaOption};

impl FileManagerApp {
    pub(crate) fn update_meta_view(&mut self, ctx: &Context) {
        if self.meta_window_open {
            self.meta_view_handle_input(ctx);
        }

        if !self.meta_window_open {
            return;
        }

        Window::new("People")
            .id(eframe::egui::Id::new("meta_window"))
            .collapsible(false)
            .show(ctx, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(Layout::left_to_right(eframe::emath::Align::Center))
                    .column(Column::remainder().at_least(100.0))
                    .header(25.0, |mut header| {
                        header.col(|ui| {
                            let response = ui.add(
                                TextEdit::singleline(&mut self.meta_search).hint_text("Search"),
                            );
                            response.request_focus();
                            if response.changed() {
                                self.update_meta_options();
                            }
                        });
                    })
                    .body(|body| {
                        body.rows(25.0, self.meta_options.len(), |option_index, mut row| {
                            row.col(|ui| {
                                if self.meta_selected_option as usize == option_index {
                                    ui.painter().rect_filled(
                                        ui.max_rect().expand2(0.5 * ui.spacing().item_spacing),
                                        0.0,
                                        Color32::from_rgb(60, 60, 60),
                                    );
                                }
                                self.add_meta_option(ui, option_index);
                            });
                        })
                    });
            });
    }

    fn add_meta_option(&mut self, ui: &mut Ui, option_index: usize) {
        if ui.button("+").clicked() {
            self.meta_handle_confirm(option_index).unwrap(); // TODO: Improve error handling.
        }

        let option = &self.meta_options[option_index];
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            match option {
                MetaOption::Create => {
                    ui.label("Create New");
                }
                MetaOption::MatchResult(match_result, _) => {
                    Self::add_meta_option_match_result(match_result, ui);
                }
            }
        });
    }

    fn add_meta_option_match_result(match_result: &MatchResult, ui: &mut Ui) {
        let mut matches = match_result.matches();
        for (index, letter) in match_result.target().chars().enumerate() {
            let mut letter = RichText::new(letter);
            while !matches.is_empty() && matches[0] < index {
                matches = &matches[1..];
            }
            if !matches.is_empty() && matches[0] == index {
                letter = letter.color(Color32::GREEN);
            }
            ui.label(letter);
        }
    }
}
