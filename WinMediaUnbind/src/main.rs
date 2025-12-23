#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 隐藏命令行窗口

use eframe::egui;
use std::ffi::c_void;
use std::fs;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use winreg::enums::*;
use winreg::RegKey;

#[link(name = "shell32")]
extern "system" {
    fn SHChangeNotify(
        wEventId: i32,
        uFlags: u32,
        dwItem1: *const c_void,
        dwItem2: *const c_void,
    );
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

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_path = "C:\\Windows\\Fonts\\msyh.ttc";

    match fs::read(font_path) {
        Ok(font_data) => {
            fonts.font_data.insert(
                "microsoft_yahei".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "microsoft_yahei".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "microsoft_yahei".to_owned());
            ctx.set_fonts(fonts);
        }
        Err(e) => {
            eprintln!("加载字体失败: {}", e);
        }
    }
}

fn refresh_system_icons() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, ptr::null(), ptr::null());
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("影音格式文件关联解除程序v1.0.0 by NeetheCheeBao"),
        ..Default::default()
    };
    eframe::run_native(
        "影音格式文件关联解除程序",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(WinMediaUnbind::default())
        }),
    )
}

struct WinMediaUnbind {
    logs: Arc<Mutex<String>>,
}

impl Default for WinMediaUnbind {
    fn default() -> Self {
        Self {
            logs: Arc::new(Mutex::new(String::from("就绪... 等待指令。\n"))),
        }
    }
}

impl WinMediaUnbind {
    fn log(&self, message: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push_str(message);
            logs.push('\n');
        }
    }

    fn run_cleaning(&self, label: &str, extensions: &'static [&'static str]) {
        let logs_clone = self.logs.clone();
        let label = label.to_string();

        thread::spawn(move || {
            {
                if let Ok(mut logs) = logs_clone.lock() {
                    logs.push_str(&format!("--- 开始处理: {} ---\n", label));
                }
            }

            let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
            let user_ext_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts";
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);

            for ext in extensions {
                let dot_ext = format!(".{}", ext);
                let mut deleted_count = 0;

                match hkcr.delete_subkey_all(&dot_ext) {
                    Ok(_) => deleted_count += 1,
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::NotFound {
                            // logs_clone.lock().unwrap().push_str(&format!("无法删除: {}\n", e));
                        }
                    }
                }

                if let Ok(file_exts_key) = hkcu.open_subkey(user_ext_path) {
                    match file_exts_key.delete_subkey_all(&dot_ext) {
                        Ok(_) => deleted_count += 1,
                        Err(_) => {}
                    }
                }

                if deleted_count > 0 {
                    if let Ok(mut logs) = logs_clone.lock() {
                        logs.push_str(&format!("已清理: {}\n", dot_ext));
                    }
                }
            }

            refresh_system_icons();

            if let Ok(mut logs) = logs_clone.lock() {
                logs.push_str(">> 已发送系统刷新信号，图标应已更新。\n");
                logs.push_str(&format!("--- {} 处理完成 ---\n", label));
            }
        });
    }
}

impl eframe::App for WinMediaUnbind {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("项目地址").clicked() {
                    let _ = open::that("https://github.com/NeetheCheeBao/WinMediaUnbind");
                    self.log("正在打开项目地址...");
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 30.0) / 4.0;

                if ui.add(egui::Button::new("删除视频格式关联").min_size([width, 30.0].into())).clicked() {
                    self.run_cleaning("视频格式", VIDEO_EXTS);
                }
                if ui.add(egui::Button::new("删除音频格式关联").min_size([width, 30.0].into())).clicked() {
                    self.run_cleaning("音频格式", AUDIO_EXTS);
                }
                if ui.add(egui::Button::new("删除播放列表格式关联").min_size([width, 30.0].into())).clicked() {
                    self.run_cleaning("播放列表格式", PLAYLIST_EXTS);
                }
                
                let btn = egui::Button::new("全部解除关联").fill(egui::Color32::from_rgb(180, 0, 0)).min_size([width, 30.0].into());
                if ui.add(btn).clicked() {
                    self.run_cleaning("所有格式", VIDEO_EXTS);
                    self.run_cleaning("所有格式", AUDIO_EXTS);
                    self.run_cleaning("所有格式", PLAYLIST_EXTS);
                }
            });
            ui.add_space(5.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("日志");
            ui.separator();
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let logs = self.logs.lock().unwrap();
                    ui.add(
                        egui::TextEdit::multiline(&mut logs.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                    );
                });
        });
    }
}