//! The purpose of this module is to avoid calling `CoIncrementMTAUsage`,
//! which is *extremely* convenient, but does not exist on Windows 7.
//!
//! So, to the 7% of gamers still on Windows 7 - this one's for you.
use com::sys::FAILED;
use log;
use once_cell::sync::Lazy;
use std::{
    ffi::c_void,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CoInitializeEx failed with code {0}")]
    CoInitializeExFailed(HRESULT),
}

#[link(name = "ole32")]
extern "C" {
    fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> HRESULT;
    fn CoUninitialize();
}

// TODO: this is only here because cbindgen refuses to ignore
// this module and emits a function declaration for CoInitializeEx.
pub type HRESULT = i32;

enum Action {
    Increment,
    Decrement,
}
const COINIT_MULTITHREADED: u32 = 0x0;

type ComResult<T> = Result<T, Error>;

static SENDER_SLOT: Lazy<Mutex<Option<(Sender<Action>, Receiver<ComResult<()>>)>>> =
    Lazy::new(|| Mutex::new(None));
static ACTIVE: AtomicBool = AtomicBool::new(false);

fn call_com_thread(action: Action) -> ComResult<()> {
    let mut sender_slot = SENDER_SLOT.lock().unwrap();
    match sender_slot.as_ref() {
        None => {
            log::debug!("Spawning COM thread");
            let (action_tx, action_rx) = channel();
            let (result_tx, result_rx) = channel();

            std::thread::spawn(move || {
                let mut counter: i64 = 0;

                while let Ok(action) = action_rx.recv() {
                    match action {
                        Action::Increment => counter += 1,
                        Action::Decrement => counter -= 1,
                    }
                    let res = match counter {
                        0 => {
                            log::debug!("COM thread winding down (0 reached)");
                            unsafe {
                                CoUninitialize();
                            }
                            ACTIVE.store(false, Ordering::SeqCst);
                            Ok(())
                        }
                        1 => {
                            let ret = unsafe {
                                CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)
                            };
                            if FAILED(ret) {
                                log::debug!("COM initialization failed");
                                counter = 0;
                                Err(Error::CoInitializeExFailed(ret))
                            } else {
                                log::debug!("COM initialization succeeded");
                                ACTIVE.store(true, Ordering::SeqCst);
                                Ok(())
                            }
                        }
                        _ => Ok(()),
                    };
                    result_tx.send(res).unwrap();
                }
            });

            action_tx.send(action).unwrap();
            let res = result_rx.recv().unwrap();
            *sender_slot = Some((action_tx, result_rx));
            res
        }
        Some((action_tx, result_rx)) => {
            action_tx.send(action).unwrap();
            result_rx.recv().unwrap()
        }
    }
}

fn increment() -> ComResult<()> {
    call_com_thread(Action::Increment)
}

fn decrement() -> ComResult<()> {
    call_com_thread(Action::Decrement)
}

/// Returns true if COM is currently initialized.
#[cfg(test)]
pub(crate) fn is_active() -> bool {
    ACTIVE.load(Ordering::SeqCst)
}

pub struct ComGuard {
    dead: bool,
}

impl ComGuard {
    pub fn new() -> Result<Self, Error> {
        increment()?;
        Ok(Self { dead: false })
    }

    pub fn dispose(&mut self) -> Result<(), Error> {
        if self.dead {
            return Ok(());
        }
        self.dead = true;
        decrement()?;
        Ok(())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        // this swallows errors
        self.dispose().ok();
    }
}
