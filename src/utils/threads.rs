use std::{marker::PhantomData, panic::{AssertUnwindSafe, catch_unwind, resume_unwind}, sync::{Arc, Condvar, LazyLock, Mutex, atomic::{AtomicBool, Ordering}}};


pub static THREAD_POOL: LazyLock<Arc<Mutex<ThreadPool>>> = LazyLock::new(|| Arc::new(Mutex::new(ThreadPool::default())));
pub static PROCESS_COUNT: LazyLock<usize> = LazyLock::new(|| THREAD_POOL.lock().unwrap().handles.len());

trait ThreadJob = FnOnce() + 'static + Send;
pub struct ThreadPool {
    handles: Vec<ThreadHandle>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        let thread_count = num_cpus::get();
        // let thread_count = 32;
        let mut handles = Vec::with_capacity(thread_count);
        for i in 0..thread_count {
            let (send_job, recv_job) = std::sync::mpsc::channel::<Box<dyn ThreadJob>>();
            let busy_semaphore = Arc::new(Mutex::new(false));
            let thread_side_busy_semaphore = busy_semaphore.clone();

            let join_handle = std::thread::spawn(move || {
                loop {
                    if let Ok(f) = recv_job.recv() {
                        *thread_side_busy_semaphore.lock().unwrap() = true;
                        (f)();
                        log::info!("Job on thread {i} finished");
                        *thread_side_busy_semaphore.lock().unwrap() = false;
                    } else {
                        std::hint::spin_loop();
                    }
                }
            }); 

            handles.push(ThreadHandle {
                send_job, join_handle, busy_semaphore, id: i
            });
        }

        Self {
            handles
        }
    }
}

impl ThreadPool {
    pub fn add_job(&mut self, job: Box<dyn ThreadJob>) {
        for handle in self.handles.iter_mut() {
            if !handle.is_busy() { 
                handle.send_job.send(job).unwrap(); 
                handle.force_busy(); 
                break; 
            }
        }
    }

    pub fn scope<'env, F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Scope<'_, 'env>) -> R,
    {
        let mut scope = Scope {
            pool: self,
            state: Arc::new(ScopeState {
                pending: Mutex::new(0),
                cv: Condvar::new(),
                panicked: AtomicBool::new(false),
            }),
            _env: PhantomData,
        };

        // Catch panics from `f` so we ALWAYS wait before borrows can end.
        let result = catch_unwind(AssertUnwindSafe(|| f(&mut scope)));
        scope.state.wait();

        match result {
            Err(e) => resume_unwind(e),
            Ok(_) if scope.state.panicked.load(Ordering::SeqCst) => {
                panic!("a scoped job panicked")
            }
            Ok(r) => r,
        }
    }
}

struct ThreadHandle {
    send_job: std::sync::mpsc::Sender<Box<dyn ThreadJob>>,
    join_handle: std::thread::JoinHandle<()>,
    busy_semaphore: Arc<Mutex<bool>>,
    id: usize,
}

impl ThreadHandle {
    fn is_busy(&self) -> bool {
        log::info!("{} is_busy called", self.id);
        match self.busy_semaphore.try_lock() {
            Ok(val) => *val,
            Err(_) => true
        }
        // *self.busy_semaphore.lock().unwrap()
    }

    fn force_busy(&self) {
        *self.busy_semaphore.lock().unwrap() = true;
    }
}

struct ScopeState {
    pending: Mutex<usize>,
    cv: Condvar,
    panicked: AtomicBool,
}

impl ScopeState {
    fn job_done(&self) {
        let mut p = self.pending.lock().unwrap();
        *p -= 1;
        if *p == 0 {
            self.cv.notify_all();
        }
    }
    fn wait(&self) {
        let mut p = self.pending.lock().unwrap();
        while *p > 0 {
            p = self.cv.wait(p).unwrap();
        }
    }
}

pub struct Scope<'pool, 'env> {
    pool: &'pool mut ThreadPool,
    state: Arc<ScopeState>,
    // Makes 'env invariant, so callers can't shrink it to something unsound
    _env: PhantomData<&'env mut &'env ()>,
}

impl<'pool, 'env> Scope<'pool, 'env> {
    pub fn add_job<F>(&mut self, job: F)
    where
        F: FnOnce() + Send + 'env,
    {
        *self.state.pending.lock().unwrap() += 1;
        let state = Arc::clone(&self.state);

        let job: Box<dyn FnOnce() + Send + 'env> = Box::new(move || {
            if catch_unwind(AssertUnwindSafe(job)).is_err() {
                state.panicked.store(true, Ordering::SeqCst);
            }
            state.job_done();
        });

        // SAFETY: `ThreadPool::scope` does not return until every job has
        // run to completion (it waits on `pending`, even if `f` panics),
        // so the borrows captured with lifetime 'env outlive the job.
        let job: Box<dyn ThreadJob> =
            unsafe { std::mem::transmute(job) };

        self.pool.add_job(job);
    }
}