use std::io;
use std::path::{Path, PathBuf};

/// 应用清单：公共控件 6.0、管理员权限、按显示器 DPI 感知。
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <dependency>
        <dependentAssembly>
            <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
        </dependentAssembly>
    </dependency>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    <application xmlns="urn:schemas-microsoft-com:asm.v3">
        <windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </windowsSettings>
    </application>
</assembly>
"#;

/// 查找 rc.exe 所在目录。
/// 部分环境不允许查询注册表，无法由 winres 自行定位 Windows SDK，
/// 因此在常见安装位置中显式探测，并优先选择版本号最高的工具集。
#[cfg(windows)]
fn find_rc_toolkit() -> Option<String> {
    if let Ok(path) = std::env::var("WINRES_TOOLKIT_PATH") {
        return Some(path);
    }

    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("WindowsSdkDir") {
        roots.push(PathBuf::from(dir).join("bin"));
    }
    for drive in b'C'..=b'Z' {
        roots.push(PathBuf::from(format!(r"{}:\Windows Kits\10\bin", drive as char)));
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin"));
    roots.push(PathBuf::from(r"C:\Program Files\Windows Kits\10\bin"));

    let mut best: Option<(Vec<u64>, PathBuf)> = None;
    for root in roots {
        let entries = match std::fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let version: Vec<u64> = name.split('.').filter_map(|part| part.parse().ok()).collect();
            let dir = entry.path().join("x64");
            if version.is_empty() || !dir.join("rc.exe").is_file() {
                continue;
            }
            if best.as_ref().map_or(true, |(current, _)| version > *current) {
                best = Some((version, dir));
            }
        }
    }

    best.map(|(_, dir)| dir.to_string_lossy().into_owned())
}

/// 读取 version.ini 中的程序名称与版本。
/// 文件缺失或某项未填写时退回内置默认值，保证构建不中断。
fn read_version_ini() -> (String, String) {
    let mut name = String::from("WinMediaUnbind");
    let mut version = String::from("0.0.0");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let path = PathBuf::from(manifest_dir).join("version.ini");
    let text = std::fs::read_to_string(&path).unwrap_or_default();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"').to_string();
            match key.trim().to_ascii_lowercase().as_str() {
                "name" => name = value,
                "version" => version = value,
                _ => {}
            }
        }
    }

    (name, version)
}

/// 生成资源脚本：应用清单与程序图标，不写入任何版本信息。
/// 版本信息会让文件属性出现“文件版本”，因此保持为空。
#[cfg(windows)]
fn write_resource_script(out_dir: &Path) -> io::Result<PathBuf> {
    let manifest_path = out_dir.join("app.manifest");
    std::fs::write(&manifest_path, MANIFEST)?;

    let manifest_ref = manifest_path.to_string_lossy().replace('\\', "/");
    let icon_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default())
        .join("res")
        .join("icon.ico");
    let icon_ref = icon_path.to_string_lossy().replace('\\', "/");
    let script = format!("1 24 \"{manifest_ref}\"\n1 ICON \"{icon_ref}\"\n");

    let script_path = out_dir.join("app.rc");
    std::fs::write(&script_path, script)?;
    Ok(script_path)
}

fn main() -> io::Result<()> {
    let (name, version) = read_version_ini();
    println!("cargo:rustc-env=APP_NAME={}", name);
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rerun-if-changed=version.ini");
    println!("cargo:rerun-if-changed=res/icon.ico");

    #[cfg(windows)]
    {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let script = write_resource_script(&out_dir)?;

        let mut res = winres::WindowsResource::new();
        if let Some(toolkit) = find_rc_toolkit() {
            res.set_toolkit_path(&toolkit);
        }
        res.set_resource_file(&script.to_string_lossy());
        res.compile()?;
    }

    Ok(())
}
