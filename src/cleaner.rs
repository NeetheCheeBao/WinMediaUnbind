//! 影音扩展名清单与注册表关联清理。

use crate::logger::Logger;
use crate::theme::Tone;
use crate::winffi;
use native_windows_gui as nwg;
use std::thread;
use std::time::Duration;
use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER};
use winreg::RegKey;

pub const VIDEO_EXTS: &[&str] = &[
    "avi", "wmv", "wmp", "wm", "asf", "mpg", "mpeg", "mpe", "m1v", "m2v", "mpv2", "mp2v", "ts",
    "tp", "tpr", "trp", "vob", "ifo", "ogm", "ogv", "mp4", "m4v", "m4p", "m4b", "3gp", "3gpp",
    "3g2", "3gp2", "mkv", "rm", "ram", "rmvb", "rpm", "flv", "mov", "qt", "nsv", "dpg", "m2ts",
    "m2t", "mts", "dvr-ms", "k3g", "skm", "evo", "nsr", "amv", "divx", "webm", "wtv", "f4v", "mxf",
];

pub const AUDIO_EXTS: &[&str] = &[
    "wav", "wma", "mpa", "mp2", "m1a", "m2a", "mp3", "ogg", "m4a", "aac", "mka", "ra", "flac",
    "ape", "mpc", "mod", "ac3", "eac3", "dts", "dtshd", "wv", "tak", "cda", "dsf", "tta", "aiff",
    "aif", "opus", "amr",
];

pub const PLAYLIST_EXTS: &[&str] = &[
    "asx", "m3u", "m3u8", "pls", "wvx", "wax", "wmx", "cue", "mpls", "mpl", "dpl", "xspf", "mpd",
];

/// 用户级文件关联所在的注册表路径。
const USER_FILE_EXTS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts";

/// 一组待处理的扩展名及其在日志中的名称。
pub type Group = (&'static str, &'static [&'static str]);

/// 在后台线程按顺序清理若干组扩展名。
/// 所有分组共用一个线程，避免并发写入注册表导致日志交错。
/// 全部处理结束后发出一次完成通知，供界面恢复按钮状态。
pub fn start_cleaning(groups: Vec<Group>, logger: Logger, finished: nwg::NoticeSender) {
    thread::spawn(move || {
        // 先备份再删除，备份失败则中止本次操作。
        let mut targets: Vec<&str> = Vec::new();
        for (_, exts) in groups.iter() {
            targets.extend_from_slice(exts);
        }
        let label = groups
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(" + ");

        let backup_path = match crate::backup::write_backup(&label, &targets) {
            Ok(path) => {
                let name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_default();
                logger.line(
                    format!("  已备份 {} 个格式到 {}", targets.len(), name),
                    Tone::System,
                );
                path
            }
            Err(err) => {
                logger.line(format!("  备份失败，已中止本次操作：{}", err), Tone::Header);
                finished.notice();
                return;
            }
        };

        // 备份必须与当前注册表一致才继续，否则宁可不清理。
        if let Err(err) = crate::backup::verify_backup(&backup_path, &targets) {
            logger.line(
                format!("  备份校验失败，已中止本次操作：{}", err),
                Tone::Header,
            );
            finished.notice();
            return;
        }
        logger.line("  备份校验通过，与当前注册表一致。", Tone::System);

        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        for (label, exts) in groups {
            logger.line(format!("── 开始处理：{} ──", label), Tone::Header);

            for ext in exts {
                let dot_ext = format!(".{}", ext);

                // 是否报告“已清理”只看文件类型注册（HKCR），它才代表关联本身。
                // 用户级 FileExts 一并清理，但那里 Windows 会自动重建 UserChoice，
                // 若把它算进结果，就会出现同一项被反复报告已清理的假象。
                let cleared = clear_key(&hkcr, &dot_ext);
                if let Ok(file_exts) = hkcu.open_subkey(USER_FILE_EXTS) {
                    clear_key(&file_exts, &dot_ext);
                }
                if cleared {
                    logger.line(format!("  已清理 {}", dot_ext), Tone::Success);
                }
            }

            logger.line(format!("── {} 处理完成 ──", label), Tone::Done);
        }

        winffi::refresh_associations();
        logger.line(">> 已发送系统刷新信号，图标与关联应已更新。", Tone::System);

        // 清理完成后再保持 1 秒禁用，随后恢复按钮。
        thread::sleep(Duration::from_millis(1000));
        finished.notice();
    });
}

/// 删除一个扩展名子键，返回是否真的清掉了内容。
/// 先确认键里确实有值或子键：已被删除但仍有进程持有句柄的键会以空壳形式存在，
/// 这类空壳不应当算作“已清理”，否则会出现清理后再次清理的假象。
fn clear_key(parent: &RegKey, sub: &str) -> bool {
    let Ok(key) = parent.open_subkey(sub) else {
        return false;
    };
    let has_content = key.enum_values().next().is_some() || key.enum_keys().next().is_some();
    drop(key);

    if !has_content {
        return false;
    }
    if parent.delete_subkey_all(sub).is_err() {
        return false;
    }
    // 内容清空后可能留下空键，尽力再删一次；失败不影响结果。
    let _ = parent.delete_subkey(sub);

    // 复查：被其他进程持有句柄的键会以“挂起删除”状态继续存在（例如播放器占着
    // UserChoice），这种情况下关联其实没解除，不能算作已清理，否则会反复报同一项。
    parent.open_subkey(sub).is_err()
}
