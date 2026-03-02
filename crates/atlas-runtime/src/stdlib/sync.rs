//! Synchronization primitives for Atlas stdlib.
//!
//! Provides RwLock, Semaphore, and AtomicCounter — essential for
//! production concurrent workloads beyond basic Mutex.

use crate::span::Span;
use crate::value::{RuntimeError, Value, ValueArray};

use std::collections::HashMap as StdHashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

// ── Handle management ────────────────────────────────────────────────

static NEXT_SYNC_ID: AtomicU64 = AtomicU64::new(1);

static RWLOCKS: std::sync::OnceLock<Mutex<StdHashMap<u64, Arc<RwLock<Value>>>>> =
    std::sync::OnceLock::new();
static SEMAPHORES: std::sync::OnceLock<Mutex<StdHashMap<u64, Arc<SemaphoreInner>>>> =
    std::sync::OnceLock::new();
static ATOMICS: std::sync::OnceLock<Mutex<StdHashMap<u64, Arc<AtomicI64>>>> =
    std::sync::OnceLock::new();

fn rwlocks() -> &'static Mutex<StdHashMap<u64, Arc<RwLock<Value>>>> {
    RWLOCKS.get_or_init(|| Mutex::new(StdHashMap::new()))
}
fn semaphores() -> &'static Mutex<StdHashMap<u64, Arc<SemaphoreInner>>> {
    SEMAPHORES.get_or_init(|| Mutex::new(StdHashMap::new()))
}
fn atomics() -> &'static Mutex<StdHashMap<u64, Arc<AtomicI64>>> {
    ATOMICS.get_or_init(|| Mutex::new(StdHashMap::new()))
}

const RWLOCK_TAG: &str = "__rwlock__";
const SEMAPHORE_TAG: &str = "__semaphore__";
const ATOMIC_TAG: &str = "__atomic__";

fn make_handle(tag: &str, id: u64) -> Value {
    Value::Array(ValueArray::from_vec(vec![
        Value::string(tag.to_string()),
        Value::Number(id as f64),
    ]))
}

fn extract_handle_id(
    value: &Value,
    expected_tag: &str,
    func_name: &str,
    span: Span,
) -> Result<u64, RuntimeError> {
    match value {
        Value::Array(arr) if arr.len() == 2 => {
            let tag = match &arr.as_slice()[0] {
                Value::String(s) => s.as_str(),
                _ => return Err(super::stdlib_arg_error(func_name, "handle", value, span)),
            };
            let id = match &arr.as_slice()[1] {
                Value::Number(n) => *n as u64,
                _ => return Err(super::stdlib_arg_error(func_name, "handle", value, span)),
            };
            if tag == expected_tag {
                Ok(id)
            } else {
                Err(RuntimeError::InvalidStdlibArgument {
                    msg: format!(
                        "{}(): expected {} handle, got {} handle",
                        func_name, expected_tag, tag
                    ),
                    span,
                })
            }
        }
        _ => Err(super::stdlib_arg_error(func_name, "handle", value, span)),
    }
}

// ── RwLock ────────────────────────────────────────────────────────────

/// rwLockNew(initial_value: any) -> rwlock handle
pub fn rwlock_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("rwLockNew", 1, args.len(), span));
    }
    let id = NEXT_SYNC_ID.fetch_add(1, Ordering::Relaxed);
    let lock = Arc::new(RwLock::new(args[0].clone()));
    rwlocks().lock().unwrap().insert(id, lock);
    Ok(make_handle(RWLOCK_TAG, id))
}

/// rwLockRead(handle: rwlock) -> value (acquires read lock, returns snapshot)
pub fn rwlock_read(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("rwLockRead", 1, args.len(), span));
    }
    let id = extract_handle_id(&args[0], RWLOCK_TAG, "rwLockRead", span)?;
    let lock = rwlocks().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "rwLockRead(): handle has been destroyed".into(),
            span,
        }
    })?;
    let guard = lock
        .read()
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("rwLockRead(): lock poisoned: {}", e),
            span,
        })?;
    Ok(guard.clone())
}

