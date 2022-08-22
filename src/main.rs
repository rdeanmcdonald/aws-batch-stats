use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use clap::Parser;
use futures::future::join_all;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::io;
use std::process::Command;
use std::str;
use std::thread::sleep;
use tokio::spawn;
use tokio::task::JoinHandle;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Outputs cold_start time to the first column
    #[clap(short, long, value_parser)]
    load_test: bool,

    /// Outputs cold_start time to the first column
    #[clap(short, long, value_parser)]
    cold: bool,

    /// Outputs run_time time to the next column
    #[clap(short, long, value_parser)]
    run: bool,

    /// Outputs total_time time to the final column
    #[clap(short, long, value_parser)]
    total: bool,

    /// Outputs | cold run total | times
    #[clap(short, long, value_parser)]
    all: bool,
}

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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.load_test {
        let submitter_handles: Vec<JoinHandle<()>> = (0..10)
            .into_iter()
            .map(|_x| {
                sleep(std::time::Duration::from_secs(5));
                spawn(submit_job_group())
            })
            .collect();

        let _results = join_all(submitter_handles).await;
    } else {
        let stage = "stage";

        let aws_res = Command::new("aws")
            .arg("batch")
            .arg("list-jobs")
            .arg("--job-queue")
            .arg(format!("scheduler-{}", stage))
            .arg("--job-status")
            .arg("Succeeded")
            .arg("--output")
            .arg("json")
            .output()
            .expect("failed to execute process")
            .stdout;

        let job_summary_list: JobSummaryList = serde_json::from_slice(&aws_res).unwrap();

        let jobs: Vec<&Job> = job_summary_list
            .job_summary_list
            .iter()
            .filter(|job| job.started_at.timestamp() != 0)
            .collect();

        let times: Vec<(i64, i64, i64)> = jobs
            .iter()
            .map(|job| {
                let cold_time = (job.started_at - job.created_at);
                let run_time = (job.stopped_at - job.started_at);
                let total_time = cold_time + run_time;
                (
                    cold_time.num_seconds() / 1000,
                    run_time.num_seconds() / 1000,
                    total_time.num_seconds() / 1000,
                )
            })
            .collect();

        match (&args.all, &args.cold, &args.run, &args.total) {
            (true, _, _, _) => {
                println!("{0: <10} {1: <10} {2: <10}", "cold", "run", "time");
                for time in times {
                    println!("{0: <10} {1: <10} {2: <10}", time.0, time.1, time.2);
                }
            }
            (false, disp_cold, disp_run, disp_total) => {
                for time in times {
                    let cold_str = if *disp_cold {
                        time.0.to_string() + "s"
                    } else {
                        "".to_string()
                    };
                    let run_str = if *disp_run {
                        time.1.to_string() + "s"
                    } else {
                        "".to_string()
                    };
                    let total_str = if *disp_total {
                        time.2.to_string() + "s"
                    } else {
                        "".to_string()
                    };

                    let line = format!("{} {} {}", cold_str, run_str, total_str);
                    println!("{}", line.trim());
                }
            }
        }
    }
}

async fn submit_job_group() {
    let submitter_handles: Vec<JoinHandle<()>> = (0..5)
        .into_iter()
        .map(|x| {
            let container_overrides = r#"
{
    "environment": [
        { "name": "STAGE", "value": "stage" },
        { "name": "CORR_ID", "value": "42c6d830-1f17-11ed-8dc3-cbe6144980fb" },
        { "name": "java_opts", "value": "" }
    ],
    "resourceRequirements": [
        { "type": "MEMORY", "value": "2048" },
        { "type": "VCPU", "value": "1" }
    ],
    "command": ["-J-XX:+UseContainerSupport","-J-XX:InitialRAMPercentage=80.0","-J-XX:MaxRAMPercentage=80.0","-main","io.wisesystems.engine.planning.Main","02083785-1978-454f-8b97-2ce66cb0a86d/2022/08/18/1660842028339.json","lrsStart"]
}
"#;
            spawn(async move {
                println!("Submitting {}", x);
                Command::new("aws")
                    .arg("batch")
                    .arg("submit-job")
                    .arg("--job-name")
                    .arg("load-testing-cold-start")
                    .arg("--job-queue")
                    .arg("scheduler-stage")
                    .arg("--job-definition")
                    .arg("arn:aws:batch:us-east-1:663148821630:job-definition/run-scheduler-stage:8")
                    .arg("--container-overrides")
                    .arg(container_overrides)
                    .spawn()
                    .expect("Failed to execute command");
            })
        })
        .collect();

    let results = join_all(submitter_handles).await;
}
