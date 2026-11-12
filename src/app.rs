//! 主窗口的控件定义、事件处理与布局。
//! 尺寸以 96 DPI 下的逻辑像素书写，仅在高 DPI 环境按比例换算。

use crate::{backup, cleaner, logger, terminal, theme::Tone, winffi};
use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

const WINDOW_W: i32 = 600;
const WINDOW_H: i32 = 350;
const MARGIN: i32 = 12;
const GAP: i32 = 8;
const BUTTON_H: i32 = 32;
const EXIT_W: i32 = 72;
const MENU_W: i32 = 40;
/// 终端与窗口边缘的间距，按逻辑像素给定，实际取整到物理像素。
const TERMINAL_MARGIN: f64 = 1.5;
/// 终端与按钮行之间的纵向间距，独立于按钮之间的横向间距。
const TERMINAL_GAP: i32 = 16;

const PROJECT_URL: &str = "https://github.com/NeetheCheeBao/WinMediaUnbind";

/// 原生消息处理器需要一直存活，否则窗口拖动会失效。
const DRAG_HANDLER_ID: usize = 0x0001_0001;
/// 恢复列表窗口的消息处理器标识。
const RESTORE_HANDLER_ID: usize = 0x0001_0003;
/// 恢复列表内 ListBox 的消息处理器标识。
const LIST_HANDLER_ID: usize = 0x0001_0004;
/// 恢复窗口标题标签的消息处理器标识。
const LABEL_HANDLER_ID: usize = 0x0001_0005;

/// 恢复列表窗口及其控件，首次打开时创建。
/// 控件对象必须一直持有：nwg 在 drop 时会销毁对应窗口。
struct RestoreUi {
    window: nwg::Window,
    #[allow(dead_code)]
    caption: nwg::Label,
    list: nwg::ListBox<String>,
    restore_button: nwg::Button,
    #[allow(dead_code)]
    close_button: nwg::Button,
    /// 列表项与备份文件路径一一对应，顺序与列表一致。
    paths: Rc<RefCell<Vec<PathBuf>>>,
    #[allow(dead_code)]
    close_handler: nwg::RawEventHandler,
    #[allow(dead_code)]
    list_handler: nwg::RawEventHandler,
    #[allow(dead_code)]
    label_handler: nwg::RawEventHandler,
}

#[derive(Default, NwgUi)]
pub struct App {
    // 只保留 WS_VISIBLE：无标题栏、无边框，因此也没有系统按钮。
    // 创建时的占位标题，实际标题在 initialize 中按 version.ini 覆盖。
    #[nwg_control(
        size: (600, 350),
        position: (300, 300),
        title: "影音格式文件关联解除程序",
        flags: "VISIBLE"
    )]
    #[nwg_events(
        OnWindowClose: [App::exit],
        OnResize: [App::on_resize]
    )]
    window: nwg::Window,

    #[nwg_control(text: "视频")]
    #[nwg_events(OnButtonClick: [App::clean_video])]
    btn_video: nwg::Button,

    #[nwg_control(text: "音频")]
    #[nwg_events(OnButtonClick: [App::clean_audio])]
    btn_audio: nwg::Button,

    #[nwg_control(text: "播放列表")]
    #[nwg_events(OnButtonClick: [App::clean_playlist])]
    btn_playlist: nwg::Button,

    #[nwg_control(text: "全部解除")]
    #[nwg_events(OnButtonClick: [App::clean_all])]
    btn_all: nwg::Button,

    // 恢复选项的界面占位，点击行为待定。
    #[nwg_control(text: "恢复选项")]
    #[nwg_events(OnButtonClick: [App::open_restore])]
    btn_restore: nwg::Button,

    #[nwg_control(text: "退出")]
    #[nwg_events(OnButtonClick: [App::exit])]
    btn_exit: nwg::Button,

    #[nwg_control(text: "≡")]
    #[nwg_events(OnButtonClick: [App::open_menu])]
    btn_menu: nwg::Button,

    #[nwg_control(parent: window, popup: true)]
    popup_menu: nwg::Menu,

    #[nwg_control(parent: popup_menu, text: "项目地址")]
    #[nwg_events(OnMenuItemSelected: [App::open_link])]
    menu_link: nwg::MenuItem,

    // 控件不显示滚动条，滚动能力由滚轮与键盘消息驱动。
    #[nwg_control(readonly: true, flags: "VISIBLE | TAB_STOP")]
    log_view: nwg::RichTextBox,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::update_log])]
    notice: nwg::Notice,

    // 清理线程结束时用它通知界面恢复按钮。
    #[nwg_control]
    #[nwg_events(OnNotice: [App::on_clean_finished])]
    finish_notice: nwg::Notice,

    queue: logger::Queue,
    scale: Cell<f64>,
    drag_handler: RefCell<Option<nwg::RawEventHandler>>,
    scroll_handler: RefCell<Option<nwg::RawEventHandler>>,
    tooltip: RefCell<Option<nwg::Tooltip>>,
    restore_ui: RefCell<Option<RestoreUi>>,
}

