use std::fs;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
use tam_intranet::{
    intranet_client::{IntranetClient, School},
    timetable::TimeTable,
};

#[tokio::main]
async fn main() -> Result<()> {
    let password = fs::read_to_string("password")?;

    let intranet_client = IntranetClient::new(School::Mng, "julius.langhoff")?
        .authenticate(&password)
        .await?;

    let resources = intranet_client.get_resources().await?;

    let timetable: TimeTable = intranet_client
        .get_timetable(resources.get_student_id(intranet_client.student()))
        .await?;

    Ok(())
}
