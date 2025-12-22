# WinMediaUnbind

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**WinMediaUnbind** 是一个基于 Rust 和 egui 开发的轻量级 Windows 工具，可以一键删除常见的视频、音频和播放列表文件的默认程序关联。

> **🎯 项目原由**
> - 有的用户的影音格式被强行关联了，用户希望自己的影音软件关联上这些格式却无法操作。至此，本项目诞生，用于强力删除影音格式文件的关联。

![IMG](/IMG/1.png)

* **彻底删除**：递归清理 `HKEY_CLASSES_ROOT` 和 `HKEY_CURRENT_USER` 下的 `FileExts` 注册表项。
* **即时生效**：内置系统 API 调用，清理后自动刷新 Windows 图标缓存，无需重启资源管理器或电脑。

## 📂 涉及的文件格式
>
>  ### 视频
>  ```text
>  avi wmv wmp wm asf mpg mpeg mpe m1v m2v mpv2 mp2v ts tp tpr trp vob ifo ogm ogv mp4 m4v m4p m4b 3gp 3gpp 3g2 3gp2 mkv rm ram rmvb rpm flv mov qt nsv dpg m2ts m2t mts dvr-ms k3g skm evo nsr amv divx webm wtv f4v mxf
>  ```
>
>  ### 音频
>  ```text
>  wav wma mpa mp2 m1a m2a mp3 ogg m4a aac mka ra flac ape mpc mod ac3 eac3 dts dtshd wv tak cda dsf tta aiff aif opus
>  ```
>
>  ### 播放列表
>  ```text
>  asx m3u m3u8 pls wvx wax wmx cue mpls mpl dpl xspf mpd
>  ```

## 🛠️ 本地构建

如果你想自己编译本项目，请确保已安装 [Rust](https://www.rust-lang.org/tools/install)。

* **编译**
```bash
cargo build --release
 ```

* **生成文件**

编译完成后，可执行文件位于 `target/release/WinMediaUnbind.exe`。

## ⚠️ 警告

本程序会删除 Windows 注册表中关于文件扩展名的关联项。虽然开发过程中已经过测试，但**修改注册表始终存在风险**。

* 作者不对因使用本软件造成的任何数据丢失或系统不稳定承担责任。
* 建议在运行前备份重要数据。
* 清理后，双击相关文件时 Windows 会提示“您想如何打开这个文件？”，这是正常现象。

## ⬇️ 下载使用

前往 [Releases](https://github.com/NeetheCheeBao/WinMediaUnbind/releases) 下载最新版、

## ⚖️ 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。