impl App {
    /// 界面构建完成后的一次性初始化，必须在消息循环启动前调用。
    pub fn initialize(&self, scale: f64) {
        self.scale.set(scale);
        let px = |v: i32| (v as f64 * scale).round() as i32;

        // 窗口创建完成后直接改写样式位，确保系统标题栏与三个系统按钮都不存在。
        let hwnd = self.window.handle.hwnd();
        let chrome_removed = hwnd
            .map(|handle| winffi::strip_window_decorations(handle as _))
            .unwrap_or(false);
        if let Some(handle) = hwnd {
            winffi::set_window_icon(handle as _);
        }

        let width = px(WINDOW_W);
        let height = px(WINDOW_H);
        self.window.set_size(width as u32, height as u32);
        self.window
            .set_text(&format!("{} v{}", crate::APP_NAME, crate::APP_VERSION));
        if let Some(handle) = hwnd {
            winffi::center_window(handle as _, width, height);
        }

        *self.scroll_handler.borrow_mut() = terminal::setup(&self.log_view);
        self.log_view.set_focus();
        self.on_resize();
        self.enable_drag(scale);
        self.setup_tooltips(scale);

        if !chrome_removed {
            self.logger()
                .line("警告：标题栏样式清除失败。", Tone::Header);
        }

        let total = cleaner::VIDEO_EXTS.len()
            + cleaner::AUDIO_EXTS.len()
            + cleaner::PLAYLIST_EXTS.len();
        self.logger().line(
            format!("{}  v{}", crate::APP_NAME, crate::APP_VERSION),
            Tone::Header,
        );
        self.logger().line(
            "点击下方按钮即可解除对应文件格式，鼠标悬浮在按钮上可看到详细文件格式",
            Tone::Ready,
        );
        self.logger().line(
            format!(
                "共收录 {} 个扩展名：视频 {} / 音频 {} / 播放列表 {}。",
                total,
                cleaner::VIDEO_EXTS.len(),
                cleaner::AUDIO_EXTS.len(),
                cleaner::PLAYLIST_EXTS.len()
            ),
            Tone::Ready,
        );
    }

    /// 为按钮注册悬浮提示：分类按钮列出该组包含的全部扩展名。
    /// 提示文本由扩展名清单直接拼接，避免与清理逻辑脱节。
    fn setup_tooltips(&self, scale: f64) {
        let video = cleaner::VIDEO_EXTS.join(" ");
        let audio = cleaner::AUDIO_EXTS.join(" ");
        let playlist = cleaner::PLAYLIST_EXTS.join(" ");

        let mut tooltip = nwg::Tooltip::default();
        let built = nwg::Tooltip::builder()
            .register(&self.btn_video, &video)
            .register(&self.btn_audio, &audio)
            .register(&self.btn_playlist, &playlist)
            .register(&self.btn_all, "三类文件格式全部清除")
            .build(&mut tooltip);

        if built.is_ok() {
            // 扩展名清单很长，限制宽度让提示自动换行，否则会撑满屏幕。
            if let Some(handle) = tooltip.handle.hwnd() {
                let width = (420.0 * scale).round() as i32;
                winffi::set_tooltip_width(handle as _, width);
            }
            *self.tooltip.borrow_mut() = Some(tooltip);
        }
    }

