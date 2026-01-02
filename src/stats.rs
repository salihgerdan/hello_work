use chrono::{Local, NaiveDate, TimeDelta};
use rusqlite::Connection;

use crate::db;

pub fn last_week_chart(conn: &Connection, config_offset_hours: u32) -> Vec<(NaiveDate, f32)> {
    //let offset = FixedOffset::east_opt((config_offset_hours * 60 * 60) % (24 * 60 * 60)).unwrap();

    let mut day = Local::now()
        .checked_sub_signed(TimeDelta::hours((config_offset_hours % 24) as i64))
        .unwrap()
        .date_naive();

    let mut stats = vec![];
    for _ in 0..7 {
        stats.push((
            day,
            db::get_work_hours_for_day(conn, &day, config_offset_hours).unwrap(),
        ));
        day = day.pred_opt().unwrap();
    }
    stats
}
