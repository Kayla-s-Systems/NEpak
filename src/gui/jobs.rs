#![forbid(unsafe_code)]

#[cfg(feature = "gui")]
use crate::pak;

#[cfg(feature = "gui")]
use std::sync::mpsc;

#[cfg(feature = "gui")]
#[derive(Debug)]
pub enum JobMsg {
    Log(String),
    Done(Result<(), String>),
    ListDone(Result<Vec<pak::EntryInfo>, String>),
    VerifyDone(Result<(), String>),
}

#[cfg(feature = "gui")]
pub struct JobHandle {
    pub rx: mpsc::Receiver<JobMsg>,
}

#[cfg(feature = "gui")]
pub fn spawn_job<F>(f: F) -> JobHandle
where
    F: FnOnce(mpsc::Sender<JobMsg>) + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || f(tx));
    JobHandle { rx }
}