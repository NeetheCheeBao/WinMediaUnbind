//! 清理线程与界面线程之间的日志传递。
//! 清理线程只负责写入队列并发出通知，界面线程统一取出后着色显示。

use crate::theme::Tone;
use crate::winffi;
use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};

/// 一条待显示的日志，文本中不含换行符。
pub struct Entry {
    /// 消息产生时的本地时间戳，格式形如 [21:15:30] 。
    pub stamp: String,
    pub text: String,
    pub tone: Tone,
}

/// 共享的待显示日志队列。
pub type Queue = Arc<Mutex<Vec<Entry>>>;

/// 日志写入端，持有一个通知发送器用于唤醒界面线程。
pub struct Logger {
    queue: Queue,
    sender: nwg::NoticeSender,
}

impl Logger {
    pub fn new(queue: Queue, sender: nwg::NoticeSender) -> Self {
        Self { queue, sender }
    }

    /// 追加一行日志并通知界面线程刷新。
    /// 时间戳在消息产生时生成，避免批量积压后显示为同一时刻。
    pub fn line(&self, text: impl Into<String>, tone: Tone) {
        let (hour, minute, second) = winffi::local_hms();
        let stamp = format!("[{:02}:{:02}:{:02}] ", hour, minute, second);

        if let Ok(mut pending) = self.queue.lock() {
            pending.push(Entry {
                stamp,
                text: text.into(),
                tone,
            });
        }
        self.sender.notice();
    }
}

/// 取出当前积压的全部日志，保持写入顺序。
pub fn drain(queue: &Queue) -> Vec<Entry> {
    match queue.lock() {
        Ok(mut pending) => pending.drain(..).collect(),
        Err(_) => Vec::new(),
    }
}