/// rwLockWrite(handle: rwlock, new_value: any) -> null (acquires write lock, sets value)
pub fn rwlock_write(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error(
            "rwLockWrite",
            2,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], RWLOCK_TAG, "rwLockWrite", span)?;
    let lock = rwlocks().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "rwLockWrite(): handle has been destroyed".into(),
            span,
        }
    })?;
    let mut guard = lock
        .write()
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("rwLockWrite(): lock poisoned: {}", e),
            span,
        })?;
    *guard = args[1].clone();
    Ok(Value::Null)
}

/// rwLockTryRead(handle: rwlock) -> Option<value>
pub fn rwlock_try_read(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "rwLockTryRead",
            1,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], RWLOCK_TAG, "rwLockTryRead", span)?;
    let lock = rwlocks().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "rwLockTryRead(): handle has been destroyed".into(),
            span,
        }
    })?;
    let result = match lock.try_read() {
        Ok(guard) => Value::Option(Some(Box::new(guard.clone()))),
        Err(_) => Value::Option(None),
    };
    Ok(result)
}

/// rwLockTryWrite(handle: rwlock, new_value: any) -> bool
pub fn rwlock_try_write(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error(
            "rwLockTryWrite",
            2,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], RWLOCK_TAG, "rwLockTryWrite", span)?;
    let lock = rwlocks().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "rwLockTryWrite(): handle has been destroyed".into(),
            span,
        }
    })?;
    let result = match lock.try_write() {
        Ok(mut guard) => {
            *guard = args[1].clone();
            true
        }
        Err(_) => false,
    };
    Ok(Value::Bool(result))
}

// ── Semaphore ────────────────────────────────────────────────────────

struct SemaphoreInner {
    mutex: Mutex<usize>,
    condvar: Condvar,
    max_permits: usize,
}

impl SemaphoreInner {
    fn new(permits: usize) -> Self {
        SemaphoreInner {
            mutex: Mutex::new(permits),
            condvar: Condvar::new(),
            max_permits: permits,
        }
    }

    fn acquire(&self) {
        let mut count = self.mutex.lock().unwrap();
        while *count == 0 {
            count = self.condvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    fn try_acquire(&self) -> bool {
        let mut count = self.mutex.lock().unwrap();
        if *count > 0 {
            *count -= 1;
            true
        } else {
            false
        }
    }

    fn release(&self) {
        let mut count = self.mutex.lock().unwrap();
        if *count < self.max_permits {
            *count += 1;
            self.condvar.notify_one();
        }
    }

    fn available(&self) -> usize {
        *self.mutex.lock().unwrap()
    }
}

/// semaphoreNew(permits: number) -> semaphore handle
pub fn semaphore_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "semaphoreNew",
            1,
            args.len(),
            span,
        ));
    }
    let permits = match &args[0] {
        Value::Number(n) => {
            let p = *n as usize;
            if p == 0 {
                return Err(RuntimeError::InvalidStdlibArgument {
                    msg: "semaphoreNew(): permits must be > 0".into(),
                    span,
                });
            }
            p
        }
        _ => {
            return Err(super::stdlib_arg_error(
                "semaphoreNew",
                "number",
                &args[0],
                span,
            ))
        }
    };

    let id = NEXT_SYNC_ID.fetch_add(1, Ordering::Relaxed);
    let sem = Arc::new(SemaphoreInner::new(permits));
    semaphores().lock().unwrap().insert(id, sem);
    Ok(make_handle(SEMAPHORE_TAG, id))
}

/// semaphoreAcquire(handle: semaphore) -> null (blocks until permit available)
pub fn semaphore_acquire(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "semaphoreAcquire",
            1,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], SEMAPHORE_TAG, "semaphoreAcquire", span)?;
    let sem = semaphores()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "semaphoreAcquire(): handle has been destroyed".into(),
            span,
        })?;
    sem.acquire();
    Ok(Value::Null)
}

/// semaphoreTryAcquire(handle: semaphore) -> bool
pub fn semaphore_try_acquire(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "semaphoreTryAcquire",
            1,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], SEMAPHORE_TAG, "semaphoreTryAcquire", span)?;
    let sem = semaphores()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "semaphoreTryAcquire(): handle has been destroyed".into(),
            span,
        })?;
    Ok(Value::Bool(sem.try_acquire()))
}

