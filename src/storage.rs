use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::logic::tasks::TaskDocument;

pub(crate) fn task_file() -> io::Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "ZFordDev", "FIFO Tasker")
        .ok_or_else(|| io::Error::other("Cannot locate the user's app-data directory"))?;
    Ok(dirs.data_local_dir().join("tasks.json"))
}

// Read only: a broken file gets an explanation, not an unsolicited makeover.
pub(crate) fn load(path: &Path) -> io::Result<Option<TaskDocument>> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("Cannot read {}: {error}", path.display()),
            ));
        }
    };
    TaskDocument::from_json(&json).map(Some).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid task data in {}: {error}", path.display()),
        )
    })
}
