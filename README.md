# MyPhoto

一个ai编程尝试，几乎全部使用vibe coding实现。

一个专为摄影师打造的桌面照片管理工具，基于 Tauri + Vue 构建，支持 Windows 和 macOS 双平台。

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri)
![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vue.js)
![Tailwind](https://img.shields.io/badge/Tailwind-4.0-06B6D4?logo=tailwindcss)

## ✨ 功能特性

### 📁 智能文件夹管理
- **目录树浏览**：递归展示文件夹结构，支持展开/折叠
- **快速导航**：右键菜单支持设置根目录、刷新、全部折叠
- **智能分组**：自动识别同名 RAW + JPG 文件，合并展示避免重复

### 🖼️ 高效选图工作流
- **快速预览**：点击缩略图即时查看原图，支持 RAW/JPG 切换
- **多选操作**：Shift + 左键批量选择，Delete 键快速删除（移至回收站）
- **键盘快捷键**：0-5 数字键快速为照片打分评级

### 📊 专业 EXIF 信息
- **完整元数据**：拍摄时间、相机型号、镜头、焦段、快门、光圈、ISO
- **地理位置**：支持批量添加/修改 GPS 坐标
- **评分系统**：星级评分直接写入 EXIF，跨软件兼容

### 🎨 格式支持
- **通用格式**：JPEG、PNG
- **RAW 格式**：支持富士 RAF 格式（内置解码器）。由于我没有其他品牌的相机，暂时没有支持其他raw格式的计划。
- **智能识别**：自动检测并关联同名不同格式的文件

### 🌓 界面主题
- **深浅色模式**：支持浅色/深色主题切换，自动适配系统偏好
- **现代化 UI**：基于 shadcn-vue 组件库，简洁优雅的设计语言
- **流畅动画**：主题切换、交互反馈均有平滑过渡效果

### ⚡ 性能优化
- **虚拟滚动**：大量图片流畅浏览不卡顿
- **异步加载**：图片加载不影响 UI 响应
- **缩略图缓存**：快速生成和读取缩略图

## 🚀 快速开始

### 环境要求
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/) 1.70+
- [pnpm](https://pnpm.io/) 8+

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖（自动执行）
cd src-tauri
cargo build
```

### 开发模式

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建发布版本

```bash
# 构建生产版本
pnpm tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

## 📸 界面预览

> 应用采用三栏式布局设计：
> - **左侧**：文件夹目录树 + 缩略图浏览器
> - **中间**：大图预览区（支持 RAW/JPG 切换）
> - **右侧**：EXIF 信息面板 + 评分控件

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3.5 + TypeScript 5.6 |
| 桌面框架 | Tauri 2.0 |
| 构建工具 | Vite 6 |
| UI 组件 | shadcn-vue + Tailwind CSS 4 |
| 图标库 | Lucide Vue |
| 状态管理 | Vue Composition API |
| 图像处理 | Rust image crate |
| EXIF 处理 | kamadak-exif / little_exif |

### 前端技术亮点

- **shadcn-vue**: 采用流行的 Vue 组件库，提供一致的设计语言和优秀的可访问性支持
- **Tailwind CSS 4**: 使用最新的原子化 CSS 框架，支持 CSS 变量主题系统
- **深浅色主题**: 基于 CSS 变量实现的主题系统，支持手动切换和自动跟随系统偏好
- **虚拟滚动**: 使用 vue-virtual-scroller 实现大量图片的流畅浏览
- **响应式布局**: 三栏式自适应布局，支持拖拽调整面板宽度

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| `0-5` | 为选中图片打分（0 = 无评分） |
| `Delete` | 删除选中图片（移至回收站） |
| `Shift + 左键` | 多选图片 |
| `左键` | 选中图片 / 预览 |

## 📝 开发计划

查看 [TODO.md](./specs/TODO.md) 了解当前开发进度和计划功能。
