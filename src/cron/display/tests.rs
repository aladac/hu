use super::*;

#[test]
fn format_jobs_empty() {
    let jobs: Vec<CronJob> = vec![];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("No cron jobs"));
}

#[test]
fn format_jobs_single() {
    let jobs = vec![CronJob {
        expression: "35 18 * * *".to_string(),
        command: "echo hello".to_string(),
        schedule_name: Some("daily".to_string()),
        is_hu_job: true,
    }];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("daily"));
    assert!(output.contains("echo hello"));
    assert!(output.contains("hu"));
}

#[test]
fn format_jobs_multiple() {
    let jobs = vec![
        CronJob {
            expression: "0 * * * *".to_string(),
            command: "job1".to_string(),
            schedule_name: Some("hourly".to_string()),
            is_hu_job: true,
        },
        CronJob {
            expression: "30 12 * * *".to_string(),
            command: "job2".to_string(),
            schedule_name: None,
            is_hu_job: false,
        },
    ];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("hourly"));
    assert!(output.contains("job1"));
    assert!(output.contains("job2"));
}

#[test]
fn format_jobs_json() {
    let jobs = vec![CronJob {
        expression: "35 18 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: Some("daily".to_string()),
        is_hu_job: true,
    }];
    let output = format_jobs(&jobs, true);
    assert!(output.contains("\"expression\""));
    assert!(output.contains("\"is_hu_job\": true"));
}

#[test]
fn format_jobs_json_empty() {
    let jobs: Vec<CronJob> = vec![];
    let output = format_jobs(&jobs, true);
    assert_eq!(output, "[]");
}

#[test]
fn format_added_basic() {
    let job = CronJob {
        expression: "35 18 * * *".to_string(),
        command: "hu gh sync ~/docs".to_string(),
        schedule_name: Some("daily".to_string()),
        is_hu_job: true,
    };
    let output = format_added(&job, false);
    assert!(output.contains("\u{2713}")); // checkmark
    assert!(output.contains("daily"));
    assert!(output.contains("35 18 * * *"));
}

#[test]
fn format_added_json() {
    let job = CronJob {
        expression: "35 18 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: Some("daily".to_string()),
        is_hu_job: true,
    };
    let output = format_added(&job, true);
    assert!(output.contains("\"expression\""));
    assert!(output.contains("35 18 * * *"));
}

#[test]
fn format_removed_empty() {
    let jobs: Vec<CronJob> = vec![];
    let output = format_removed(&jobs, false);
    assert!(output.contains("No matching jobs"));
}

#[test]
fn format_removed_single() {
    let jobs = vec![CronJob {
        expression: "35 18 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: None,
        is_hu_job: false,
    }];
    let output = format_removed(&jobs, false);
    assert!(output.contains("Removed 1 job"));
    assert!(!output.contains("jobs:")); // singular
}

#[test]
fn format_removed_multiple() {
    let jobs = vec![
        CronJob {
            expression: "0 * * * *".to_string(),
            command: "job1".to_string(),
            schedule_name: None,
            is_hu_job: false,
        },
        CronJob {
            expression: "30 12 * * *".to_string(),
            command: "job2".to_string(),
            schedule_name: None,
            is_hu_job: false,
        },
    ];
    let output = format_removed(&jobs, false);
    assert!(output.contains("Removed 2 jobs"));
    assert!(output.contains("job1"));
    assert!(output.contains("job2"));
}

#[test]
fn format_removed_json() {
    let jobs = vec![CronJob {
        expression: "35 18 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: None,
        is_hu_job: false,
    }];
    let output = format_removed(&jobs, true);
    assert!(output.contains("\"expression\""));
}

#[test]
fn truncate_command_short() {
    let cmd = "echo hello";
    assert_eq!(truncate_command(cmd, 50), "echo hello");
}

#[test]
fn truncate_command_long() {
    let cmd = "this is a very long command that should be truncated";
    let truncated = truncate_command(cmd, 20);
    assert!(truncated.ends_with("..."));
    assert_eq!(truncated.len(), 20);
}

#[test]
fn truncate_command_exact() {
    let cmd = "exact";
    assert_eq!(truncate_command(cmd, 5), "exact");
}

#[test]
fn format_jobs_no_schedule_name() {
    let jobs = vec![CronJob {
        expression: "0 0 * * *".to_string(),
        command: "midnight job".to_string(),
        schedule_name: None,
        is_hu_job: false,
    }];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("-")); // dash for no schedule name
    assert!(output.contains("midnight job"));
}

#[test]
fn format_jobs_table_has_headers() {
    let jobs = vec![CronJob {
        expression: "0 0 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: Some("daily".to_string()),
        is_hu_job: true,
    }];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("Schedule"));
    assert!(output.contains("Time"));
    assert!(output.contains("Command"));
}

#[test]
fn format_added_no_schedule_name() {
    let job = CronJob {
        expression: "0 0 * * *".to_string(),
        command: "test".to_string(),
        schedule_name: None,
        is_hu_job: false,
    };
    let output = format_added(&job, false);
    assert!(output.contains("cron job")); // fallback
}

#[test]
fn format_jobs_long_command_truncated() {
    let jobs = vec![CronJob {
        expression: "0 0 * * *".to_string(),
        command: "this is an extremely long command that should definitely be truncated in the table display".to_string(),
        schedule_name: None,
        is_hu_job: false,
    }];
    let output = format_jobs(&jobs, false);
    assert!(output.contains("..."));
}
