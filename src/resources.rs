use std::collections::HashMap;

use anyhow::Result;
use compact_str::CompactString;
use serde::Deserialize;
use serde_aux::field_attributes::{deserialize_bool_from_anything, deserialize_number_from_string};
use simd_json::prelude::ArrayTrait;

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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Student {
    person_id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Teacher {
    person_id: u32,
    name: String,
    acronym: Option<String>,
    // Really a bool (probably)
    occupied: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Class {
    class_id: u16,
    class_name: CompactString,
    class_short: CompactString, // Duplicate of class_name
    class_common_name: CompactString,
    period_id: u8,
    // Really an integer (u8)
    class_level: String,
    // Really a bool (probably)
    occupied: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Room {
    room_id: u16,
    room: CompactString,
    // Null
    // Really a bool (probably)
    occupied: u8,
    // Really an integer (u8)
    sort1: String,
    // Really an integer (u8)
    sort2: String,
    // Really an integer (u8)
    room_category: String,
    description: Option<CompactString>,
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
        let mut resource_body = resource_response.text().await?;

        // Safe because the original resource text isnt used after this
        let resources =
            simd_json::from_slice::<ResData>(unsafe { resource_body.as_bytes_mut() })?.data;
        Ok(resources)
    }
}
