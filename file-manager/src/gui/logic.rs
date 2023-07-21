use std::borrow::Cow;

use super::FileManagerApp;

impl FileManagerApp {
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
}
