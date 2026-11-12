//! 清理前的注册表备份。
//! 每次操作导出一份 INI，记录将被删除的键与值，便于日后恢复。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::{RegKey, RegValue};

/// 备份目录名，位于程序所在目录下。
const BACKUP_DIR: &str = "WinMediaUnbinBackup";
/// 用户级文件关联所在的注册表路径。
const USER_FILE_EXTS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts";
/// 多值字符串内部的分隔符。
const MULTI_SEP: char = '\u{1f}';

/// 备份目录的完整路径；无法定位程序目录时返回 None。
pub fn backup_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join(BACKUP_DIR))
}

/// 导出一组扩展名对应的注册表键，返回备份文件路径。
pub fn write_backup(label: &str, exts: &[&str]) -> io::Result<PathBuf> {
    let dir = backup_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "无法定位程序所在目录"))?;
    fs::create_dir_all(&dir)?;

    let (year, month, day, hour, minute, second) = crate::winffi::local_datetime();
    let file = dir.join(format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}.ini",
        year, month, day, hour, minute, second
    ));

    let mut text = String::new();
    text.push_str("[meta]\n");
    text.push_str(&format!("operation={}\n", label));
    text.push_str(&format!(
        "created={:04}-{:02}-{:02} {:02}:{:02}:{:02}\n",
        year, month, day, hour, minute, second
    ));
    text.push_str(&format!("extensions={}\n\n", exts.join(",")));
    text.push_str(&build_registry_body(exts));

    fs::write(&file, text)?;
    Ok(file)
}

/// 按当前注册表内容生成备份正文（不含 [meta] 段）。
fn build_registry_body(exts: &[&str]) -> String {
    let mut text = String::new();
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let file_exts = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(USER_FILE_EXTS)
        .ok();

    for ext in exts {
        let sub = format!(".{}", ext);
        if let Ok(key) = hkcr.open_subkey(&sub) {
            dump_key(&mut text, &key, &format!("HKCR\\{}", sub));
        }
        if let Some(root) = file_exts.as_ref() {
            if let Ok(key) = root.open_subkey(&sub) {
                dump_key(&mut text, &key, &format!("HKCU\\FileExts\\{}", sub));
            }
        }
    }

    text
}

/// 校验备份文件与当前注册表是否一致：重新导出一次并比对文件正文。
pub fn verify_backup(path: &Path, exts: &[&str]) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    let body = build_registry_body(exts);
    if content.ends_with(&body) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "备份内容与当前注册表不一致",
        ))
    }
}

/// 列出备份目录中的文件，最新在前；返回（显示文本, 文件路径）。
/// 显示文本形如“2026-11-12 23:40:12  视频格式”。
pub fn list_entries(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut rows: Vec<(String, String, PathBuf)> = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ini") {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stamp = format_stamp(&stem);
        let operation = read_operation(&path);
        let line = if operation.is_empty() {
            stamp
        } else {
            format!("{}  {}", stamp, operation)
        };
        rows.push((stem, line, path));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.into_iter().map(|(_, line, path)| (line, path)).collect()
}

/// 把 14 位时间戳格式化成可读形式，无法解析时原样返回。
fn format_stamp(stem: &str) -> String {
    if stem.len() == 14 && stem.chars().all(|c| c.is_ascii_digit()) {
        format!(
            "{}-{}-{} {}:{}:{}",
            &stem[0..4],
            &stem[4..6],
            &stem[6..8],
            &stem[8..10],
            &stem[10..12],
            &stem[12..14]
        )
    } else {
        stem.to_string()
    }
}

/// 读取备份文件 [meta] 中的 operation 字段。
fn read_operation(path: &Path) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("operation=") {
            return value.trim().to_string();
        }
    }
    String::new()
}

/// 递归导出一个键及其子键。
fn dump_key(out: &mut String, key: &RegKey, path: &str) {
    out.push_str(&format!("[{}]\n", path));
    for (name, value) in key.enum_values().flatten() {
        let label = if name.is_empty() { "@".to_string() } else { name };
        out.push_str(&format!("{}={}\n", label, encode(&value)));
    }
    out.push('\n');

    for name in key.enum_keys().flatten() {
        if let Ok(child) = key.open_subkey(&name) {
            dump_key(out, &child, &format!("{}\\{}", path, name));
        }
    }
}

/// 把注册表值编码成“类型:内容”。
fn encode(value: &RegValue) -> String {
    match value.vtype {
        REG_SZ => format!("sz:{}", escape(&from_utf16(&value.bytes))),
        REG_EXPAND_SZ => format!("expand:{}", escape(&from_utf16(&value.bytes))),
        REG_MULTI_SZ => {
            let text = from_utf16(&value.bytes);
            let items: Vec<String> = text
                .split('\0')
                .filter(|item| !item.is_empty())
                .map(escape)
                .collect();
            format!("multi:{}", items.join(&MULTI_SEP.to_string()))
        }
        REG_DWORD => format!("dword:{}", as_u32(&value.bytes)),
        REG_QWORD => format!("qword:{}", as_u64(&value.bytes)),
        REG_NONE => "none:".to_string(),
        _ => format!("bin:{}", to_hex(&value.bytes)),
    }
}

