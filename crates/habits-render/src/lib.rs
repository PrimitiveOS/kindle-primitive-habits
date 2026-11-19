mod common;

use std::{io, path::PathBuf, time::SystemTime};

use chrono::{DateTime, Days, NaiveDate, Utc};
use habits_core::{Habit, Habits, Target};
use primitive_app::{App, Change, WIDTH};
use primitive_ui::{
    back_button, header, Canvas, Editor, Icon, Input, Press, Rect, Sheet, SheetAction, TextStyle,
    CAPTION,
};

use common::{
    editor, field, local_day, page_buttons, page_count, page_step, page_tap, press_key, rect, row,
    State, PAGE_SIZE,
};

const CHECKMARK: Icon = Icon {
    stroke: &["M5 12 L10 17 L20 7"],
    fill: &[],
};
const ADD: Rect = Rect::new(846, 170, 378, 104);
const EXPORT: Rect = Rect::new(538, 170, 282, 104);
const NAME: Rect = Rect::new(40, 174, 1184, 104);
const CHANGE_TARGET: Rect = Rect::new(40, 311, 1184, 112);
const SAVE: Rect = Rect::new(846, 458, 378, 104);
const REMOVE: Rect = Rect::new(40, 458, 276, 104);
const EDIT: Rect = Rect::new(846, 170, 378, 104);
const PREVIOUS_DAYS: Rect = Rect::new(40, 570, 365, 104);
const NEXT_DAYS: Rect = Rect::new(859, 570, 365, 104);
const APPLY_DAY: Rect = Rect::new(846, 724, 378, 104);

struct Form {
    id: Option<u64>,
    name: Editor,
    target: Target,
    typing: bool,
}

struct Detail {
    id: u64,
    end: NaiveDate,
    selected: NaiveDate,
}

