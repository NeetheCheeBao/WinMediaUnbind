#![windows_subsystem = "windows"]

use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use winreg::enums::*;
use winreg::RegKey;

#[link(name = "shell32")]
extern "system" {
    fn SHChangeNotify(wEventId: i32, uFlags: u32, dwItem1: *const c_void, dwItem2: *const c_void);
}

#[link(name = "user32")]
extern "system" {
    fn SendMessageW(hWnd: *mut c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

const SHCNE_ASSOCCHANGED: i32 = 0x08000000;
const SHCNF_IDLIST: u32 = 0x0000;

const VIDEO_EXTS: &[&str] = &[
    "avi", "wmv", "wmp", "wm", "asf", "mpg", "mpeg", "mpe", "m1v", "m2v", "mpv2", "mp2v", "ts",
    "tp", "tpr", "trp", "vob", "ifo", "ogm", "ogv", "mp4", "m4v", "m4p", "m4b", "3gp", "3gpp",
    "3g2", "3gp2", "mkv", "rm", "ram", "rmvb", "rpm", "flv", "mov", "qt", "nsv", "dpg", "m2ts",
    "m2t", "mts", "dvr-ms", "k3g", "skm", "evo", "nsr", "amv", "divx", "webm", "wtv", "f4v", "mxf",
];

const AUDIO_EXTS: &[&str] = &[
    "wav", "wma", "mpa", "mp2", "m1a", "m2a", "mp3", "ogg", "m4a", "aac", "mka", "ra", "flac",
    "ape", "mpc", "mod", "ac3", "eac3", "dts", "dtshd", "wv", "tak", "cda", "dsf", "tta", "aiff",
    "aif", "opus", "amr",
];

const PLAYLIST_EXTS: &[&str] = &[
    "asx", "m3u", "m3u8", "pls", "wvx", "wax", "wmx", "cue", "mpls", "mpl", "dpl", "xspf", "mpd",
];

#[derive(Default, NwgUi)]
pub struct WinMediaUnbind {
    #[nwg_control(size: (800, 600), position: (300, 300), title: "影音格式文件关联解除程序v1.0.0")]
    #[nwg_events(
        OnWindowClose: [WinMediaUnbind::exit],
        OnResize: [WinMediaUnbind::on_resize]
    )]
    window: nwg::Window,

    #[nwg_control(text: "项目地址")]
    #[nwg_events( OnButtonClick: [WinMediaUnbind::open_link] )]
    btn_link: nwg::Button,

    #[nwg_control(readonly: true)]
    log_view: nwg::TextBox,

    #[nwg_control(text: "删除视频格式关联")]
    #[nwg_events( OnButtonClick: [WinMediaUnbind::clean_video] )]
    btn_video: nwg::Button,

    #[nwg_control(text: "删除音频格式关联")]
    #[nwg_events( OnButtonClick: [WinMediaUnbind::clean_audio] )]
    btn_audio: nwg::Button,

    #[nwg_control(text: "删除播放列表格式关联")]
    #[nwg_events( OnButtonClick: [WinMediaUnbind::clean_playlist] )]
    btn_playlist: nwg::Button,

    #[nwg_control(text: "全部解除关联")]
    #[nwg_events( OnButtonClick: [WinMediaUnbind::clean_all] )]
    btn_all: nwg::Button,

    #[nwg_control]
    #[nwg_events( OnNotice: [WinMediaUnbind::update_log] )]
    notice: nwg::Notice,

    logs: Arc<Mutex<String>>,
}

impl WinMediaUnbind {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn open_link(&self) {
        let _ = open::that("https://github.com/NeetheCheeBao/WinMediaUnbind");
        self.append_log("正在打开项目地址...\r\n");
    }

