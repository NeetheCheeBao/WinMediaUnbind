//! 直接引用的少量 Win32 接口。
//! 只声明本项目实际用到的函数，避免为几个调用额外引入依赖。

use std::ffi::c_void;
use std::ptr;

#[link(name = "shell32")]
extern "system" {
    fn SHChangeNotify(w_event_id: i32, u_flags: u32, item1: *const c_void, item2: *const c_void);
}

#[link(name = "user32")]
extern "system" {
    fn SendMessageW(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: *mut c_void, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: *mut c_void, index: i32, value: isize) -> isize;
    fn SetWindowPos(
        hwnd: *mut c_void,
        after: *mut c_void,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    fn ClientToScreen(hwnd: *mut c_void, point: *mut Point) -> i32;
    fn GetWindowRect(hwnd: *mut c_void, rect: *mut Rect) -> i32;
    fn GetSystemMetrics(index: i32) -> i32;
    fn GetDC(hwnd: *mut c_void) -> *mut c_void;
    fn HideCaret(hwnd: *mut c_void) -> i32;
    fn ShowWindow(hwnd: *mut c_void, cmd: i32) -> i32;
    fn EnableWindow(hwnd: *mut c_void, enable: i32) -> i32;
    fn ReleaseCapture() -> i32;
    fn PostMessageW(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn LoadImageW(
        instance: *mut c_void,
        name: *const u16,
        image_type: u32,
        cx: i32,
        cy: i32,
        load: u32,
    ) -> *mut c_void;
    fn ReleaseDC(hwnd: *mut c_void, dc: *mut c_void) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetDeviceCaps(dc: *mut c_void, index: i32) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLocalTime(out: *mut SystemTime);
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

/// 与 GetLocalTime 对应的本地时间结构。
#[repr(C)]
pub struct SystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

/// 返回本地时间的年、月、日、时、分、秒。
pub fn local_datetime() -> (u32, u32, u32, u32, u32, u32) {
    let mut time = SystemTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe {
        GetLocalTime(&mut time);
    }
    (
        time.year as u32,
        time.month as u32,
        time.day as u32,
        time.hour as u32,
        time.minute as u32,
        time.second as u32,
    )
}

/// 返回本地时间的时、分、秒，用于日志时间戳。
pub fn local_hms() -> (u32, u32, u32) {
    let (_, _, _, hour, minute, second) = local_datetime();
    (hour, minute, second)
}

/// 与 ClientToScreen 对应的屏幕坐标结构。
#[repr(C)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// 与 GetWindowRect 对应的窗口矩形。
#[repr(C)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// 与 EM_EXGETSEL 对应的字符区间。
#[repr(C)]
pub struct CharRange {
    pub min: i32,
    pub max: i32,
}

/// 与 EM_GETTEXTLENGTHEX 对应的参数结构。
#[repr(C)]
pub struct TextLengthEx {
    pub flags: u32,
    pub codepage: u32,
}

const SHCNE_ASSOCCHANGED: i32 = 0x0800_0000;
const SHCNF_IDLIST: u32 = 0;
const LOGPIXELSX: i32 = 88;
const DEFAULT_DPI: f64 = 96.0;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const WS_CAPTION: isize = 0x00C0_0000;
const WS_SYSMENU: isize = 0x0008_0000;
const WS_THICKFRAME: isize = 0x0004_0000;
const WS_MINIMIZEBOX: isize = 0x0002_0000;
const WS_MAXIMIZEBOX: isize = 0x0001_0000;
const WS_BORDER: isize = 0x0080_0000;
const WS_EX_CLIENTEDGE: isize = 0x0000_0200;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const SM_CXSMICON: i32 = 49;
const SM_CYSMICON: i32 = 50;
const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const ICON_BIG: usize = 1;
const IMAGE_ICON: u32 = 1;
const LR_DEFAULTSIZE: u32 = 0x0040;
const LR_SHARED: u32 = 0x8000;
/// 程序图标在资源脚本中的编号。
const APP_ICON_ID: usize = 1;

pub const EM_REPLACESEL: u32 = 0x00C2;
pub const EM_EXGETSEL: u32 = 0x0434;
pub const EM_GETTEXTLENGTHEX: u32 = 0x045F;
/// 要求按字符计数并使用精确算法。
const GTL_PRECISE: u32 = 0x0002;
const GTL_NUMCHARS: u32 = 0x0008;
const CP_UNICODE: u32 = 1200;
pub const EM_SETTARGETDEVICE: u32 = 0x0448;
pub const EM_SETMARGINS: u32 = 0x00D3;
pub const EC_LEFTMARGIN: usize = 0x0001;
pub const EC_RIGHTMARGIN: usize = 0x0002;
pub const EM_SCROLL: u32 = 0x0115;
pub const EM_LINESCROLL: u32 = 0x00B6;
pub const SB_BOTTOM: usize = 7;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const LB_GETCOUNT: u32 = 0x018B;
pub const LB_GETCURSEL: u32 = 0x0188;
pub const LB_SETCURSEL: u32 = 0x0186;
pub const LB_GETTOPINDEX: u32 = 0x018E;
pub const LB_GETITEMHEIGHT: u32 = 0x01A1;
pub const TTM_SETMAXTIPWIDTH: u32 = 0x0418;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_NCLBUTTONDOWN: u32 = 0x00A1;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_COMMAND: u32 = 0x0111;
pub const HTCLIENT: isize = 1;
pub const HTCAPTION: isize = 2;

/// 向系统广播文件关联已变化，使资源管理器立即刷新图标与打开方式。
pub fn refresh_associations() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, ptr::null(), ptr::null());
    }
}

