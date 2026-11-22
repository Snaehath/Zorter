use std::fs;
use std::path::Path;
use serde::de::DeserializeOwned;
use crate::core::errors::Result;
use crate::storage::json_store::read_json_file;

pub fn recover_or_load<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }

    match read_json_file::<T>(path) {
        Ok(data) => Ok(data),
        Err(e) => {
            log::warn!("Corruption detected in {:?}: {}. Attempting .bak recovery...", path, e);
            let bak_path = path.with_extension("json.bak");
            if bak_path.exists()
                && let Ok(recovered) = read_json_file::<T>(&bak_path) {
                    log::info!("Successfully recovered {:?} from backup", path);
                    let _ = fs::copy(&bak_path, path);
                    return Ok(recovered);
                }
            log::error!("Could not recover {:?}. Falling back to default state.", path);
            Ok(T::default())
        }
    }
}
