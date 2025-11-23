use chrono::NaiveDate;
use rusqlite::Connection;

use crate::db::{self, WorkSession};

pub fn last_week_chart(conn: &Connection) -> Vec<(NaiveDate, Vec<(i32, String, f32)>)> {
    let mut day = chrono::Utc::now().date_naive();
    let mut stats = vec![];
    for i in 0..7 {
        stats.push((
            day,
            //((7 - i) - 7) as f32,
            db::get_work_hours_for_day(conn, &day).unwrap(),
        ));
        day = day.pred_opt().unwrap();
    }
    stats
}
