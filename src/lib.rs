#![feature(duration_constructors)]
#![feature(iter_intersperse)]
#![feature(iterator_try_collect)]

use timetable::TimeTableLayout;

// TODO: Remove this const
const DEBUG: bool = true;
// TODO: Remove this const
const TIMETABLE_LAYOUT: TimeTableLayout = TimeTableLayout::Centered;
const CSRF_REGEX: &str = r"csrfToken\s*=\s*'([a-zA-Z0-9]+)'";

pub mod intranet_client;
pub mod resources;
pub mod timetable;
