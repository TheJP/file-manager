use std::{borrow::Cow, cmp::Ordering};

use approximate_string_matcher::compare;
use eframe::egui::{Context, Key};
use meta::model::PersonId;

use super::{FileManagerApp, MetaOption};
use crate::Result;

impl FileManagerApp {
    pub(crate) fn open_meta_window(&mut self) {
        self.meta_search = String::new();
        // TODO: Populate with commonly used options.
        self.meta_options = vec![MetaOption::Create];
        self.meta_window_open = true;
    }

    pub(crate) fn current_file_name(&mut self) -> Cow<'_, str> {
        let root_path = self
            .meta
            .root_folders()
            .root_folder(&self.meta_current_folder);
        self.images
            .current_image_path()
            .map_or("No File".into(), |path| {
                root_path.map_or_else(
                    || path.to_string_lossy(),
                    |root_path| path.strip_prefix(root_path).unwrap().to_string_lossy(),
                )
            })
    }

    pub(crate) fn main_view_handle_input(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::PageDown))
        {
            self.images.forward();
        }

        if ctx.input(|input| input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::PageUp)) {
            self.images.back();
        }

        if ctx.input(|input| input.key_pressed(Key::Num1)) {
            self.open_meta_window();
        }
    }

    pub(crate) fn meta_view_handle_input(&mut self, ctx: &Context) {
        let escape = ctx.input(|input| input.key_pressed(Key::Escape));
        let enter = ctx.input(|input| input.key_pressed(Key::Enter));
        if escape || enter {
            self.meta_window_open = false;
            if enter {
                self.meta_handle_confirm(self.meta_selected_option as usize)
                    .unwrap(); // TODO: Improve error handling.
            }
            return;
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
    }

    pub(crate) fn meta_handle_confirm(&mut self, option_index: usize) -> Result<()> {
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

    pub(crate) fn update_meta_options(&mut self) {
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
}
