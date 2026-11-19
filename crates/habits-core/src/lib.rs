//! Local check-ins keyed by a civil date.

use std::collections::BTreeSet;

use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Daily,
    Weekly(u8),
}

impl Target {
    pub fn label(self) -> String {
        match self {
            Self::Daily => "Daily".into(),
            Self::Weekly(1) => "1 day each week".into(),
            Self::Weekly(days) => format!("{days} days each week"),
        }
    }

    pub fn days(self) -> usize {
        match self {
            Self::Daily => 7,
            Self::Weekly(days) => usize::from(days),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Habit {
    pub id: u64,
    pub name: String,
    pub target: Target,
    pub dates: BTreeSet<NaiveDate>,
}

impl Habit {
    pub fn week_count(&self, today: NaiveDate) -> usize {
        let since_monday = u64::from(today.weekday().num_days_from_monday());
        let monday = today
            .checked_sub_days(Days::new(since_monday))
            .unwrap_or(today);
        self.dates.range(monday..=today).count()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Habits {
    pub habits: Vec<Habit>,
}

impl Habits {
    pub fn habit(&self, id: u64) -> Option<&Habit> {
        self.habits.iter().find(|habit| habit.id == id)
    }

    pub fn save_habit(
        &mut self,
        editing: Option<u64>,
        name: &str,
        target: Target,
    ) -> Result<(), &'static str> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
            return Err("Enter a name of 1 to 100 characters.");
        }
        if matches!(target, Target::Weekly(0 | 8..)) {
            return Err("Choose a weekly target of 1 to 7 days.");
        }
        if self
            .habits
            .iter()
            .any(|habit| Some(habit.id) != editing && habit.name.eq_ignore_ascii_case(name))
        {
            return Err("A habit with that name already exists.");
        }
        if let Some(id) = editing {
            let habit = self
                .habits
                .iter_mut()
                .find(|habit| habit.id == id)
                .ok_or("Habit no longer exists.")?;
            habit.name = name.to_owned();
            habit.target = target;
        } else {
            if self.habits.len() >= 100 {
                return Err("The habit limit has been reached.");
            }
            let id = self
                .habits
                .iter()
                .map(|habit| habit.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("The habit limit has been reached.")?;
            self.habits.push(Habit {
                id,
                name: name.to_owned(),
                target,
                dates: BTreeSet::new(),
            });
        }
        Ok(())
    }

    pub fn toggle(
        &mut self,
        id: u64,
        date: NaiveDate,
        today: NaiveDate,
    ) -> Result<bool, &'static str> {
        if date > today {
            return Err("Choose today or an earlier date.");
        }
        let habit = self
            .habits
            .iter_mut()
            .find(|habit| habit.id == id)
            .ok_or("Habit no longer exists.")?;
        if habit.dates.remove(&date) {
            return Ok(false);
        }
        habit.dates.insert(date);
        Ok(true)
    }

    pub fn remove(&mut self, id: u64) {
        self.habits.retain(|habit| habit.id != id);
    }

    pub fn csv(&self) -> String {
        let mut text = "habit,target,completed_date\n".to_owned();
        for habit in &self.habits {
            let name = csv_cell(&habit.name);
            let label = habit.target.label();
            for date in &habit.dates {
                text.push_str(&format!("{name},{label},{date}\n"));
            }
        }
        text
    }
}

fn csv_cell(value: &str) -> String {
    let protective_prefix = if value.starts_with(['=', '+', '-', '@']) {
        "'"
    } else {
        ""
    };
    format!("\"{protective_prefix}{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn one_log_per_day_can_be_corrected() {
        let mut habits = Habits::default();
        habits.save_habit(None, "Read", Target::Weekly(3)).unwrap();
        let today = day("2026-11-20");
        let yesterday = day("2026-11-19");
        assert_eq!(habits.toggle(1, yesterday, today), Ok(true));
        assert_eq!(habits.toggle(1, yesterday, today), Ok(false));
        assert!(habits.habit(1).unwrap().dates.is_empty());
        assert!(habits.toggle(1, day("2026-11-21"), today).is_err());
    }

    #[test]
    fn week_goal_resets_on_local_monday_and_export_escapes_quotes() {
        let mut habits = Habits::default();
        habits
            .save_habit(None, "Read, \"study\"", Target::Weekly(3))
            .unwrap();
        let monday = day("2026-11-23");
        habits.toggle(1, day("2026-11-22"), monday).unwrap();
        habits.toggle(1, monday, monday).unwrap();
        assert_eq!(habits.habit(1).unwrap().week_count(monday), 1);
        assert!(habits
            .csv()
            .contains("\"Read, \"\"study\"\"\",3 days each week,2026-11-23"));
    }
}
