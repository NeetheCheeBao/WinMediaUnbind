#![windows_subsystem = "windows"]

//! 程序入口：初始化界面环境，按系统 DPI 缩放后启动消息循环。

mod app;
mod backup;
mod cleaner;
mod logger;
mod terminal;
mod theme;
mod winffi;

use native_windows_gui as nwg;
use native_windows_gui::NativeUi;

/// 程序名称，来自 version.ini，由构建脚本注入。
pub const APP_NAME: &str = env!("APP_NAME");
/// 程序版本，来自 version.ini，由构建脚本注入。
pub const APP_VERSION: &str = env!("APP_VERSION");

fn main() {
    nwg::init().expect("初始化界面库失败");

    let scale = winffi::system_scale();
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Microsoft YaHei")
        .size((17.0 * scale).round() as u32)
        .build(&mut font)
        .ok();
    nwg::Font::set_global_default(Some(font));

    let app = app::App::build_ui(Default::default()).expect("构建主界面失败");
    app.initialize(scale);

    nwg::dispatch_thread_events();
}
