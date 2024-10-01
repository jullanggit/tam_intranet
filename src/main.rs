#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
use mimalloc::MiMalloc;
use std::fs;
use tam_intranet::intranet_client::{IntranetClient, School};

#[tokio::main]
async fn main() -> Result<()> {
    let password = fs::read_to_string("password")?;

    let intranet_client = IntranetClient::new(School::Mng, "julius.langhoff")?
        .authenticate(&password)
        .await?;

    let resources = intranet_client.get_resources().await?;

    let timetable = intranet_client
        .get_timetable(
            resources
                .get_student_id(&intranet_client.student)
                .expect("Student should exist"),
        )
        .await?;

    Ok(())
}
