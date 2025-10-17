use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use tokio::sync::mpsc::Sender;

pub static TASK_LOGGERS: Lazy<DashMap<String, Sender<String>>> = Lazy::new(DashMap::new);

tokio::task_local! {
    pub static ASYNC_TASK_ID: String;
}

thread_local! {
    static BLOCKING_TASK_ID: RefCell<Option<String>> = RefCell::new(None);
}

pub fn set_async_task_id(id: String) {
    ASYNC_TASK_ID.scope(id, async {});
}

pub fn set_blocking_task_id(id: String) {
    BLOCKING_TASK_ID.with(|cell| {
        *cell.borrow_mut() = Some(id);
    });
}

pub fn get_blocking_task_id() -> Option<String> {
    BLOCKING_TASK_ID.with(|cell| cell.borrow().clone())
}

pub fn clear_blocking_task_id() {
    BLOCKING_TASK_ID.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

pub fn current_task_id() -> Option<String> {
    if let Ok(id) = ASYNC_TASK_ID.try_with(|id| id.clone()) {
        return Some(id);
    }
    BLOCKING_TASK_ID.with(|cell| cell.borrow().clone())
}

pub fn send_task_log(msg: String) {
    if let Some(task_id) = current_task_id()
        && let Some(sender) = TASK_LOGGERS.get(&task_id)
    {
        let _ = sender.try_send(msg.clone());
    }
    println!("{}", msg);
}

#[macro_export]
macro_rules! task_log {
    ($($arg:tt)*) => {{
        $crate::logs::send_task_log(format!($($arg)*));
    }};
}

// #[macro_export]
// macro_rules! task_log {
//     // Explicit task_id: task_log!(task_id, "message {}", value)
//     ($task_id:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
//         let msg = format!($fmt $(, $arg)*);
//         if let Some(sender) = $crate::logs::TASK_LOGGERS.get(&$task_id) {
//             let _ = sender.try_send(msg.clone());
//         }
//         println!("{}", msg);
//     }};

//     // Implicit task_id: task_log!("message {}", value)
//     ($fmt:literal $(, $arg:expr)* $(,)?) => {{
//         let msg = format!($fmt $(, $arg)*);
//         $crate::logs::send_task_log(msg);
//     }};
// }
