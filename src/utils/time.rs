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
use chrono::{Datelike, Local, TimeZone};
use chrono_tz::Asia::Kolkata;
use std::time::Duration;

pub fn time_until(hour: u32, minute: u32) -> Duration {
    let now = Local::now().with_timezone(&Kolkata);
    let today_run = Kolkata
        .with_ymd_and_hms(now.year(), now.month(), now.day(), hour, minute, 0)
        .single()
        .expect("Valid datetime must be created");

    let next_run = if now < today_run {
        today_run
    } else {
        today_run + chrono::Duration::days(1)
    };

    let duration = next_run.signed_duration_since(now);
    Duration::from_secs(duration.num_seconds().max(0) as u64)
}
