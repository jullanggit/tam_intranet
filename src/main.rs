#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
use mimalloc::MiMalloc;
use std::fs;
use tam_intranet::intranet_client::{IntranetClient, School};

fn main() -> Result<()> {
    let credentials_str = fs::read_to_string("credentials")?;
    let credentials: Vec<_> = credentials_str.split_whitespace().collect();
    let username = credentials[0];
    let password = credentials[1];

    let intranet_client =
        IntranetClient::new(School::UetikonAmSee, username)?.authenticate(password)?;

    let resources = intranet_client.get_resources()?;

    let timetable = intranet_client.get_timetable(
        resources
            .get_student_id(&intranet_client.student)
            .expect("Student should exist"),
    )?;

    Ok(())
}
