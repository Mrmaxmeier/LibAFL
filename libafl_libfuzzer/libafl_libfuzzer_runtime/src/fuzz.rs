use core::ffi::c_int;
use std::{
    fmt::Debug,
    fs::File,
    net::TcpListener,
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(unix)]
use std::{
    io::Write,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd},
};

use libafl::{
    bolts::{
        core_affinity::Cores,
        launcher::Launcher,
        shmem::{ShMemProvider, StdShMemProvider},
    },
    corpus::Corpus,
    events::{EventConfig, ProgressReporter, SimpleEventManager, SimpleRestartingEventManager},
    executors::ExitKind,
    inputs::UsesInput,
    monitors::{
        tui::{ui::TuiUI, TuiMonitor},
        Monitor, MultiMonitor, SimpleMonitor,
    },
    stages::StagesTuple,
    state::{HasClientPerfMonitor, HasExecutions, HasMetadata, HasSolutions, UsesState},
    Error, Fuzzer,
};

use crate::{feedbacks::LibfuzzerCrashCauseMetadata, fuzz_with, options::LibfuzzerOptions};

fn do_fuzz<F, ST, E, S, EM>(
    options: &LibfuzzerOptions,
    fuzzer: &mut F,
    stages: &mut ST,
    executor: &mut E,
    state: &mut S,
    mgr: &mut EM,
) -> Result<(), Error>
where
    F: Fuzzer<E, EM, ST, State = S>,
    S: HasClientPerfMonitor + HasMetadata + HasExecutions + UsesInput + HasSolutions,
    E: UsesState<State = S>,
    EM: ProgressReporter<State = S>,
    ST: StagesTuple<E, EM, S, F>,
{
    if let Some(solution) = state.solutions().last() {
        let kind = state
            .solutions()
            .get(solution)
            .expect("Last solution was not available")
            .borrow()
            .metadata::<LibfuzzerCrashCauseMetadata>()
            .expect("Crash cause not attached to solution")
            .kind();
        let mut halt = false;
        match kind {
            ExitKind::Oom if !options.ignore_ooms() => halt = true,
            ExitKind::Crash if !options.ignore_crashes() => halt = true,
            ExitKind::Timeout if !options.ignore_timeouts() => halt = true,
            _ => {
                log::info!("Ignoring {kind:?} according to requested ignore rules.");
            }
        }
        if halt {
            log::info!("Halting; the error on the next line is actually okay. :)");
            return Err(Error::shutting_down());
        }
    }
    fuzzer.fuzz_loop(stages, executor, state, mgr)?;
    Ok(())
}

fn fuzz_single_forking<M>(
    options: LibfuzzerOptions,
    harness: &extern "C" fn(*const u8, usize) -> c_int,
    mut shmem_provider: StdShMemProvider,
    monitor: M,
) -> Result<(), Error>
where
    M: Monitor + Debug,
{
    fuzz_with!(options, harness, do_fuzz, |fuzz_single| {
        let (state, mgr): (
            Option<StdState<_, _, _, _>>,
            SimpleRestartingEventManager<_, StdState<_, _, _, _>, _>,
        ) = match SimpleRestartingEventManager::launch(monitor, &mut shmem_provider) {
            // The restarting state will spawn the same process again as child, then restarted it each time it crashes.
            Ok(res) => res,
            Err(err) => match err {
                Error::ShuttingDown => {
                    return Ok(());
                }
                _ => {
                    panic!("Failed to setup the restarter: {err}");
                }
            },
        };
        #[cfg(unix)]
        {
            if options.close_fd_mask() != 0 {
                let file_null = File::open("/dev/null")?;
                unsafe {
                    if options.close_fd_mask() & 1 != 0 {
                        libc::dup2(file_null.as_raw_fd().into(), 1);
                    }
                    if options.close_fd_mask() & 2 != 0 {
                        libc::dup2(file_null.as_raw_fd().into(), 2);
                    }
                }
            }
        }
        crate::start_fuzzing_single(fuzz_single, state, mgr)
    })
}

fn fuzz_many_forking<M>(
    options: LibfuzzerOptions,
    harness: &extern "C" fn(*const u8, usize) -> c_int,
    shmem_provider: StdShMemProvider,
    forks: usize,
    monitor: M,
) -> Result<(), Error>
where
    M: Monitor + Clone + Debug,
{
    fuzz_with!(options, harness, do_fuzz, |mut run_client| {
        let cores = Cores::from((0..forks).collect::<Vec<_>>());
        let broker_port = TcpListener::bind("127.0.0.1:0")?
            .local_addr()
            .unwrap()
            .port();

        match Launcher::builder()
            .shmem_provider(shmem_provider)
            .configuration(EventConfig::from_name(options.fuzzer_name()))
            .monitor(monitor)
            .run_client(&mut run_client)
            .cores(&cores)
            .broker_port(broker_port)
            // TODO .remote_broker_addr(opt.remote_broker_addr)
            .stdout_file(Some("/dev/null"))
            .build()
            .launch()
        {
            Ok(()) => (),
            Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
            res @ Err(_) => return res,
        }
        Ok(())
    })
}

pub fn fuzz(
    options: LibfuzzerOptions,
    harness: &extern "C" fn(*const u8, usize) -> c_int,
) -> Result<(), Error> {
    if let Some(forks) = options.forks() {
        let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");
        if options.tui() {
            let monitor = TuiMonitor::new(TuiUI::new(options.fuzzer_name().to_string(), true));
            fuzz_many_forking(options, harness, shmem_provider, forks, monitor)
        } else if forks == 1 {
            #[cfg(unix)]
            let mut stderr = unsafe {
                let new_fd = libc::dup(std::io::stderr().as_raw_fd().into());
                File::from_raw_fd(new_fd.into())
            };
            let monitor = MultiMonitor::with_time(
                move |s| {
                    #[cfg(unix)]
                    writeln!(stderr, "{s}").expect("Could not write to stderr???");
                    #[cfg(not(unix))]
                    eprintln!("{s}");
                },
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            );
            fuzz_single_forking(options, harness, shmem_provider, monitor)
        } else {
            #[cfg(unix)]
            let stderr_fd = unsafe { libc::dup(std::io::stderr().as_raw_fd().into()) };
            let monitor = MultiMonitor::with_time(
                move |s| {
                    #[cfg(unix)]
                    {
                        // unfortunate requirement to meet Clone... thankfully, this does not
                        // generate effectively any overhead (no allocations, calls get merged)
                        let mut stderr = unsafe { File::from_raw_fd(stderr_fd) };
                        writeln!(stderr, "{s}").expect("Could not write to stderr???");
                        let _ = stderr.into_raw_fd(); // discard the file without closing
                    }
                    #[cfg(not(unix))]
                    eprintln!("{s}");
                },
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            );
            fuzz_many_forking(options, harness, shmem_provider, forks, monitor)
        }
    } else if options.tui() {
        // if the user specifies TUI, we assume they want to fork; it would not be possible to use
        // TUI safely otherwise
        let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");
        let monitor = TuiMonitor::new(TuiUI::new(options.fuzzer_name().to_string(), true));
        fuzz_many_forking(options, harness, shmem_provider, 1, monitor)
    } else {
        fuzz_with!(options, harness, do_fuzz, |fuzz_single| {
            let mgr = SimpleEventManager::new(SimpleMonitor::new(|s| eprintln!("{s}")));
            crate::start_fuzzing_single(fuzz_single, None, mgr)
        })
    }
}