    /// 打开恢复列表窗口；备份目录不存在时只在终端提示。
    fn open_restore(&self) {
        let Some(dir) = backup::backup_dir() else {
            self.logger()
                .line("无法定位程序所在目录，恢复功能不可用。", Tone::Header);
            return;
        };

        if !dir.is_dir() {
            self.logger()
                .line(format!("{}不存在，无可恢复项目", dir.display()), Tone::Header);
            return;
        }

        let entries = backup::list_entries(&dir);
        if self.restore_ui.borrow().is_none() {
            match self.build_restore_window() {
                Some(ui) => *self.restore_ui.borrow_mut() = Some(ui),
                None => {
                    self.logger().line("恢复列表窗口创建失败。", Tone::Header);
                    return;
                }
            }
        }

        if let Some(ui) = self.restore_ui.borrow().as_ref() {
            // 每次重新打开都要求重新选择，避免误点上一次的选中项。
            ui.restore_button.set_enabled(false);
            let labels: Vec<String> = entries.iter().map(|(line, _)| line.clone()).collect();
            *ui.paths.borrow_mut() = entries.into_iter().map(|(_, path)| path).collect();
            ui.list.set_collection(labels);
            // 小窗打开期间锁定主窗口，关闭后恢复。
            winffi::enable_window(self.window.handle.hwnd().unwrap_or(std::ptr::null_mut()) as _, false);
            ui.window.set_visible(true);
            ui.window.set_focus();
        }
    }

