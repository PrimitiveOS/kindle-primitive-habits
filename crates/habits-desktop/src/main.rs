use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Days;
use habits_core::{Habits, Target};
use habits_render::View;
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("build/previews"), PathBuf::from);
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut capture = |view: &View, name: &str, dark| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(view, &mut canvas, dark),
        )
    };
    let today: chrono::NaiveDate = "2026-11-20".parse()?;

    let mut model = Habits::default();
    model
        .save_habit(None, "Read 10 pages", Target::Daily)
        .unwrap();
    model
        .save_habit(None, "Practice vocabulary", Target::Weekly(4))
        .unwrap();
    model
        .save_habit(None, "Write a journal entry", Target::Weekly(3))
        .unwrap();
    for offset in [1, 2, 3, 5, 6] {
        model
            .toggle(1, today.checked_sub_days(Days::new(offset)).unwrap(), today)
            .unwrap();
    }
    assert!(model.toggle(2, today, today).unwrap());
    let mut view = View::preview(today, model);
    capture(&view, "habits-today.png", false)?;
    assert_eq!(view.input(Input::Tap(95.0, 430.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(342.0, 440.0)), Change::Full);
    capture(&view, "habits-history-dark.png", true)?;
    assert_eq!(view.input(Input::Tap(525.0, 498.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(945.0, 781.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(1020.0, 220.0)), Change::Full);
    assert_eq!(view.input(Input::Tap(550.0, 356.0)), Change::Full);
    capture(&view, "habits-target.png", false)?;
    Ok(())
}
