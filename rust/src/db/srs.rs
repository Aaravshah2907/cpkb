//! SuperMemo SM-2 Spaced Repetition calculation and revision engine.

use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewRecord {
    pub snippet_id: String,
    pub ease_factor: f64,
    pub interval: u32,
    pub repetitions: u32,
    pub next_review: String,
    pub last_reviewed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SrsStats {
    pub total: u32,
    pub reviewed: u32,
    pub unreviewed: u32,
    pub due: u32,
    pub avg_ease_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevisionSnippet {
    pub id: String,
    pub title: String,
    pub description: String,
    pub code: String,
}

/// Update the SRS review schedule using the SuperMemo SM-2 algorithm.
pub fn upsert_review(
    conn: &mut Connection,
    snippet_id: &str,
    quality: u8,
) -> Result<ReviewRecord, rusqlite::Error> {
    let q = quality.min(5) as i32;

    let existing: Option<(f64, u32, u32)> = conn
        .query_row(
            "SELECT ease_factor, interval, repetitions FROM reviews WHERE snippet_id = ?1",
            params![snippet_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    let (mut ef, mut interval, mut reps) = existing.unwrap_or((2.5, 0, 0));

    // SM-2 algorithm
    if q >= 3 {
        if reps == 0 {
            interval = 1;
        } else if reps == 1 {
            interval = 6;
        } else {
            interval = (interval as f64 * ef).round() as u32;
        }
        reps += 1;
    } else {
        reps = 0;
        interval = 1;
    }

    // Update ease factor (minimum 1.3)
    let q_diff = 5.0 - q as f64;
    ef = ef + (0.1 - q_diff * (0.08 + q_diff * 0.02));
    if ef < 1.3 {
        ef = 1.3;
    }

    let now = Utc::now();
    let next_review = (now + Duration::days(interval as i64)).to_rfc3339();
    let last_reviewed = now.to_rfc3339();

    conn.execute(
        "INSERT INTO reviews (snippet_id, ease_factor, interval, repetitions, next_review, last_reviewed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(snippet_id) DO UPDATE SET
             ease_factor = excluded.ease_factor,
             interval = excluded.interval,
             repetitions = excluded.repetitions,
             next_review = excluded.next_review,
             last_reviewed = excluded.last_reviewed",
        params![snippet_id, ef, interval, reps, next_review, last_reviewed],
    )?;

    Ok(ReviewRecord {
        snippet_id: snippet_id.to_string(),
        ease_factor: ef,
        interval,
        repetitions: reps,
        next_review,
        last_reviewed,
    })
}

/// Fetch the most overdue snippet due for revision.
pub fn get_due_snippet(conn: &Connection) -> Result<Option<RevisionSnippet>> {
    let now = Utc::now().to_rfc3339();
    conn.query_row(
        "SELECT s.id, s.title, s.description, s.code
         FROM snippets s
         JOIN reviews r ON s.id = r.snippet_id
         WHERE r.next_review <= ?1
         ORDER BY r.next_review ASC
         LIMIT 1",
        params![now],
        |row| {
            Ok(RevisionSnippet {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                code: row.get(3)?,
            })
        },
    )
    .optional()
}

/// Fetch an unreviewed snippet that has no review schedule yet.
pub fn get_unreviewed_snippet(conn: &Connection) -> Result<Option<RevisionSnippet>> {
    conn.query_row(
        "SELECT s.id, s.title, s.description, s.code
         FROM snippets s
         LEFT JOIN reviews r ON s.id = r.snippet_id
         WHERE r.snippet_id IS NULL
         ORDER BY RANDOM()
         LIMIT 1",
        [],
        |row| {
            Ok(RevisionSnippet {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                code: row.get(3)?,
            })
        },
    )
    .optional()
}

/// Fetch a random snippet for review fallback.
pub fn get_random_snippet(conn: &Connection) -> Result<Option<RevisionSnippet>> {
    conn.query_row(
        "SELECT id, title, description, code FROM snippets ORDER BY RANDOM() LIMIT 1",
        [],
        |row| {
            Ok(RevisionSnippet {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                code: row.get(3)?,
            })
        },
    )
    .optional()
}

/// Compute summary statistics for Spaced Repetition system.
pub fn get_srs_stats(conn: &Connection) -> Result<SrsStats> {
    let now = Utc::now().to_rfc3339();
    let total: u32 = conn.query_row("SELECT COUNT(*) FROM snippets", [], |r| r.get(0))?;
    let reviewed: u32 = conn.query_row("SELECT COUNT(*) FROM reviews", [], |r| r.get(0))?;
    let unreviewed = total.saturating_sub(reviewed);
    let due: u32 = conn.query_row(
        "SELECT COUNT(*) FROM reviews WHERE next_review <= ?1",
        params![now],
        |r| r.get(0),
    )?;
    let avg_ef: Option<f64> = conn
        .query_row("SELECT AVG(ease_factor) FROM reviews", [], |r| r.get(0))
        .optional()?;

    Ok(SrsStats {
        total,
        reviewed,
        unreviewed,
        due,
        avg_ease_factor: avg_ef.unwrap_or(2.5),
    })
}