    /// 创建恢复列表窗口：无标题栏，底部为“恢复此项”与“关闭”。
    fn build_restore_window(&self) -> Option<RestoreUi> {
        let width = 360;
        let height = 340;
        let (x, y) = self.centered_position(width, height);

        let mut window = nwg::Window::default();
        nwg::Window::builder()
            .size((width, height))
            .position((x, y))
            .title("恢复选项")
            .flags(nwg::WindowFlags::empty())
            .build(&mut window)
            .ok()?;

        // 设为主窗口的属主窗口：不单独占任务栏，最小化时随主窗口一起隐藏。
        if let (Some(handle), Some(owner)) = (window.handle.hwnd(), self.window.handle.hwnd()) {
            winffi::set_window_owner(handle as _, owner as _);
            winffi::strip_window_decorations(handle as _);
        }

        let mut caption = nwg::Label::default();
        nwg::Label::builder()
            .parent(&window)
            .text("恢复列表")
            .position((14, 12))
            .size((width - 28, 22))
            .build(&mut caption)
            .ok()?;

        // 创建时就带上 LBS_DISABLENOSCROLL：项目不满一屏时滚动条以禁用态常显。
        const LBS_DISABLENOSCROLL: u32 = 0x0000_1000;
        let extra_flags = unsafe { nwg::ListBoxFlags::from_bits_unchecked(LBS_DISABLENOSCROLL) };

        let mut list = nwg::ListBox::<String>::default();
        nwg::ListBox::builder()
            .parent(&window)
            .flags(nwg::ListBoxFlags::VISIBLE | nwg::ListBoxFlags::TAB_STOP | extra_flags)
            .position((14, 40))
            .size((width - 28, height - 102))
            .build(&mut list)
            .ok()?;

        let button_y = height - 46;
        let mut restore_button = nwg::Button::default();
        nwg::Button::builder()
            .parent(&window)
            .text("恢复此项")
            .position((14, button_y))
            .size((120, 32))
            .build(&mut restore_button)
            .ok()?;
        restore_button.set_enabled(false);

        let mut close_button = nwg::Button::default();
        nwg::Button::builder()
            .parent(&window)
            .text("关闭")
            .position((width - 14 - 88, button_y))
            .size((88, 32))
            .build(&mut close_button)
            .ok()?;

        // 无标题栏后按钮即唯一的交互入口，事件在这里统一分发。
        let paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
        let paths_for_click = Rc::clone(&paths);
        let main_hwnd = self.window.handle.hwnd().unwrap_or(std::ptr::null_mut()) as isize;
        let restore_hwnd = restore_button.handle.hwnd().unwrap_or(std::ptr::null_mut()) as isize;
        let close_hwnd = close_button.handle.hwnd().unwrap_or(std::ptr::null_mut()) as isize;
        let list_hwnd = list.handle.hwnd().unwrap_or(std::ptr::null_mut()) as isize;
        let logger = logger::Logger::new(self.queue.clone(), self.notice.sender());

        // 列表：点击行尾以下的空白区域时取消选择。
        let list_handler = nwg::bind_raw_event_handler(
            &list.handle,
            LIST_HANDLER_ID,
            move |_h, msg, _wparam, lparam| {
                if msg == winffi::WM_LBUTTONDOWN {
                    // 点击行尾以下的空白区域时取消选择。
                    let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                    let count = winffi::send_message(list_hwnd as _, winffi::LB_GETCOUNT, 0, 0) as i32;
                    let item_height =
                        winffi::send_message(list_hwnd as _, winffi::LB_GETITEMHEIGHT, 0, 0) as i32;
                    let top =
                        winffi::send_message(list_hwnd as _, winffi::LB_GETTOPINDEX, 0, 0) as i32;
                    if item_height > 0 && y >= 0 && top + y / item_height >= count {
                        winffi::send_message(list_hwnd as _, winffi::LB_SETCURSEL, usize::MAX, 0);
                        winffi::enable_window(restore_hwnd as _, false);
                        return Some(0);
                    }
                    return None;
                }
                None
            },
        )
        .ok()?;

        // 标签是独立子窗口，需要自己把拖动转交给父窗口，否则整条标题行拖不动。
        let window_hwnd = window.handle.hwnd().unwrap_or(std::ptr::null_mut()) as isize;
        let label_handler = nwg::bind_raw_event_handler(
            &caption.handle,
            LABEL_HANDLER_ID,
            move |_h, msg, _wparam, _lparam| {
                if msg == winffi::WM_LBUTTONDOWN {
                    winffi::start_window_drag(window_hwnd as _);
                    return Some(0);
                }
                None
            },
        )
        .ok()?;

        let close_handler = nwg::bind_raw_event_handler(
            &window.handle,
            RESTORE_HANDLER_ID,
            move |hwnd, msg, wparam, lparam| {
                // 没有标题栏，整块背景充当拖动区，控件自身仍可点击。
                if msg == winffi::WM_NCHITTEST {
                    return Some(winffi::HTCAPTION);
                }

                if msg == winffi::WM_CLOSE {
                    winffi::show_window(hwnd as _, false);
                    winffi::enable_window(main_hwnd as _, true);
                    return Some(0);
                }

                if msg == winffi::WM_COMMAND {
                    let code = ((wparam >> 16) & 0xFFFF) as u32;
                    let source = lparam;
                    if source == list_hwnd && code == 1 {
                        // LBN_SELCHANGE：选中项目才允许恢复，点空白取消选择后重新禁用
                        let selected =
                            winffi::send_message(list_hwnd as _, winffi::LB_GETCURSEL, 0, 0) >= 0;
                        winffi::enable_window(restore_hwnd as _, selected);
                    } else if source == close_hwnd && code == 0 {
                        winffi::show_window(hwnd as _, false);
                        winffi::enable_window(main_hwnd as _, true);
                        return Some(0);
                    } else if source == restore_hwnd && code == 0 {
                        let index =
                            winffi::send_message(list_hwnd as _, winffi::LB_GETCURSEL, 0, 0);
                        let target = if index >= 0 {
                            paths_for_click.borrow().get(index as usize).cloned()
                        } else {
                            None
                        };
                        match target {
                            Some(path) => {
                                let name = path
                                    .file_name()
                                    .map(|value| value.to_string_lossy().into_owned())
                                    .unwrap_or_default();
                                match backup::restore_backup(&path) {
                                    Ok(report) => {
                                        winffi::refresh_associations();
                                        logger.line(
                                            format!(
                                                "  已恢复 {}：{} 个键 / {} 个值{}",
                                                name,
                                                report.keys,
                                                report.values,
                                                if report.failed > 0 {
                                                    format!(
                                                        "（另有 {} 个系统默认应用记录受保护，已跳过，不影响实际关联）",
                                                        report.failed
                                                    )
                                                } else {
                                                    String::new()
                                                }
                                            ),
                                            Tone::Success,
                                        );
                                        logger.line(
                                            ">> 已发送系统刷新信号，打开方式应已还原。",
                                            Tone::System,
                                        );
                                    }
                                    Err(err) => logger.line(
                                        format!("  恢复失败：{}", err),
                                        Tone::Header,
                                    ),
                                }
                            }
                            None => logger.line("  请先选择一个备份。", Tone::Menu),
                        }
                    }
                }

                None
            },
        )
        .ok()?;

        Some(RestoreUi {
            window,
            caption,
            list,
            restore_button,
            close_button,
            paths,
            close_handler,
            list_handler,
            label_handler,
        })
    }

