// src/ui/helpers.rs

use chrono::{DateTime, Local, TimeZone};

/// Format Unix timestamp to readable date string
pub fn format_date(timestamp: i64) -> String {
    let datetime = Local.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%b %d, %Y").to_string()
}

/// Format Unix timestamp to relative time (e.g., "2 hours ago")
pub fn format_relative_time(timestamp: i64) -> String {
    let now = Local::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else {
        format_date(timestamp)
    }
}

/// Count words in text
pub fn count_words(text: &str) -> i32 {
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .count() as i32
}

/// Count characters (excluding whitespace)
pub fn count_chars(text: &str) -> i32 {
    text.chars().filter(|c| !c.is_whitespace()).count() as i32
}

/// Estimate reading time in minutes
pub fn estimate_reading_time(text: &str) -> i32 {
    let words = count_words(text);
    // Average reading speed: 200 words per minute
    ((words as f32) / 200.0).ceil() as i32
}

/// Truncate text with ellipsis
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
}

/// Get word count status message
pub fn word_count_status(count: i32, target: i32) -> String {
    if count == 0 {
        "Start writing...".to_string()
    } else if count < target {
        format!("{} / {}", count, target)
    } else if count == target {
        format!("Perfect! {} words", count)
    } else {
        format!("{} / {} ({})", count, target, count - target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("  hello   world  "), 2);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("one"), 1);
    }

    #[test]
    fn test_count_chars() {
        assert_eq!(count_chars("hello world"), 10);
        assert_eq!(count_chars("  hello  "), 5);
        assert_eq!(count_chars(""), 0);
    }

    #[test]
    fn test_estimate_reading_time() {
        assert_eq!(estimate_reading_time("word "), 1); // < 200 words = 1 min
        let text = "word ".repeat(300);
        assert_eq!(estimate_reading_time(&text), 2); // 300 words = 2 mins
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 5), "hello...");
    }
}