/// semaphoreRelease(handle: semaphore) -> null
pub fn semaphore_release(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "semaphoreRelease",
            1,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], SEMAPHORE_TAG, "semaphoreRelease", span)?;
    let sem = semaphores()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "semaphoreRelease(): handle has been destroyed".into(),
            span,
        })?;
    sem.release();
    Ok(Value::Null)
}

/// semaphoreAvailable(handle: semaphore) -> number
pub fn semaphore_available(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error(
            "semaphoreAvailable",
            1,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], SEMAPHORE_TAG, "semaphoreAvailable", span)?;
    let sem = semaphores()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "semaphoreAvailable(): handle has been destroyed".into(),
            span,
        })?;
    Ok(Value::Number(sem.available() as f64))
}

// ── AtomicCounter ────────────────────────────────────────────────────

/// atomicNew(initial: number) -> atomic handle
pub fn atomic_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("atomicNew", 1, args.len(), span));
    }
    let initial = match &args[0] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicNew",
                "number",
                &args[0],
                span,
            ))
        }
    };

    let id = NEXT_SYNC_ID.fetch_add(1, Ordering::Relaxed);
    let atomic = Arc::new(AtomicI64::new(initial));
    atomics().lock().unwrap().insert(id, atomic);
    Ok(make_handle(ATOMIC_TAG, id))
}

/// atomicLoad(handle: atomic) -> number
pub fn atomic_load(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("atomicLoad", 1, args.len(), span));
    }
    let id = extract_handle_id(&args[0], ATOMIC_TAG, "atomicLoad", span)?;
    let atomic = atomics().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "atomicLoad(): handle has been destroyed".into(),
            span,
        }
    })?;
    Ok(Value::Number(atomic.load(Ordering::SeqCst) as f64))
}

/// atomicStore(handle: atomic, value: number) -> null
pub fn atomic_store(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error(
            "atomicStore",
            2,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], ATOMIC_TAG, "atomicStore", span)?;
    let val = match &args[1] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicStore",
                "number",
                &args[1],
                span,
            ))
        }
    };
    let atomic = atomics().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "atomicStore(): handle has been destroyed".into(),
            span,
        }
    })?;
    atomic.store(val, Ordering::SeqCst);
    Ok(Value::Null)
}

/// atomicAdd(handle: atomic, delta: number) -> number (previous value)
pub fn atomic_add(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("atomicAdd", 2, args.len(), span));
    }
    let id = extract_handle_id(&args[0], ATOMIC_TAG, "atomicAdd", span)?;
    let delta = match &args[1] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicAdd",
                "number",
                &args[1],
                span,
            ))
        }
    };
    let atomic = atomics().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "atomicAdd(): handle has been destroyed".into(),
            span,
        }
    })?;
    let prev = atomic.fetch_add(delta, Ordering::SeqCst);
    Ok(Value::Number(prev as f64))
}

/// atomicSub(handle: atomic, delta: number) -> number (previous value)
pub fn atomic_sub(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("atomicSub", 2, args.len(), span));
    }
    let id = extract_handle_id(&args[0], ATOMIC_TAG, "atomicSub", span)?;
    let delta = match &args[1] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicSub",
                "number",
                &args[1],
                span,
            ))
        }
    };
    let atomic = atomics().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "atomicSub(): handle has been destroyed".into(),
            span,
        }
    })?;
    let prev = atomic.fetch_sub(delta, Ordering::SeqCst);
    Ok(Value::Number(prev as f64))
}

/// atomicCompareExchange(handle: atomic, expected: number, desired: number) -> bool
pub fn atomic_compare_exchange(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(super::stdlib_arity_error(
            "atomicCompareExchange",
            3,
            args.len(),
            span,
        ));
    }
    let id = extract_handle_id(&args[0], ATOMIC_TAG, "atomicCompareExchange", span)?;
    let expected = match &args[1] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicCompareExchange",
                "number",
                &args[1],
                span,
            ))
        }
    };
    let desired = match &args[2] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err(super::stdlib_arg_error(
                "atomicCompareExchange",
                "number",
                &args[2],
                span,
            ))
        }
    };
    let atomic = atomics().lock().unwrap().get(&id).cloned().ok_or_else(|| {
        RuntimeError::InvalidStdlibArgument {
            msg: "atomicCompareExchange(): handle has been destroyed".into(),
            span,
        }
    })?;
    let result = atomic.compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst);
    Ok(Value::Bool(result.is_ok()))
}
