use std::collections::HashMap;

use anyhow::Result;
use compact_str::CompactString;
use serde::Deserialize;
use serde_aux::field_attributes::{deserialize_bool_from_anything, deserialize_number_from_string};

use crate::intranet_client::IntranetClient;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResData {
    _status: u8,
    data: Resources,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resources {
    courses: Vec<Course>,
    students: Vec<Student>,
    teachers: Vec<Teacher>,
    classes: Vec<Class>,
    rooms: Vec<Room>,
    resources: [(); 0],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Course {
    course: CompactString,
    course_id: u32,
    course_short: CompactString,
    course_label_or_course_description: String,
    course_short_with_classes: CompactString,
    period_id: u8,
    teacher_name: CompactString,
    // Really a vec but idk if deserialization would work
    student_name: String,
    subject_id: u8,
    teacher_id: Vec<u32>,
    student_id: Vec<u32>,
    class_id: Vec<u16>,
    class_name: Vec<CompactString>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Student {
    #[serde(alias = "personId", alias = "studentId")]
    id: u32,
    #[serde(alias = "studentName")]
    name: Name,
}

/// A name in the format firstname.lastname
#[derive(Debug, PartialEq)]
pub struct Name {
    pub string: CompactString,
}
impl<'de> Deserialize<'de> for Name {
    /// Deserialize from (Lastname, Firstname) to (firstname.lastname)
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str: String = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_lastname_firstname(&str))
    }
}
impl Name {
    /// From format (Lastname, Firstname)
    pub fn from_lastname_firstname(name: &str) -> Self {
        Self {
            string: name
                // Cant split by ", " because that wouldn't return a Double-Ended Iterator
                .split(',')
                .map(|name_part| name_part.trim_start().to_lowercase())
                .rev()
                .intersperse('.'.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Teacher {
    person_id: u32,
    name: CompactString,
    acronym: Option<CompactString>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    occupied: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Class {
    class_id: u16,
    class_name: CompactString,
    class_short: CompactString, // Duplicate of class_name
    class_common_name: CompactString,
    period_id: u8,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    class_level: u8,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    occupied: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Room {
    room_id: u16,
    room: CompactString,
    // Null
    description: Option<CompactString>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    occupied: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    sort1: u8,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    sort2: u8,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    room_category: u8,
    // Null
    building: Option<CompactString>,
}

impl Resources {
    pub fn get_student_id(&self, student_name: &Name) -> Option<u32> {
        self.students
            .iter()
            .find(|other_student| other_student.name == *student_name)
            .map(|student| student.id)
    }
}
impl<State> IntranetClient<State> {
    /// TODO: Test if it needs authentication
    /// TODO: Calculate the right periodId
    pub async fn get_resources(&self) -> Result<Resources> {
        // Get the csrf token from the classbook site, as the requests normally comes from there
        let csrf_token = self
            .get_csrf_token(&(format!("{}/timetable/classbook", self.school_url())))
            .await?;

        let mut resource_form = HashMap::new();
        resource_form.insert("periodId", "81");
        resource_form.insert("csrfToken", csrf_token.as_str());

        let resource_url = format!("{}/timetable/ajax-get-resources", self.school_url());
        let resource_response = self
            .client()
            .post(&resource_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&resource_form)
            .send()
            .await?;
        let resource_body = resource_response.text().await?;

        let resources = serde_json::from_str::<ResData>(&resource_body)?.data;
        Ok(resources)
    }
}
