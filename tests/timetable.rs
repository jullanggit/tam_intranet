use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use tam_intranet::{
    intranet_client::{IntranetClient, School},
    timetable::{TimeTable, TimeTableTrait},
};

#[derive(Deserialize)]
struct TimeTableTest {
    #[serde(flatten)]
    timetable: TimeTable,
    #[serde(flatten)]
    extra_fields: HashMap<String, simd_json::owned::Value>,
}
impl TimeTableTrait for TimeTableTest {}

#[tokio::test]
async fn ensure_api_covered() -> Result<()> {
    let password = fs::read_to_string("password")?;

    let intranet_client = IntranetClient::new(School::Mng, "julius.langhoff")?
        .authenticate(&password)
        .await?;

    let resources = intranet_client.get_resources().await?;

    let timetable: TimeTableTest = intranet_client
        .get_timetable(resources.get_student_id(intranet_client.student()))
        .await?;

    assert!(timetable.extra_fields.is_empty());

    Ok(())
}