    /// 以主窗口为基准计算新窗口的居中位置。
    fn centered_position(&self, width: i32, height: i32) -> (i32, i32) {
        match self.window.handle.hwnd() {
            Some(handle) => {
                let (left, top, right, bottom) = winffi::window_rect(handle as _);
                let x = left + ((right - left) - width) / 2;
                let y = top + ((bottom - top) - height) / 2;
                (x.max(0), y.max(0))
            }
            None => (200, 200),
        }
    }

    /// 无边框窗口没有系统标题栏，让蓝色区域充当拖动区，按钮行保持可点击。
    fn enable_drag(&self, scale: f64) {
        let reserved = ((BUTTON_H + MARGIN) as f64 * scale).round() as i32;
        let handler = nwg::bind_raw_event_handler(
            &self.window.handle,
            DRAG_HANDLER_ID,
            move |hwnd, msg, _wparam, lparam| {
                if msg == winffi::WM_NCHITTEST {
                    let (left, top, _right, bottom) = winffi::window_rect(hwnd as _);
                    let x = (lparam & 0xFFFF) as i16 as i32;
                    let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                    let rel_x = x - left;
                    let rel_y = y - top;
                    if rel_x >= 0 && rel_y >= 0 && rel_y < (bottom - top) - reserved {
                        return Some(winffi::HTCAPTION);
                    }
                    return Some(winffi::HTCLIENT);
                }
                None
            },
        );
        if let Ok(bound) = handler {
            *self.drag_handler.borrow_mut() = Some(bound);
        }
    }

    /// 构造一个可交给清理线程使用的日志写入端。
    fn logger(&self) -> logger::Logger {
        logger::Logger::new(self.queue.clone(), self.notice.sender())
    }

    /// 结束消息循环，退出程序。
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    /// 在浏览器打开项目主页。
    fn open_link(&self) {
        let _ = open::that(PROJECT_URL);
        self.logger().line("正在打开项目地址…", Tone::Menu);
    }

    /// 在汉堡按钮下方弹出菜单，坐标需从客户区换算到屏幕。
    fn open_menu(&self) {
        let (x, y) = self.btn_menu.position();
        let (_, h) = self.btn_menu.size();
        if let Some(hwnd) = self.window.handle.hwnd() {
            let (screen_x, screen_y) = winffi::client_to_screen(hwnd as _, x, y + h as i32);
            self.popup_menu.popup(screen_x, screen_y);
        }
    }

