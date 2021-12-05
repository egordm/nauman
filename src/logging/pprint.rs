use colored::*;
use crate::config::ExecutionPolicy;


const BANNER_CHAR: &str = "-";
const BANNER_H_PADDING: usize = 3;


/// Prints a banner with the given title.
pub fn flex_banner(text: impl ToString) -> String {
    let text = text.to_string();
    let width = text.len() + 2 + BANNER_H_PADDING * 2;
    let banner = BANNER_CHAR.repeat(width);
    let padding = BANNER_CHAR.repeat(BANNER_H_PADDING);

    format!("{0}\n{1} {2} {1}\n{0}", banner, padding, text)
}

pub fn command(text: &str) -> colored::ColoredString {
    format!("$ {}", text).cyan()
}

pub fn error(text: &str) -> colored::ColoredString {
    text.red()
}

pub fn task_error(name: &str, status: i32, duration: Option<&std::time::Duration>) -> colored::ColoredString {
    if let Some(duration) = duration {
        format!(
            "Task \"{name}\" completed in {duration}s with a non-zero exit status: {status}. This indicates a failure",
            name=name, status=status, duration=duration.as_secs()
        ).red()
    } else {
        format!(
            "Task \"{name}\" completed with a non-zero exit status: {status}. This indicates a failure",
            name=name, status=status
        ).red()
    }

}

pub fn task_success(name: &str, duration: Option<&std::time::Duration>) -> colored::ColoredString {
    if let Some(duration) = duration {
        format!(
            "Task \"{name}\" completed with a zero exit status in {duration}s. This indicates a success",
            name=name, duration=duration.as_secs()
        ).green()
    } else {
        format!(
            "Task \"{name}\" completed with a zero exit status. This indicates a success",
            name=name
        ).green()
    }
}

pub fn task_aborted(name: &str, policy: ExecutionPolicy) -> colored::ColoredString {
    match policy {
        ExecutionPolicy::NoPriorFailed => {
            format!(
                "Task \"{name}\" was aborted: Because one of the prior tasks failed, this task was not executed",
                name=name
            ).red()
        },
        ExecutionPolicy::PriorSuccess => {
            format!(
                "Task \"{name}\" was aborted: Because the prior task did not succeeded, this task was not executed",
                name=name
            ).red()
        },
        ExecutionPolicy::Always => {
            format!(
                "Task \"{name}\" was aborted for an unknown reason. This task was not executed",
                name=name
            ).red()
        },
    }
}