/// UTF-16LE 字节转字符串，并截掉结尾的终止符。
fn from_utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}

/// 转义会破坏 INI 行结构的字符。
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

fn as_u32(bytes: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    let len = bytes.len().min(4);
    buf[..len].copy_from_slice(&bytes[..len]);
    u32::from_le_bytes(buf)
}

fn as_u64(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);
    u64::from_le_bytes(buf)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// 恢复结果统计。
pub struct RestoreReport {
    /// 重建的注册表键数量。
    pub keys: usize,
    /// 写入的值数量。
    pub values: usize,
    /// 写入失败的值数量（例如被系统保护的 UserChoice）。
    pub failed: usize,
}

/// 把备份文件写回注册表。
/// 键按出现顺序重建（导出时父键先于子键），值按记录的类型原样写回。
pub fn restore_backup(path: &Path) -> io::Result<RestoreReport> {
    let text = fs::read_to_string(path)?;
    let mut report = RestoreReport {
        keys: 0,
        values: 0,
        failed: 0,
    };
    let mut current: Option<RegKey> = None;

    for raw in text.lines() {
        let line = raw.trim_end();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let name = &line[1..line.len() - 1];
            if name.eq_ignore_ascii_case("meta") {
                current = None;
                continue;
            }
            // 单个键写不进去（例如被系统保护的 UserChoice）不应中断整份恢复。
            match open_for_write(name) {
                Ok(key) => {
                    report.keys += 1;
                    current = Some(key);
                }
                Err(_) => {
                    report.failed += 1;
                    current = None;
                }
            }
            continue;
        }

        let Some(key) = current.as_ref() else {
            continue;
        };
        if let Some((name, value)) = line.split_once('=') {
            let value_name = if name == "@" { "" } else { name };
            match write_value(key, value_name, value) {
                Ok(()) => report.values += 1,
                Err(_) => report.failed += 1,
            }
        }
    }

    Ok(report)
}

/// 按节名打开（必要时创建）注册表键。
fn open_for_write(section: &str) -> io::Result<RegKey> {
    if let Some(rest) = section.strip_prefix("HKCR\\") {
        let (key, _) = RegKey::predef(HKEY_CLASSES_ROOT).create_subkey(rest)?;
        return Ok(key);
    }
    if let Some(rest) = section.strip_prefix("HKCU\\FileExts\\") {
        let path = format!("{}\\{}", USER_FILE_EXTS, rest);
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(&path)?;
        return Ok(key);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("无法识别的键路径：{}", section),
    ))
}

/// 写入一条“类型:内容”记录。
fn write_value(key: &RegKey, name: &str, encoded: &str) -> io::Result<()> {
    let (tag, payload) = encoded.split_once(':').unwrap_or((encoded, ""));
    let (bytes, vtype) = match tag {
        "sz" => (to_utf16_bytes(&unescape(payload)), REG_SZ),
        "expand" => (to_utf16_bytes(&unescape(payload)), REG_EXPAND_SZ),
        "multi" => {
            let mut bytes = Vec::new();
            for item in payload.split(MULTI_SEP).filter(|part| !part.is_empty()) {
                bytes.extend(to_utf16_bytes(&unescape(item)));
            }
            bytes.extend_from_slice(&[0, 0]);
            (bytes, REG_MULTI_SZ)
        }
        "dword" => (
            payload.trim().parse::<u32>().unwrap_or(0).to_le_bytes().to_vec(),
            REG_DWORD,
        ),
        "qword" => (
            payload.trim().parse::<u64>().unwrap_or(0).to_le_bytes().to_vec(),
            REG_QWORD,
        ),
        "none" => (Vec::new(), REG_NONE),
        _ => (from_hex(payload), REG_BINARY),
    };

    key.set_raw_value(name, &RegValue { bytes, vtype })
}

/// 文本转 UTF-16LE 字节，含结尾终止符。
fn to_utf16_bytes(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len() * 2 + 2);
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

/// 还原导出时的转义。
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// 十六进制字符串转字节。
fn from_hex(text: &str) -> Vec<u8> {
    let clean: Vec<u8> = text.bytes().filter(|b| b.is_ascii_hexdigit()).collect();
    clean
        .chunks(2)
        .filter_map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = pair.get(1).and_then(|b| (*b as char).to_digit(16)).unwrap_or(0);
            Some(((hi << 4) | lo) as u8)
        })
        .collect()
}