    /// 界面线程取出积压日志并逐行着色显示。
    fn update_log(&self) {
        for entry in logger::drain(&self.queue) {
            terminal::write_line(&self.log_view, &entry.stamp, &entry.text, entry.tone);
        }
    }

    fn clean_video(&self) {
        self.set_buttons_enabled(false);
        cleaner::start_cleaning(
            vec![("视频格式", cleaner::VIDEO_EXTS)],
            self.logger(),
            self.finish_notice.sender(),
        );
    }

    fn clean_audio(&self) {
        self.set_buttons_enabled(false);
        cleaner::start_cleaning(
            vec![("音频格式", cleaner::AUDIO_EXTS)],
            self.logger(),
            self.finish_notice.sender(),
        );
    }

    fn clean_playlist(&self) {
        self.set_buttons_enabled(false);
        cleaner::start_cleaning(
            vec![("播放列表格式", cleaner::PLAYLIST_EXTS)],
            self.logger(),
            self.finish_notice.sender(),
        );
    }

    fn clean_all(&self) {
        self.set_buttons_enabled(false);
        cleaner::start_cleaning(
            vec![
                ("所有格式（视频）", cleaner::VIDEO_EXTS),
                ("所有格式（音频）", cleaner::AUDIO_EXTS),
                ("所有格式（播放列表）", cleaner::PLAYLIST_EXTS),
            ],
            self.logger(),
            self.finish_notice.sender(),
        );
    }

    /// 任务执行期间禁用全部按钮，避免重复点击造成重复清理。
    fn set_buttons_enabled(&self, enabled: bool) {
        for button in [
            &self.btn_video,
            &self.btn_audio,
            &self.btn_playlist,
            &self.btn_all,
            &self.btn_restore,
            &self.btn_exit,
            &self.btn_menu,
        ] {
            button.set_enabled(enabled);
        }
    }

    /// 清理线程结束后恢复按钮可点击状态。
    fn on_clean_finished(&self) {
        self.set_buttons_enabled(true);
    }

    /// 按当前缩放系数重排控件；窗口尺寸为固定值，此方法也用于初次布局。
    fn on_resize(&self) {
        let (w, h) = self.window.size();
        if w == 0 || h == 0 {
            return;
        }

        let scale = self.scale.get().max(0.5);
        let px = |v: i32| (v as f64 * scale).round() as i32;

        let margin = px(MARGIN);
        let gap = px(GAP);
        let button_h = px(BUTTON_H);
        let exit_w = px(EXIT_W);
        let menu_w = px(MENU_W);
        let term_margin = (TERMINAL_MARGIN * scale).round() as i32;
        let term_gap = px(TERMINAL_GAP);

        let button_y = h as i32 - button_h - margin;
        if button_y <= margin {
            return;
        }

        // 五个分类按钮平分剩余宽度，退出与菜单按钮保持固定宽度。
        let free = w as i32 - margin * 2 - gap * 6 - exit_w - menu_w;
        let group_w = (free / 5).max(px(48));

        let mut x = margin;
        for button in [
            &self.btn_video,
            &self.btn_audio,
            &self.btn_playlist,
            &self.btn_all,
            &self.btn_restore,
        ] {
            button.set_position(x, button_y);
            button.set_size(group_w as u32, button_h as u32);
            x += group_w + gap;
        }

        self.btn_exit.set_position(x, button_y);
        self.btn_exit.set_size(exit_w as u32, button_h as u32);
        x += exit_w + gap;

        self.btn_menu.set_position(x, button_y);
        self.btn_menu.set_size(menu_w as u32, button_h as u32);

        // 终端贴近窗口边缘，只留一道细缝；底部与按钮行之间留出更大的间距。
        let log_h = button_y - term_gap - term_margin;
        if log_h > 0 {
            let log_w = w as i32 - term_margin * 2;
            self.log_view.set_position(term_margin, term_margin);
            self.log_view.set_size(log_w as u32, log_h as u32);
        }
    }
}
