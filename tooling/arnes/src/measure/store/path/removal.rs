use super::ManagedPath;
use crate::measure::MeasureError;
use rustix::fs::AtFlags;

impl ManagedPath {
    pub fn validate_removal(&self) -> Result<(), MeasureError> {
        validate_tree(self)
    }

    pub fn remove_tree(&self) -> Result<(), MeasureError> {
        validate_tree(self)?;
        remove_tree(self)
    }
}

fn validate_tree(path: &ManagedPath) -> Result<(), MeasureError> {
    path.open_directory()?;
    for name in path.read_dir_names()? {
        let child = path.join(name);
        if child.open_directory().is_ok() {
            validate_tree(&child)?;
        } else {
            child.open_read()?;
        }
    }
    Ok(())
}

fn remove_tree(path: &ManagedPath) -> Result<(), MeasureError> {
    for name in path.read_dir_names()? {
        let child = path.join(name);
        if child.open_directory().is_ok() {
            remove_tree(&child)?;
        } else {
            child.open_read()?;
            child.remove_file()?;
        }
    }
    let (directory, name) = path.parent_and_name()?;
    rustix::fs::unlinkat(directory, name, AtFlags::REMOVEDIR)?;
    Ok(())
}
