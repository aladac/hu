use serde::{Deserialize, Serialize};

/// Human-friendly schedule options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schedule {
    /// Every hour at the same minute
    Hourly,
    /// Every day at the same time
    Daily,
    /// Every week on the same day and time
    Weekly,
    /// Every month on the same day and time
    Monthly,
    /// On system reboot
    Reboot,
}

impl Schedule {
    /// Parse a human-friendly schedule string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hourly" => Some(Self::Hourly),
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            "reboot" | "@reboot" => Some(Self::Reboot),
            _ => None,
        }
    }

    /// Convert to cron expression using base time + offset
    pub fn to_cron(self, minute: u32, hour: u32, day_of_month: u32, day_of_week: u32) -> String {
        match self {
            Self::Hourly => format!("{} * * * *", minute),
            Self::Daily => format!("{} {} * * *", minute, hour),
            Self::Weekly => format!("{} {} * * {}", minute, hour, day_of_week),
            Self::Monthly => format!("{} {} {} * *", minute, hour, day_of_month),
            Self::Reboot => "@reboot".to_string(),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Reboot => "reboot",
        }
    }
}

/// A cron job entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    /// The cron expression (e.g., "35 18 * * *")
    pub expression: String,
    /// The command to run
    pub command: String,
    /// Human-readable schedule (if parsed from hu)
    pub schedule_name: Option<String>,
    /// Whether this is a hu-managed job
    pub is_hu_job: bool,
}

impl CronJob {
    /// Check if this job matches a pattern (command contains pattern)
    pub fn matches(&self, pattern: &str) -> bool {
        self.command.contains(pattern)
    }

    /// Get human-readable time description from cron expression
    pub fn describe_time(&self) -> String {
        if self.expression == "@reboot" {
            return "on reboot".to_string();
        }

        let parts: Vec<&str> = self.expression.split_whitespace().collect();
        if parts.len() != 5 {
            return self.expression.clone();
        }

        let (min, hour, dom, _mon, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

        // Detect schedule type
        if hour == "*" && dom == "*" && dow == "*" {
            // Hourly
            format!(":{:0>2} every hour", min)
        } else if dom == "*" && dow == "*" {
            // Daily
            format!("{}:{:0>2} daily", hour, min)
        } else if dom == "*" && dow != "*" {
            // Weekly
            let day_name = match dow {
                "0" => "Sun",
                "1" => "Mon",
                "2" => "Tue",
                "3" => "Wed",
                "4" => "Thu",
                "5" => "Fri",
                "6" => "Sat",
                _ => dow,
            };
            format!("{}:{:0>2} every {}", hour, min, day_name)
        } else if dow == "*" {
            // Monthly
            let suffix = match dom {
                "1" | "21" | "31" => "st",
                "2" | "22" => "nd",
                "3" | "23" => "rd",
                _ => "th",
            };
            format!("{}:{:0>2} on {}{}", hour, min, dom, suffix)
        } else {
            self.expression.clone()
        }
    }
}

/// Marker comment for hu-managed cron jobs
pub const HU_MARKER: &str = "# hu:";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_parse_hourly() {
        assert_eq!(Schedule::parse("hourly"), Some(Schedule::Hourly));
        assert_eq!(Schedule::parse("HOURLY"), Some(Schedule::Hourly));
    }

    #[test]
    fn schedule_parse_daily() {
        assert_eq!(Schedule::parse("daily"), Some(Schedule::Daily));
    }

    #[test]
    fn schedule_parse_weekly() {
        assert_eq!(Schedule::parse("weekly"), Some(Schedule::Weekly));
    }

    #[test]
    fn schedule_parse_monthly() {
        assert_eq!(Schedule::parse("monthly"), Some(Schedule::Monthly));
    }

    #[test]
    fn schedule_parse_reboot() {
        assert_eq!(Schedule::parse("reboot"), Some(Schedule::Reboot));
        assert_eq!(Schedule::parse("@reboot"), Some(Schedule::Reboot));
    }

    #[test]
    fn schedule_parse_invalid() {
        assert_eq!(Schedule::parse("invalid"), None);
        assert_eq!(Schedule::parse(""), None);
    }