/// 读取主显示器的 DPI 缩放比例，以 96 DPI 为 1.0。
/// 100% 缩放时返回 1.0，界面尺寸保持原值，不做任何放大或缩小。
pub fn system_scale() -> f64 {
    unsafe {
        let dc = GetDC(ptr::null_mut());
        if dc.is_null() {
            return 1.0;
        }
        let dpi = GetDeviceCaps(dc, LOGPIXELSX);
        ReleaseDC(ptr::null_mut(), dc);
        if dpi > 0 {
            dpi as f64 / DEFAULT_DPI
        } else {
            1.0
        }
    }
}

/// 清除系统标题栏及最小化、最大化、关闭按钮。
/// 直接改写已创建窗口的样式位并通知系统重算非客户区，
/// 不依赖界面库创建窗口时传入的 flags 参数。
pub fn strip_window_decorations(hwnd: *mut c_void) -> bool {
    const DECORATIONS: isize =
        WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;

    unsafe {
        let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED;
        for _ in 0..2 {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            if style & WS_CAPTION == 0 {
                return true;
            }
            SetWindowLongPtrW(hwnd, GWL_STYLE, style & !DECORATIONS);
            SetWindowPos(hwnd, ptr::null_mut(), 0, 0, 0, 0, flags);
        }
        (GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CAPTION) == 0
    }
}

