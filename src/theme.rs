//! 终端配色与日志级别。

/// 日志级别，决定终端中该行文字的显示颜色。
#[derive(Clone, Copy, Debug)]
pub enum Tone {
    /// 启动提示与操作说明。
    Ready,
    /// 任务分组标题。
    Header,
    /// 单个扩展名清理成功。
    Success,
    /// 系统级通知。
    System,
    /// 任务分组结束。
    Done,
    /// 菜单等辅助操作。
    Menu,
}

impl Tone {
    /// 返回该级别对应的 [红, 绿, 蓝] 分量。
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Tone::Ready => [150, 200, 255],
            Tone::Header => [255, 209, 102],
            Tone::Success => [122, 231, 166],
            Tone::System => [86, 216, 255],
            Tone::Done => [255, 209, 102],
            Tone::Menu => [204, 190, 255],
        }
    }
}

/// 终端背景色，采用低亮度深蓝以与前景文字保持足够对比。
pub const TERMINAL_BG: [u8; 3] = [10, 32, 92];

/// 时间戳颜色，低饱和灰蓝，避免与分级正文颜色争夺注意力。
pub const TIMESTAMP_COLOR: [u8; 3] = [112, 146, 200];
