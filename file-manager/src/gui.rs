use std::cmp::Ordering;
use std::ops::ControlFlow;

use crate::{images::ImageCache, Result};

use approximate_string_matcher::{compare, MatchResult};
use eframe::egui::{Layout, RichText, TextEdit, TopBottomPanel, Window};
use eframe::epaint::Color32;
use eframe::Frame;
use eframe::{
    egui::{vec2, CentralPanel, Context, Image, Key, Ui},
    App,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use meta::model::{PersonId, RootFolderId};
use meta::Repository;

pub(crate) struct FileManagerApp {
    images: ImageCache,
    meta: Repository,
    meta_current_folder: RootFolderId,
    meta_window_open: bool,
    meta_search: String,
    meta_options: Vec<MetaOption>,
    meta_selected_option: isize,
}

pub(crate) enum MetaOption {
    Create,
    MatchResult(MatchResult, usize),
}

impl FileManagerApp {
    pub(crate) fn new(
        images: ImageCache,
        meta: Repository,
        meta_current_folder: RootFolderId,
    ) -> Self {
        Self {
            images,
            meta,
            meta_current_folder,
            meta_window_open: false,
            meta_search: String::new(),
            meta_options: vec![MetaOption::Create],
            meta_selected_option: 0,
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
            self.open_meta_window();
        }
    }

    fn open_meta_window(&mut self) {
        self.meta_search = String::new();
        // TODO: Populate with commonly used options.
        self.meta_options = vec![MetaOption::Create];
        self.meta_window_open = true;
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

    fn update_meta_options(&mut self) {
        let search = self.meta_search.trim();
        self.meta_selected_option = 0;
        self.meta_options = std::iter::once(MetaOption::Create)
            .chain(
                self.meta
                    .persons()
                    .entries()
                    .iter()
                    .filter_map(|(id, person)| {
                        compare(search, &person.name)
                            .map(|result| MetaOption::MatchResult(result, id.0))
                    })
                    .take(10),
            )
            .collect();

        self.meta_options.sort_by(|a, b| match (a, b) {
            (MetaOption::Create, MetaOption::Create) => Ordering::Equal,
            (MetaOption::Create, _) => Ordering::Less,
            (_, MetaOption::Create) => Ordering::Greater,
            (MetaOption::MatchResult(a, _), MetaOption::MatchResult(b, _)) => {
                b.score().cmp(&a.score())
            }
        })
    }

    fn meta_window_handle_input(&mut self, ctx: &Context) -> ControlFlow<()> {
        let escape = ctx.input(|input| input.key_pressed(Key::Escape));
        let enter = ctx.input(|input| input.key_pressed(Key::Enter));
        if escape || enter {
            self.meta_window_open = false;
            if enter {
                self.meta_handle_confirm(self.meta_selected_option as usize)
                    .unwrap(); // TODO: Improve error handling.
            }
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

    fn meta_handle_confirm(&mut self, option_index: usize) -> Result<()> {
        self.meta_window_open = false;
        let option = &self.meta_options[option_index];
        match option {
            MetaOption::Create => todo!(),
            MetaOption::MatchResult(_, id) => {
                let Some(image_path) = self.images.current_image_path() else {
                    return Ok(());
                };
                self.meta
                    .load_or_create_file(&self.meta_current_folder, image_path)?
                    .persons
                    .insert(PersonId(*id));
            }
        }

        Ok(())
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
        ui.vertical_centered_justified(|ui| {
            let root_path = self
                .meta
                .root_folders()
                .root_folder(&self.meta_current_folder);
            let file_name = self
                .images
                .current_image_path()
                .map_or("No File".into(), |path| {
                    root_path.map_or_else(
                        || path.to_string_lossy(),
                        |root_path| path.strip_prefix(root_path).unwrap().to_string_lossy(),
                    )
                });
            ui.label(file_name);
        });
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

        TopBottomPanel::top(eframe::egui::Id::new("top_panel")).show(ctx, |ui| {
            self.top_panel(ui);
        });

        TopBottomPanel::bottom(eframe::egui::Id::new("bottom_panel")).show(ctx, |ui| {
            self.bottom_panel(ui);
        });

        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| self.add_image(ctx, ui));
                });
        });
    }

    fn on_close_event(&mut self) -> bool {
        let result = self.meta.save();
        if let Err(error) = result {
            eprintln!("Encountered error when saving meta data: {}", error);
            false
        } else {
            true
        }
    }
}
