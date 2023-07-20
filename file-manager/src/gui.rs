use std::ops::ControlFlow;

use crate::images::ImageCache;

use approximate_string_matcher::{compare, MatchResult};
use eframe::egui::{Layout, RichText, TextEdit, Window};
use eframe::epaint::Color32;
use eframe::Frame;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image, Key, Ui},
    App,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

pub(crate) struct FileManagerApp {
    images: ImageCache,
    meta_window_open: bool,
    meta_search: String,
    meta_options: Vec<MetaOption>,
    meta_selected_option: isize,
    persons: Vec<String>,
}

pub(crate) enum MetaOption {
    Create,
    MatchResult(MatchResult),
}

impl From<MatchResult> for MetaOption {
    fn from(value: MatchResult) -> Self {
        MetaOption::MatchResult(value)
    }
}

impl FileManagerApp {
    pub(crate) fn new(images: ImageCache) -> Self {
        Self {
            images,
            meta_window_open: false,
            meta_search: String::new(),
            meta_options: vec![MetaOption::Create],
            meta_selected_option: 0,
            persons: vec!["Janis".into(), "Jaguar Nano".into()],
        }
    }

    fn add_image(&mut self, ctx: &Context, ui: &mut Ui) {
        let Some(image) = self.images.current_image().transpose() else {
            return;
        };

        let Ok(image) = image else {
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

    fn navigate_images(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::PageDown))
        {
            self.images.forward();
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::PageUp)) {
            self.images.back();
        }
    }

    fn meta_hotkeys(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::Num1)) {
            self.meta_search = String::new();
            // TODO: Populate with commonly used options.
            self.meta_options = vec![MetaOption::Create];
            self.meta_window_open = true;
        }
    }

    fn meta_window(&mut self, ctx: &Context) {
        if let ControlFlow::Break(_) = self.meta_window_handle_input(ctx) {
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
                            let option = &self.meta_options[option_index];
                            row.col(|ui| {
                                if self.meta_selected_option as usize == option_index {
                                    ui.painter().rect_filled(
                                        ui.max_rect().expand2(0.5 * ui.spacing().item_spacing),
                                        0.0,
                                        Color32::from_rgb(60, 60, 60),
                                    );
                                }
                                Self::add_meta_option(ui, option);
                            });
                        })
                    });
            });
    }

    fn update_meta_options(&mut self) {
        let search = self.meta_search.trim();
        self.meta_selected_option = 0;
        self.meta_options = std::iter::once(MetaOption::Create)
            .chain(
                self.persons
                    .iter()
                    .filter_map(|name| compare(search, name))
                    .map(Into::into),
            )
            .collect();
    }

    fn meta_window_handle_input(&mut self, ctx: &Context) -> ControlFlow<()> {
        let escape = ctx.input(|input| input.key_pressed(Key::Escape));
        let enter = ctx.input(|input| input.key_pressed(Key::Enter));
        if escape || enter {
            self.meta_window_open = false;
            // TODO: Handle enter
            return ControlFlow::Break(());
        }

        if ctx.input(|input| input.key_pressed(Key::ArrowUp)) {
            self.meta_selected_option -= 1;
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowDown)) {
            self.meta_selected_option += 1;
        }
        self.meta_selected_option = self
            .meta_selected_option
            .rem_euclid(self.meta_options.len() as isize);

        ControlFlow::Continue(())
    }

    fn add_meta_option(ui: &mut Ui, option: &MetaOption) {
        let _ = ui.button("+");
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            match option {
                MetaOption::Create => {
                    ui.label("Create New");
                }
                MetaOption::MatchResult(match_result) => {
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

impl App for FileManagerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.meta_window_open {
            self.navigate_images(ctx);
            self.meta_hotkeys(ctx);
        }

        if self.meta_window_open {
            self.meta_window(ctx);
        }

        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| self.add_image(ctx, ui));
                });
        });
    }
}
