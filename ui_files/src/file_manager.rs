use std::path::{Path, PathBuf};

///
/// The file manager is used to retrieve what files are available and organize them
/// 
pub trait FileManager : Send+Sync {
    ///
    /// Returns a list of all the files that can be opened by this manager
    /// 
    fn get_all_files(&self) -> Vec<PathBuf>;

    ///
    /// Returns the display name for a particular path
    /// 
    fn display_name_for_path(&self, path: &Path) -> Option<String>;

    ///
    /// Reserves a path for a new file (this path is valid and won't be re-used by future calls but
    /// no files will exist here yet)
    /// 
    fn create_new_path(&self) -> PathBuf;

    ///
    /// Updates or creates the display name associated with a particular path (which must be
    /// returned via get_all_files: setting the name for a non-existent path will just
    /// result)
    ///
    fn set_display_name_for_path(&self, path: &Path, display_name: String);
}