/// 去掉控件自带的立体边框，让背景色直接铺满控件矩形。
/// 边框属于非客户区，必须在改写样式后通知系统重算。
pub fn strip_control_border(hwnd: *mut c_void) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(hwnd, GWL_STYLE, style & !WS_BORDER);
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !WS_EX_CLIENTEDGE);
        SetWindowPos(
            hwnd,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// 把窗口按指定物理尺寸移动到屏幕中央。
pub fn center_window(hwnd: *mut c_void, width: i32, height: i32) {
    unsafe {
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        SetWindowPos(
            hwnd,
            ptr::null_mut(),
            (screen_w - width) / 2,
            (screen_h - height) / 2,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// 返回窗口矩形在屏幕坐标系下的四条边。
pub fn window_rect(hwnd: *mut c_void) -> (i32, i32, i32, i32) {
    let mut rect = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }
    (rect.left, rect.top, rect.right, rect.bottom)
}

/// 把窗口客户区坐标换算为屏幕坐标，用于定位弹出菜单。
pub fn client_to_screen(hwnd: *mut c_void, x: i32, y: i32) -> (i32, i32) {
    let mut point = Point { x, y };
    unsafe {
        ClientToScreen(hwnd, &mut point);
    }
    (point.x, point.y)
}

/// 同步发送一条窗口消息。
pub fn send_message(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> isize {
    unsafe { SendMessageW(hwnd, msg, wparam, lparam) }
}

/// 显示或隐藏窗口。
pub fn show_window(hwnd: *mut c_void, visible: bool) {
    unsafe {
        ShowWindow(hwnd, if visible { 5 } else { 0 });
    }
}

/// 设置窗口的属主窗口：不出现在任务栏，并随属主一起最小化、还原。
pub fn set_window_owner(hwnd: *mut c_void, owner: *mut c_void) {
    const GWLP_HWNDPARENT: i32 = -8;
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner as isize);
    }
}

/// 交给系统执行窗口拖动。
/// 子控件命中测试返回 HTCAPTION 不会真正触发拖动，必须改为模拟标题栏按下。
pub fn start_window_drag(hwnd: *mut c_void) {
    unsafe {
        ReleaseCapture();
        SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
    }
}

/// 启用或禁用控件。
pub fn enable_window(hwnd: *mut c_void, enabled: bool) {
    unsafe {
        EnableWindow(hwnd, if enabled { 1 } else { 0 });
    }
}

/// 投递一条异步消息，供控件处理完自身逻辑后再执行动作。
pub fn post_message(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) {
    unsafe {
        PostMessageW(hwnd, msg, wparam, lparam);
    }
}

/// 隐藏控件自身的文本插入符。
/// 插入符的高度由整行行高决定，系统不提供缩小接口，只读日志区直接不显示。
pub fn hide_caret(hwnd: *mut c_void) {
    unsafe {
        HideCaret(hwnd);
    }
}

/// 限制悬浮提示的最大宽度，超长文本由系统自动换行。
pub fn set_tooltip_width(hwnd: *mut c_void, width: i32) {
    unsafe {
        SendMessageW(hwnd, TTM_SETMAXTIPWIDTH, 0, width as isize);
    }
}

/// 把程序自身携带的图标设置到窗口，任务栏与 Alt+Tab 使用该图标。
pub fn set_window_icon(hwnd: *mut c_void) {
    let name = APP_ICON_ID as *const u16;
    unsafe {
        // 自定义图标必须给出模块句柄，传空句柄取不到资源。
        let instance = GetModuleHandleW(ptr::null());
        if instance.is_null() {
            return;
        }

        let big = LoadImageW(
            instance,
            name,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        );
        if !big.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG, big as isize);
        }

        let small = LoadImageW(
            instance,
            name,
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_SHARED,
        );
        if !small.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL, small as isize);
        }
    }
}

/// 读取当前选区的起止位置，单位与 EM_SETSEL 一致。
/// 段落分隔符在此单位下只占一个字符，因此不能改用 WM_GETTEXTLENGTH 推算位置。
pub fn selection_range(hwnd: *mut c_void) -> (u32, u32) {
    let mut range = CharRange { min: 0, max: 0 };
    unsafe {
        SendMessageW(
            hwnd,
            EM_EXGETSEL,
            0,
            &mut range as *mut CharRange as isize,
        );
    }
    (range.min.max(0) as u32, range.max.max(0) as u32)
}

/// 返回文本的字符总数，段落分隔符按一个字符计。
/// 该计数与选区位置使用同一套单位，可直接作为文末位置。
pub fn text_length(hwnd: *mut c_void) -> u32 {
    let params = TextLengthEx {
        flags: GTL_PRECISE | GTL_NUMCHARS,
        codepage: CP_UNICODE,
    };
    let length = unsafe {
        SendMessageW(
            hwnd,
            EM_GETTEXTLENGTHEX,
            &params as *const TextLengthEx as usize,
            0,
        )
    };
    length.max(0) as u32
}