    fn append_log(&self, msg: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push_str(msg);
        }
        self.notice.sender().notice();
    }

    fn update_log(&self) {
        if let Ok(logs) = self.logs.lock() {
            self.log_view.set_text(&logs);
            self.log_view.set_selection(u32::MAX..u32::MAX);
            if let Some(hwnd) = self.log_view.handle.hwnd() {
                unsafe {
                    SendMessageW(hwnd as _, 0x0115, 7, 0);
                }
            }
        }
    }

    fn on_resize(&self) {
        let (w, h) = self.window.size();
        if w == 0 || h == 0 { return; }

        self.btn_link.set_position(10, 10);
        self.btn_link.set_size(100, 30);

        let btn_h = 35;
        let spacing = 10;
        let margin = 10;
        let btn_y = h as i32 - btn_h as i32 - margin;
        
        let total_spacing = spacing * 3;
        let available_w = w as i32 - (margin * 2) - total_spacing;
        let btn_w = available_w / 4;

        self.btn_video.set_position(margin, btn_y);
        self.btn_video.set_size(btn_w as u32, btn_h as u32);

        self.btn_audio.set_position(margin + btn_w + spacing, btn_y);
        self.btn_audio.set_size(btn_w as u32, btn_h as u32);

        self.btn_playlist.set_position(margin + (btn_w * 2) + (spacing * 2), btn_y);
        self.btn_playlist.set_size(btn_w as u32, btn_h as u32);

        self.btn_all.set_position(margin + (btn_w * 3) + (spacing * 3), btn_y);
        self.btn_all.set_size(btn_w as u32, btn_h as u32);

        let log_y = 50;
        let log_h = btn_y - log_y - margin;
        if log_h > 0 {
            self.log_view.set_position(margin, log_y);
            self.log_view.set_size(w - (margin as u32 * 2), log_h as u32);
        }
    }

    fn run_cleaning(&self, label: String, extensions: &'static [&'static str]) {
        let logs_clone = self.logs.clone();
        let notice_clone = self.notice.sender();

        thread::spawn(move || {
            {
                if let Ok(mut logs) = logs_clone.lock() {
                    logs.push_str(&format!("--- 开始处理: {} ---\r\n", label));
                }
                notice_clone.notice();
            }

            let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
            let user_ext_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts";
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);

            for ext in extensions {
                let dot_ext = format!(".{}", ext);
                let mut deleted_count = 0;

                if hkcr.delete_subkey_all(&dot_ext).is_ok() {
                    deleted_count += 1;
                }

                if let Ok(file_exts_key) = hkcu.open_subkey(user_ext_path) {
                    if file_exts_key.delete_subkey_all(&dot_ext).is_ok() {
                        deleted_count += 1;
                    }
                }

                if deleted_count > 0 {
                    if let Ok(mut logs) = logs_clone.lock() {
                        logs.push_str(&format!("已清理: {}\r\n", dot_ext));
                    }
                    notice_clone.notice();
                }
            }

            unsafe {
                SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, ptr::null(), ptr::null());
            }

            if let Ok(mut logs) = logs_clone.lock() {
                logs.push_str(">> 已发送系统刷新信号，图标应已更新。\r\n");
                logs.push_str(&format!("--- {} 处理完成 ---\r\n", label));
            }
            notice_clone.notice();
        });
    }

    fn clean_video(&self) {
        self.run_cleaning("视频格式".to_string(), VIDEO_EXTS);
    }

    fn clean_audio(&self) {
        self.run_cleaning("音频格式".to_string(), AUDIO_EXTS);
    }

    fn clean_playlist(&self) {
        self.run_cleaning("播放列表格式".to_string(), PLAYLIST_EXTS);
    }

    fn clean_all(&self) {
        self.run_cleaning("所有格式(视频)".to_string(), VIDEO_EXTS);
        self.run_cleaning("所有格式(音频)".to_string(), AUDIO_EXTS);
        self.run_cleaning("所有格式(播放列表)".to_string(), PLAYLIST_EXTS);
    }
}

fn main() {
    nwg::init().expect("Failed to init nwg");
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Microsoft YaHei")
        .size(17)
        .build(&mut font)
        .ok();
    nwg::Font::set_global_default(Some(font));

    let app = WinMediaUnbind::build_ui(Default::default()).expect("Failed to build UI");
    
    app.on_resize();
    app.append_log("就绪... 等待指令。\r\n");
    
    nwg::dispatch_thread_events();
}