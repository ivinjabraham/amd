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
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Streak {
    #[serde(rename = "currentStreak")]
    pub current_streak: i32,
    #[serde(rename = "maxStreak")]
    pub max_streak: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Member {
    #[serde(rename = "memberId")]
    pub member_id: i32,
    pub name: String,
    #[serde(rename = "discordId")]
    pub discord_id: String,
    #[serde(rename = "groupId")]
    pub group_id: u32,
    #[serde(default)]
    pub streak: Vec<Streak>,
}