pub struct View {
    state: State<Habits>,
    today: NaiveDate,
    live: bool,
    documents: Option<PathBuf>,
    page: usize,
    detail: Option<Detail>,
    form: Option<Form>,
    choosing_target: bool,
    confirming: bool,
    notice: String,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let documents = path.parent().map(|parent| parent.join("documents"));
        Self::new(
            State::open(path.join("habits.json")),
            local_day(),
            documents,
            true,
        )
    }

    pub fn preview(today: NaiveDate, data: Habits) -> Self {
        Self::new(State::preview(data), today, None, false)
    }

    fn new(state: State<Habits>, today: NaiveDate, documents: Option<PathBuf>, live: bool) -> Self {
        Self {
            state,
            today,
            live,
            documents,
            page: 0,
            detail: None,
            form: None,
            choosing_target: false,
            confirming: false,
            notice: String::new(),
        }
    }

    fn notice(&self) -> &str {
        self.state.error.as_deref().unwrap_or(&self.notice)
    }

    fn start_form(&mut self, id: Option<u64>) -> Change {
        let (name, target) = id
            .and_then(|id| self.state.data.habit(id))
            .map_or(("", Target::Daily), |item| {
                (item.name.as_str(), item.target)
            });
        self.form = Some(Form {
            id,
            name: editor(name, 100),
            target,
            typing: true,
        });
        self.notice.clear();
        Change::Full
    }

    fn write(&mut self, next: Habits, message: &str) -> bool {
        match self.state.replace(next) {
            Ok(()) => {
                self.notice = message.into();
                self.page = self.page.min(page_count(self.state.data.habits.len()) - 1);
                true
            }
            Err(error) => {
                self.notice = format!("Could not save. {error}");
                false
            }
        }
    }

    fn toggle(&mut self, id: u64, date: NaiveDate) -> Change {
        let mut next = self.state.data.clone();
        match next.toggle(id, date, self.today) {
            Ok(true) => {
                self.write(next, "Check-in saved.");
            }
            Ok(false) => {
                self.write(next, "Check-in cleared.");
            }
            Err(error) => self.notice = error.into(),
        }
        Change::Partial
    }

    fn save_form(&mut self) -> Change {
        let Some(form) = &self.form else {
            return Change::None;
        };
        let mut next = self.state.data.clone();
        match next.save_habit(form.id, &form.name.string(), form.target) {
            Ok(()) if self.write(next, "Saved.") => {
                self.form = None;
                Change::Full
            }
            Ok(()) => Change::Partial,
            Err(error) => {
                self.notice = error.into();
                Change::Partial
            }
        }
    }

    fn export(&mut self) -> Change {
        if self.state.error.is_some() {
            return Change::None;
        }
        let now = DateTime::<Utc>::from(SystemTime::now());
        let name = format!("Habits-{}-{}.csv", self.today, now.timestamp_millis());
        let result = match &self.documents {
            Some(directory) => {
                primitive_app::save(&directory.join(&name), self.state.data.csv().as_bytes())
            }
            None => Ok(()),
        };
        self.notice = match result {
            Ok(()) => format!("Saved {name} in Documents."),
            Err(error) => format!("Could not export. {error}"),
        };
        Change::Partial
    }

    fn draw_home(&self, canvas: &mut Canvas) {
        header(canvas, "Habits", "Done");
        canvas.text(rect(40, 175, 465, 100), "Today", TextStyle::new(52).bold());
        canvas.button(ADD, "Add habit", true, false);
        if !self.state.data.habits.is_empty() {
            canvas.button(EXPORT, "Export CSV", false, false);
        }
        canvas.text(
            rect(40, 298, 1184, 65),
            &self.today.format("%A, %-d %B").to_string(),
            CAPTION,
        );
        for (index, habit) in self
            .state
            .data
            .habits
            .iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
        {
            let check = check_bounds(index);
            let pressed = canvas.control_pressed(check, false);
            let done = habit.dates.contains(&self.today);
            canvas.panel(check.inset(27), 7, 4, if pressed || done { 0 } else { 255 });
            if done {
                canvas.icon(check.inset(32), CHECKMARK, 255);
            }
            row(
                canvas,
                habit_bounds(index),
                &habit.name,
                &format!(
                    "This week · {} of {} days",
                    habit.week_count(self.today),
                    habit.target.days()
                ),
            );
        }
        if self.state.data.habits.is_empty() {
            canvas.text(
                rect(40, 422, 1184, 95),
                "Add a habit to start checking in.",
                CAPTION,
            );
        }
        canvas.text(rect(40, 1280, 1184, 125), self.notice(), CAPTION);
        page_buttons(canvas, self.page, self.state.data.habits.len());
    }

    fn draw_detail(&self, canvas: &mut Canvas, detail: &Detail, habit: &Habit) {
        header(canvas, "Habit", "Back");
        canvas.text(
            rect(40, 172, 780, 106),
            &habit.name,
            TextStyle::new(45).bold(),
        );
        canvas.button(EDIT, "Edit habit", false, false);
        canvas.text(
            rect(40, 296, 1184, 80),
            &format!(
                "This week · {} of {} days",
                habit.week_count(self.today),
                habit.target.days()
            ),
            CAPTION,
        );
        for (index, date) in days(detail.end).enumerate() {
            let cell = day_bounds(index);
            canvas.text(
                Rect::new(cell.x, 401, cell.w, 54),
                &date.format("%a %-d").to_string(),
                CAPTION.center(),
            );
            canvas.button(
                cell,
                if habit.dates.contains(&date) {
                    "Done"
                } else {
                    "—"
                },
                date == detail.selected,
                false,
            );
        }
        canvas.button(PREVIOUS_DAYS, "Previous 7 days", false, false);
        if detail.end < self.today {
            canvas.button(NEXT_DAYS, "Next 7 days", false, false);
        }
        let selected = detail.selected.format("%A, %-d %B %Y");
        canvas.text(
            rect(40, 728, 780, 100),
            &format!("Edit day · {selected}"),
            TextStyle::new(35).bold(),
        );
        canvas.button(
            APPLY_DAY,
            if habit.dates.contains(&detail.selected) {
                "Clear check-in"
            } else {
                "Mark done"
            },
            true,
            false,
        );
        canvas.text(rect(40, 870, 1184, 100), self.notice(), CAPTION);
    }

    fn draw_form(&self, canvas: &mut Canvas, form: &Form) {
        header(
            canvas,
            if form.id.is_some() {
                "Edit habit"
            } else {
                "Add habit"
            },
            "Back",
        );
        field(canvas, NAME, "Habit name", &form.name, form.typing);
        row(canvas, CHANGE_TARGET, "Target", &form.target.label());
        canvas.button(SAVE, "Save habit", true, false);
        if form.id.is_some() {
            canvas.button(REMOVE, "Remove", false, false);
        }
        canvas.text(rect(40, 612, 1184, 120), self.notice(), CAPTION);
        if form.typing && !self.confirming {
            canvas.keyboard(form.name.layer, "Done", None);
        }
    }

    fn tap_home(&mut self, x: f32, y: f32) -> Change {
        if back_button(WIDTH as i32).contains(x, y) {
            return Change::Exit;
        }
        if ADD.contains(x, y) {
            return self.start_form(None);
        }
        if !self.state.data.habits.is_empty() && EXPORT.contains(x, y) {
            return self.export();
        }
        if page_tap(&mut self.page, self.state.data.habits.len(), x, y) {
            return Change::Full;
        }
        for (index, habit) in self
            .state
            .data
            .habits
            .iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .enumerate()
        {
            if check_bounds(index).contains(x, y) {
                return self.toggle(habit.id, self.today);
            }
            if habit_bounds(index).contains(x, y) {
                self.detail = Some(Detail {
                    id: habit.id,
                    end: self.today,
                    selected: self.today,
                });
                self.notice.clear();
                return Change::Full;
            }
        }
        Change::None
    }

    fn tap_detail(&mut self, x: f32, y: f32) -> Change {
        let Some(detail) = &mut self.detail else {
            return Change::None;
        };
        if back_button(WIDTH as i32).contains(x, y) {
            self.detail = None;
            self.notice.clear();
            return Change::Full;
        }
        let id = detail.id;
        if EDIT.contains(x, y) {
            return self.start_form(Some(id));
        }
        for (index, date) in days(detail.end).enumerate() {
            if day_bounds(index).contains(x, y) {
                detail.selected = date;
                self.notice.clear();
                return Change::Partial;
            }
        }
        let moved = if PREVIOUS_DAYS.contains(x, y) {
            detail.end.checked_sub_days(Days::new(7))
        } else if detail.end < self.today && NEXT_DAYS.contains(x, y) {
            detail
                .end
                .checked_add_days(Days::new(7))
                .map(|end| end.min(self.today))
        } else {
            None
        };
        if let Some(end) = moved {
            detail.end = end;
            detail.selected = end;
            self.notice.clear();
            return Change::Full;
        }
        if APPLY_DAY.contains(x, y) {
            let date = detail.selected;
            return self.toggle(id, date);
        }
        Change::None
    }

    fn tap_form(&mut self, x: f32, y: f32) -> Change {
        if self.confirming {
            let Some(action) = remove_sheet().hit(x, y) else {
                return Change::None;
            };
            self.confirming = false;
            if action == SheetAction::Confirm {
                let id = self
                    .form
                    .as_ref()
                    .and_then(|form| form.id)
                    .expect("existing habit");
                let mut next = self.state.data.clone();
                next.remove(id);
                if self.write(next, "Habit removed.") {
                    self.form = None;
                    self.detail = None;
                }
            }
            return Change::Full;
        }
        if self.choosing_target {
            if back_button(WIDTH as i32).contains(x, y) {
                self.choosing_target = false;
                return Change::Full;
            }
            if let Some(index) = (0..8).find(|i| target_bounds(*i).contains(x, y)) {
                if let Some(form) = &mut self.form {
                    form.target = target(index);
                }
                self.choosing_target = false;
                return Change::Full;
            }
            return Change::None;
        }
        if back_button(WIDTH as i32).contains(x, y) {
            self.form = None;
            self.notice.clear();
            return Change::Full;
        }
        if SAVE.contains(x, y) {
            return self.save_form();
        }
        let Some(form) = &mut self.form else {
            return Change::None;
        };
        if REMOVE.contains(x, y) && form.id.is_some() {
            self.confirming = true;
            return Change::Full;
        }
        if CHANGE_TARGET.contains(x, y) {
            self.choosing_target = true;
            return Change::Full;
        }
        if NAME.contains(x, y) && !form.typing {
            form.typing = true;
            return Change::Full;
        }
        if form.typing {
            match press_key(&mut form.name, x, y) {
                Some(Press::Go) => {
                    form.typing = false;
                    return Change::Full;
                }
                Some(Press::Nothing) | None => {}
                _ => return Change::Partial,
            }
        }
        Change::None
    }

    fn turn(&mut self, forward: bool) -> Change {
        if self.form.is_none()
            && self.detail.is_none()
            && page_step(&mut self.page, self.state.data.habits.len(), forward)
        {
            Change::Full
        } else {
            Change::None
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        if let Some(form) = &self.form {
            if self.choosing_target {
                header(canvas, "Choose target", "Back");
                for index in 0..8 {
                    let choice = target(index);
                    canvas.button(
                        target_bounds(index),
                        &choice.label(),
                        choice == form.target,
                        false,
                    );
                }
            } else {
                self.draw_form(canvas, form);
                if self.confirming {
                    remove_sheet().draw(canvas, None);
                }
            }
        } else if let Some((detail, habit)) = self.detail.as_ref().and_then(|detail| {
            self.state
                .data
                .habit(detail.id)
                .map(|habit| (detail, habit))
        }) {
            self.draw_detail(canvas, detail, habit);
        } else {
            self.draw_home(canvas);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        if self.live && !matches!(input, Input::Sleep) {
            let today = local_day();
            if today != self.today {
                if let Some(detail) = &mut self.detail {
                    if detail.end == self.today || detail.end > today {
                        detail.end = today;
                        detail.selected = today;
                    }
                }
                self.today = today;
                return Change::Full;
            }
        }
        match input {
            Input::Tap(x, y) if self.form.is_some() => self.tap_form(x, y),
            Input::Tap(x, y) if self.detail.is_some() => self.tap_detail(x, y),
            Input::Tap(x, y) => self.tap_home(x, y),
            Input::Swipe { dx, dy, .. } if dx.abs() > dy.abs() => self.turn(dx < 0.0),
            Input::PageBack => self.turn(false),
            Input::PageForward => self.turn(true),
            _ => Change::None,
        }
    }

    fn storage_returned(&mut self) -> io::Result<()> {
        self.state.reload()?;
        self.page = 0;
        self.detail = None;
        self.form = None;
        self.choosing_target = false;
        self.confirming = false;
        self.notice.clear();
        Ok(())
    }
}

fn days(end: NaiveDate) -> impl Iterator<Item = NaiveDate> {
    (0..7)
        .rev()
        .filter_map(move |offset| end.checked_sub_days(Days::new(offset)))
}

fn check_bounds(index: usize) -> Rect {
    rect(40, 383 + index as i32 * 121, 112, 108)
}
fn habit_bounds(index: usize) -> Rect {
    rect(166, 383 + index as i32 * 121, 1058, 108)
}
fn day_bounds(index: usize) -> Rect {
    rect(40 + index as i32 * 170, 461, 158, 104)
}
fn target(index: usize) -> Target {
    if index == 0 {
        Target::Daily
    } else {
        Target::Weekly(index as u8)
    }
}
fn target_bounds(index: usize) -> Rect {
    rect(
        40 + (index % 2) as i32 * 600,
        220 + (index / 2) as i32 * 126,
        584,
        110,
    )
}

fn remove_sheet() -> Sheet<'static> {
    Sheet::centered(
        rect(0, 0, 1264, 1680),
        "Remove habit?",
        "Its saved check-ins will also be removed.",
        "Remove",
    )
}
