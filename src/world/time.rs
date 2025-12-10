use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,      // 5-7
    Morning,   // 7-11
    Noon,      // 11-14
    Afternoon, // 14-17
    Dusk,      // 17-19
    Evening,   // 19-22
    Night,     // 22-2
    Midnight,  // 2-5
}

impl TimeOfDay {
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            5..=6 => TimeOfDay::Dawn,
            7..=10 => TimeOfDay::Morning,
            11..=13 => TimeOfDay::Noon,
            14..=16 => TimeOfDay::Afternoon,
            17..=18 => TimeOfDay::Dusk,
            19..=21 => TimeOfDay::Evening,
            22..=23 | 0..=1 => TimeOfDay::Night,
            2..=4 => TimeOfDay::Midnight,
            _ => TimeOfDay::Noon,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "dawn",
            TimeOfDay::Morning => "morning",
            TimeOfDay::Noon => "noon",
            TimeOfDay::Afternoon => "afternoon",
            TimeOfDay::Dusk => "dusk",
            TimeOfDay::Evening => "evening",
            TimeOfDay::Night => "night",
            TimeOfDay::Midnight => "midnight",
        }
    }

    pub fn light_level(&self) -> f32 {
        match self {
            TimeOfDay::Dawn => 0.4,
            TimeOfDay::Morning => 0.8,
            TimeOfDay::Noon => 1.0,
            TimeOfDay::Afternoon => 0.9,
            TimeOfDay::Dusk => 0.5,
            TimeOfDay::Evening => 0.2,
            TimeOfDay::Night => 0.1,
            TimeOfDay::Midnight => 0.05,
        }
    }

    pub fn temperature_modifier(&self) -> f32 {
        match self {
            TimeOfDay::Dawn => -3.0,
            TimeOfDay::Morning => 0.0,
            TimeOfDay::Noon => 5.0,
            TimeOfDay::Afternoon => 3.0,
            TimeOfDay::Dusk => -1.0,
            TimeOfDay::Evening => -4.0,
            TimeOfDay::Night => -6.0,
            TimeOfDay::Midnight => -8.0,
        }
    }

    /// Can aurora be visible?
    pub fn aurora_visible(&self) -> bool {
        matches!(self, TimeOfDay::Evening | TimeOfDay::Night | TimeOfDay::Midnight | TimeOfDay::Dawn)
    }

    /// Sunrise visible from north (towards mountains)?
    pub fn sunrise_visible(&self) -> bool {
        matches!(self, TimeOfDay::Dawn | TimeOfDay::Morning)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTime {
    pub day: u32,
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub tick: u64,     // Total simulation ticks
}

impl WorldTime {
    pub fn new() -> Self {
        Self {
            day: 1,
            hour: 8,  // Start at 8 AM
            minute: 0,
            tick: 0,
        }
    }

    pub fn advance(&mut self, minutes: u32) {
        let total_minutes = self.minute as u32 + minutes;
        let additional_hours = total_minutes / 60;
        self.minute = (total_minutes % 60) as u8;

        let total_hours = self.hour as u32 + additional_hours;
        let additional_days = total_hours / 24;
        self.hour = (total_hours % 24) as u8;

        self.day += additional_days;
        self.tick += 1;
    }

    pub fn advance_tick(&mut self) {
        // Each tick is roughly 10 minutes
        self.advance(10);
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        TimeOfDay::from_hour(self.hour)
    }

    pub fn formatted_time(&self) -> String {
        format!("Day {} {:02}:{:02}", self.day, self.hour, self.minute)
    }

    pub fn time_description(&self) -> String {
        let tod = self.time_of_day();
        let period = if self.hour < 12 { "AM" } else { "PM" };
        let display_hour = if self.hour == 0 { 12 }
            else if self.hour > 12 { self.hour - 12 }
            else { self.hour };

        format!("{} ({}:{:02} {})", tod.name(), display_hour, self.minute, period)
    }
}

impl Default for WorldTime {
    fn default() -> Self {
        Self::new()
    }
}
