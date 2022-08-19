use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
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

    let cold_start_times: Vec<i64> = jobs
        .iter()
        .map(|job| {
            let cold_start_time = job.started_at - job.created_at;
            // not sure why the heck I need to divide by 1k, but the num_seconds
            // is returning millis
            cold_start_time.num_seconds() / 1000
        })
        .collect();

    let run_times: Vec<i64> = jobs
        .iter()
        .map(|job| {
            let time = job.stopped_at - job.started_at;
            // not sure why the heck I need to divide by 1k, but the num_seconds
            // is returning millis
            time.num_seconds() / 1000
        })
        .collect();

    let total_time: Vec<i64> = cold_start_times
        .iter()
        .zip(run_times.iter())
        .map(|t| t.0 + t.1)
        .collect();

    println!("######### COLD START SECONDS #########");
    write_stdout(&cold_start_times);

    println!("######### RUN SECONDS #########");
    write_stdout(&run_times);

    println!("######### TOTAL SECONDS #########");
    write_stdout(&total_time);

    //     let container_overrides = "{

    // }";
    // let submit_job_res = Command::new("aws")
    //     .arg("batch")
    //     .arg("submit-job")
    //     .arg("job-name")
    //     .arg("load-testing-cold-start")
    //     .arg("--job-queue")
    //     .arg("scheduler-stage")
    //     .arg("--job-definition")
    //     .arg("arn:aws:batch:us-east-1:663148821630:job-definition/run-scheduler-stage:7")
    //     .arg("--timeout")
    //     .arg("180")
    //     .arg("--container-overrides")
    //     .arg("")
    //     .output()
    //     .expect("failed to execute process")
    //     .stdout;
}
