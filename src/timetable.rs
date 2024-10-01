use crate::{
    intranet_client::{Authenticated, IntranetClient},
    resources::Student,
    TIMETABLE_LAYOUT,
};
use anyhow::Result;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Deserializer};
use serde_aux::{
    field_attributes::{deserialize_bool_from_anything, deserialize_option_number_from_string},
    prelude::StringOrVecToVec,
};
use std::{collections::HashMap, str::FromStr, time::Duration};

pub enum TimeTableLayout {
    Centered,
    Week,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeTable {
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    id: u32,
    timetable_element_id: u32,
    holiday_id: u8,
    block_id: Vec<u32>,
    block_teacher_id: Vec<u32>,
    block_class_id: Vec<u16>,
    block_room_id: Vec<u8>,
    mod_id: u32,
    period_id: u8,
    #[serde(deserialize_with = "deserialize_date_time")]
    start: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_date_time")]
    end: DateTime<Utc>,
    lesson_date: NaiveDate,
    lesson_start: NaiveTime,
    lesson_end: NaiveTime,
    exam_modification: u8,
    lesson_duration: NaiveTime,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    nbr_of_modified_lessons: Option<u8>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    connected_id: Option<u32>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    is_all_day: bool,
    timetable_entry_type_id: u8,

    // Should probably be an enum
    timetable_entry_type: String,
    timetable_entry_type_long: String,
    timetable_entry_type_short: String,

    message_id: u32,
    message: Option<CompactString>,
    output: Option<CompactString>, // Null or ""
    title: CompactString,
    half_class_lesson: Option<CompactString>, // Null or "0"
    course_id: u32,
    course_name: Option<CompactString>,
    course: Option<CompactString>,
    course_long: CompactString, // Mostly empty
    is_exam_lesson: bool,
    is_checked_lesson: bool,
    lesson_absence_count: u8,
    subject_id: u8,
    subject_name: Option<CompactString>,
    timegrid_id: u8,
    class_id: Vec<u32>,
    #[serde(deserialize_with = "deserialize_vec_from_string_or_vec")]
    class_name: Vec<CompactString>, // Actually a Vec<String> (Seperator: ', ')
    #[serde(deserialize_with = "deserialize_vec_from_string_or_vec")]
    profile_id: Vec<u8>,
    team_id: CompactString, // Can be empty
    teacher_id: Vec<u32>,
    teacher_acronym: CompactString,
    teacher_full_name: Vec<CompactString>, // Can be empty
    teacher_lastname: CompactString,       // Can be empty
    teacher_firstname: CompactString,      // Can be empty
    connected_teacher_id: [(); 0],
    connected_teacher_full_name: [(); 0],
    student: Vec<Student>,
    student_id: [(); 0],
    student_full_name: CompactString, // Can be empty
    student_lastname: CompactString,  // Can be empty
    student_firstname: CompactString, // Can be empty
    room_id: Vec<u8>,
    room_name: CompactString,
    location_description: CompactString, // Can be empty
    resource_id: [(); 0],
    timetable_class_book_id: u16,
    has_homework: bool,
    has_homework_files: bool,
    has_exam: bool,
    has_exam_files: bool,
    privileges: Option<Vec<CompactString>>,
    resource: Option<()>,
    reserved_resources: u8,
    total_stock: u8,
    school: CompactString, // Can be empty
    related_id: Vec<CompactString>,
}

impl IntranetClient<Authenticated> {
    // Generic because of tests
    pub async fn get_timetable(&self, student_id: u32) -> Result<TimeTable> {
        let today = Local::now();

        let mut timetable_form = HashMap::new();
        match TIMETABLE_LAYOUT {
            TimeTableLayout::Week => {
                let monday =
                    today - Duration::from_days((today.weekday().num_days_from_monday()).into());
                let sunday = monday + Duration::from_days(6);

                timetable_form.insert("startDate", monday.timestamp_millis().to_string());
                timetable_form.insert("endDate", sunday.timestamp_millis().to_string());
            }
            TimeTableLayout::Centered => {
                timetable_form.insert(
                    "startDate",
                    (today - Duration::from_days(3))
                        .timestamp_millis()
                        .to_string(),
                );
                timetable_form.insert(
                    "endDate",
                    (today + Duration::from_days(3))
                        .timestamp_millis()
                        .to_string(),
                );
            }
        };
        timetable_form.insert("studentId[]", student_id.to_string());
        timetable_form.insert("holidaysOnly", "0".to_string());

        let timetable_url = format!("{}/timetable/ajax-get-timetable", self.school_url());
        let timetable_response = self
            .client()
            .post(&timetable_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&timetable_form)
            .send()
            .await?;
        let mut timetable_text = timetable_response.text().await?;

        // Safe because the original timetable text isnt used after this
        Ok(simd_json::from_slice(unsafe {
            timetable_text.as_bytes_mut()
        })?)
    }
}

/// Deserializes a DateTime in the format /Date(1727349600000.000000)/
fn deserialize_date_time<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;

    // The timestamp is in ms, so we convert it to seconds by not including the last three digits
    // in the slice
    let timestamp_str = &str[6..=15]; // Extract the integer part of the Date
    let timestamp_s: i64 = timestamp_str.parse().map_err(serde::de::Error::custom)?;

    DateTime::from_timestamp(timestamp_s, 0).ok_or(serde::de::Error::custom(
        "Failed to deserialize into DateTime",
    ))
}

pub fn deserialize_vec_from_string_or_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de> + 'static,
    <T as FromStr>::Err: std::fmt::Display,
{
    StringOrVecToVec::with_separator(", ").into_deserializer()(deserializer)
}
