use std::io;

#[cfg(windows)]
fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    // 强制请求管理员权限
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
    res.compile()
}

#[cfg(not(windows))]
fn main() {
    // 非Windows系统不做处理
}