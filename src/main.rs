use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::io;
use std::process::Command;
use std::str;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Job {
    job_arn: String,
    job_id: String,
    job_name: String,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(with = "ts_seconds")]
    started_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(with = "ts_seconds")]
    stopped_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JobSummaryList {
    job_summary_list: Vec<Job>,
}

fn write_stdout<'a, T>(val: &'a T)
where
    T: Serialize,
{
    let results_file = io::stdout();
    serde_json::to_writer_pretty(&results_file, val).unwrap();
}

fn main() {
    let aws_res = Command::new("aws")
        .arg("batch")
        .arg("list-jobs")
        .arg("--job-queue")
        .arg("scheduler-stage")
        .arg("--job-status")
        .arg("Succeeded")
        .arg("--output")
        .arg("json")
        .output()
        .expect("failed to execute process")
        .stdout;

    // println!("{:?}", str::from_utf8(&aws_res).unwrap().trim());

    let job_summary_list: JobSummaryList = serde_json::from_slice(&aws_res).unwrap();

    let jobs: Vec<&Job> = job_summary_list
        .job_summary_list
        .iter()
        .filter(|job| job.started_at.timestamp() != 0)
        .collect();

    write_stdout(&jobs);
}