    #[test]
    fn schedule_to_cron_hourly() {
        let cron = Schedule::Hourly.to_cron(35, 18, 11, 2);
        assert_eq!(cron, "35 * * * *");
    }

    #[test]
    fn schedule_to_cron_daily() {
        let cron = Schedule::Daily.to_cron(35, 18, 11, 2);
        assert_eq!(cron, "35 18 * * *");
    }

    #[test]
    fn schedule_to_cron_weekly() {
        let cron = Schedule::Weekly.to_cron(35, 18, 11, 2);
        assert_eq!(cron, "35 18 * * 2");
    }

    #[test]
    fn schedule_to_cron_monthly() {
        let cron = Schedule::Monthly.to_cron(35, 18, 11, 2);
        assert_eq!(cron, "35 18 11 * *");
    }

    #[test]
    fn schedule_to_cron_reboot() {
        let cron = Schedule::Reboot.to_cron(35, 18, 11, 2);
        assert_eq!(cron, "@reboot");
    }

    #[test]
    fn schedule_display_name() {
        assert_eq!(Schedule::Hourly.display_name(), "hourly");
        assert_eq!(Schedule::Daily.display_name(), "daily");
        assert_eq!(Schedule::Weekly.display_name(), "weekly");
        assert_eq!(Schedule::Monthly.display_name(), "monthly");
        assert_eq!(Schedule::Reboot.display_name(), "reboot");
    }

    #[test]
    fn cron_job_matches() {
        let job = CronJob {
            expression: "35 18 * * *".to_string(),
            command: "hu gh sync ~/Projects/docs".to_string(),
            schedule_name: Some("daily".to_string()),
            is_hu_job: true,
        };
        assert!(job.matches("gh sync"));
        assert!(job.matches("docs"));
        assert!(!job.matches("nonexistent"));
    }

    #[test]
    fn cron_job_describe_time_hourly() {
        let job = CronJob {
            expression: "35 * * * *".to_string(),
            command: "test".to_string(),
            schedule_name: None,
            is_hu_job: false,
        };
        assert_eq!(job.describe_time(), ":35 every hour");
    }

    #[test]
    fn cron_job_describe_time_daily() {
        let job = CronJob {
            expression: "35 18 * * *".to_string(),
            command: "test".to_string(),
            schedule_name: None,
            is_hu_job: false,
        };
        assert_eq!(job.describe_time(), "18:35 daily");
    }

    #[test]
    fn cron_job_describe_time_weekly() {
        let job = CronJob {
            expression: "35 18 * * 2".to_string(),
            command: "test".to_string(),
            schedule_name: None,
            is_hu_job: false,
        };
        assert_eq!(job.describe_time(), "18:35 every Tue");
    }

    #[test]
    fn cron_job_describe_time_monthly() {
        let job = CronJob {
            expression: "35 18 11 * *".to_string(),
            command: "test".to_string(),
            schedule_name: None,
            is_hu_job: false,
        };
        assert_eq!(job.describe_time(), "18:35 on 11th");
    }

    #[test]
    fn cron_job_describe_time_reboot() {
        let job = CronJob {
            expression: "@reboot".to_string(),
            command: "test".to_string(),
            schedule_name: None,
            is_hu_job: false,
        };
        assert_eq!(job.describe_time(), "on reboot");
    }

    #[test]
    fn cron_job_describe_time_ordinal_suffixes() {
        let cases = [
            ("35 18 1 * *", "18:35 on 1st"),
            ("35 18 2 * *", "18:35 on 2nd"),
            ("35 18 3 * *", "18:35 on 3rd"),
            ("35 18 4 * *", "18:35 on 4th"),
            ("35 18 21 * *", "18:35 on 21st"),
            ("35 18 22 * *", "18:35 on 22nd"),
            ("35 18 23 * *", "18:35 on 23rd"),
        ];
        for (expr, expected) in cases {
            let job = CronJob {
                expression: expr.to_string(),
                command: "test".to_string(),
                schedule_name: None,
                is_hu_job: false,
            };
            assert_eq!(job.describe_time(), expected, "Failed for {}", expr);
        }
    }

    #[test]
    fn hu_marker_value() {
        assert_eq!(HU_MARKER, "# hu:");
    }
}
