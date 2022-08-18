use chrono::serde::ts_seconds;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::io;
use std::process::Command;
use std::str;

// fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let i: i64 = Deserialize::deserialize(deserializer)?;
//     let s: String = i.to_string();
//     // NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%f").map_err(de::Error::custom)
//     let a = NaiveDateTime::from_timestamp(i, 0);
//     Ok(a)
// }

// fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let i: i64 = Deserialize::deserialize(deserializer)?;
//     let s: String = i.to_string();
//     // NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%f").map_err(de::Error::custom)
//     let a = NaiveDateTime::from_timestamp(i, 0);
//     Ok(a)
// }

// fn naive_date_time_to_str<S: Serializer>(
//     time: &NaiveDateTime,
//     serializer: S,
// ) -> Result<S::Ok, S::Error> {
//     // time_to_json(time.clone()).serialize(serializer)
//     DateTime::<Utc>::from_utc(time.clone(), Utc)
//         .to_rfc3339()
//         .serialize(serializer)
// }

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Job {
    job_arn: String,
    job_id: String,
    job_name: String,
    // #[serde(deserialize_with = "date_time_from_str")]
    // #[serde(serialize_with = "naive_date_time_to_str")]
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
    started_at: Option<u64>,
    stopped_at: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JobSummaryList {
    job_summary_list: Vec<Job>,
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
        .filter(|job| job.started_at.is_some())
        .collect();

    let results_file = io::stdout();
    // serde_json::to_writer_pretty(&results_file, &job_summary_list).unwrap();
    serde_json::to_writer_pretty(&results_file, &jobs).unwrap();
}
