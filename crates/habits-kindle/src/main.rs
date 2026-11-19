use std::path::PathBuf;

fn main() -> Result<(), String> {
    let directory = std::env::var_os("PRIMITIVE_APP_DIR")
        .map_or_else(|| PathBuf::from("/mnt/us/habits"), PathBuf::from);
    primitive_app::run(habits_render::View::open(directory.clone()), &directory)
}
