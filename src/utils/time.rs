/*
amFOSS Daemon: A discord bot for the amFOSS Discord server.
Copyright (C) 2024 amFOSS

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use chrono::{DateTime, Datelike, Local, TimeZone};
use chrono_tz::Tz;
use tokio::time::Duration;

pub fn time_until(hour: u32, minute: u32) -> Duration {
    let now = chrono::Local::now().with_timezone(&chrono_tz::Asia::Kolkata);
    let today_run = now.date().and_hms(hour, minute, 0);

    let next_run = if now < today_run {
        today_run
    } else {
        today_run + chrono::Duration::days(1)
    };

    let time_until = (next_run - now).to_std().unwrap();
    Duration::from_secs(time_until.as_secs())
}

pub fn get_five_am_timestamp(now: DateTime<Tz>) -> DateTime<Local> {
    chrono::Local
        .ymd(now.year(), now.month(), now.day())
        .and_hms_opt(5, 0, 0)
        .expect("Chrono must work.")
}
