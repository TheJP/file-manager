mod logic;
mod main_view;
mod meta_view;

use crate::images::ImageCache;

use approximate_string_matcher::MatchResult;
use eframe::Frame;
use eframe::{egui::Context, App};
use meta::model::RootFolderId;
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
}

impl App for FileManagerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.update_meta_view(ctx);
        self.update_main_view(ctx);
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
