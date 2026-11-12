//! 彩色终端控件。
//! 基于富文本框实现蓝色背景、分级文字颜色与无滚动条的滚动交互。

use crate::theme::{Tone, TERMINAL_BG, TIMESTAMP_COLOR};
use crate::winffi;
use native_windows_gui as nwg;
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;

/// 滚轮与插入符的消息处理器标识，需大于界面库保留范围。
const SCROLL_HANDLER_ID: usize = 0x0001_0002;

/// 自定义消息：在控件处理完自身逻辑后隐藏插入符。
const WM_HIDE_CARET: u32 = 0x8001;

/// 初始化终端外观与滚动交互，返回需要保持存活的消息处理器。
/// 控件创建时不带滚动条样式，滚动完全由滚轮和方向键驱动。
pub fn setup(view: &nwg::RichTextBox) -> Option<nwg::RawEventHandler> {
    view.set_background_color(TERMINAL_BG);
    view.set_readonly(true);

    let hwnd = match view.handle.hwnd() {
        Some(handle) => handle as *mut c_void,
        None => return None,
    };

    // 去掉控件自带的立体边框，底色才能铺满整个矩形。
    winffi::strip_control_border(hwnd);

    winffi::send_message(hwnd, winffi::EM_SETTARGETDEVICE, 0, 0);

    let pad = (10.0 * winffi::system_scale()).round() as u32;
    let margins = (pad | (pad << 16)) as isize;
    winffi::send_message(
        hwnd,
        winffi::EM_SETMARGINS,
        winffi::EC_LEFTMARGIN | winffi::EC_RIGHTMARGIN,
        margins,
    );

    // 控件不显示滚动条，滚动完全由滚轮与键盘消息驱动。
    bind_input(view, hwnd)
}

/// 接管滚轮消息用于按行滚动视图，并抑制控件自带的插入符。
/// 方向键等按键不拦截，交还控件处理移动插入点与选择文本。
/// 处理器必须随窗口一起存活，因此由调用方保存返回值。
fn bind_input(view: &nwg::RichTextBox, hwnd: *mut c_void) -> Option<nwg::RawEventHandler> {
    nwg::bind_raw_event_handler(&view.handle, SCROLL_HANDLER_ID, move |_h, msg, wparam, _l| {
        if msg == WM_HIDE_CARET {
            winffi::hide_caret(hwnd);
            return Some(0);
        }

        if msg == winffi::WM_MOUSEWHEEL {
            let delta = ((wparam >> 16) & 0xFFFF) as i16 as i32;
            let notches = delta / 120;
            if notches != 0 {
                winffi::send_message(hwnd, winffi::EM_LINESCROLL, 0, (-notches * 3) as isize);
            }
            return Some(0);
        }

        // 获得焦点、点击与按键都会让控件重新显示插入符，延后一拍再隐藏。
        if matches!(
            msg,
            winffi::WM_SETFOCUS
                | winffi::WM_LBUTTONDOWN
                | winffi::WM_LBUTTONUP
                | winffi::WM_KEYDOWN
                | winffi::WM_KEYUP
        ) {
            winffi::post_message(hwnd, WM_HIDE_CARET, 0, 0);
        }

        None
    })
    .ok()
}

/// 追加一行带时间戳的日志：时间戳用固定灰蓝色，正文按级别着色。
pub fn write_line(view: &nwg::RichTextBox, stamp: &str, text: &str, tone: Tone) {
    let hwnd = match view.handle.hwnd() {
        Some(handle) => handle,
        None => return,
    };

    let line = format!("{}{}\r\n", stamp, text);
    let wide: Vec<u16> = OsStr::new(&line)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let stamp_len = stamp.encode_utf16().count() as u32;

    // 文末位置与选区必须使用同一套坐标：段落分隔符在选区坐标中只占一个字符，
    // 用 WM_GETTEXTLENGTH 推算会让着色区间逐行右移。
    let before = winffi::text_length(hwnd as _);

    // 把插入点明确移到文本末尾，并设定后续输入文字的颜色。
    view.set_selection(before..before);
    view.set_char_format(&nwg::CharFormat {
        text_color: Some(tone.rgb()),
        ..Default::default()
    });

    winffi::send_message(
        hwnd as _,
        winffi::EM_REPLACESEL,
        0,
        wide.as_ptr() as isize,
    );
    // 插入完成后插入点停在新增文本末尾，可直接作为着色终点。
    let after = winffi::selection_range(hwnd as _).1.max(before);

    // 时间戳与正文分别着色，时间戳保持统一的低饱和灰蓝。
    let stamp_end = (before + stamp_len).min(after);
    if stamp_end > before {
        view.set_selection(before..stamp_end);
        view.set_char_format(&nwg::CharFormat {
            text_color: Some(TIMESTAMP_COLOR),
            ..Default::default()
        });
    }
    if after > stamp_end {
        view.set_selection(stamp_end..after);
        view.set_char_format(&nwg::CharFormat {
            text_color: Some(tone.rgb()),
            ..Default::default()
        });
    }

    view.set_selection(after..after);
    winffi::send_message(hwnd as _, winffi::EM_SCROLL, winffi::SB_BOTTOM, 0);
}
