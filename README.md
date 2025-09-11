# Elibz2Kicad

Elibz2Kicad 是一个将立创商城的 .elibz 格式电子元件库文件转换为 kicad_mod 格式的工具。该工具可以帮助电子工程师和爱好者将立创商城的元件库导入到 KiCad 中使用。

仅限用于将封装导入到 KiCad 6.0 版本

## 功能特点

- 将立创商城的 .elibz 文件转换为 KiCad 可用的封装库
- 支持批量转换多个 .elibz 文件
- 图形化用户界面，操作简单直观
- 跨平台支持


## 使用方法

1. 启动 Elibz2Kicad 应用程序
2. 点击"选择文件"按钮选择要转换的 .elibz 文件
3. 点击"选择目录"按钮选择输出目录
4. 点击"进行转换"按钮开始转换过程
5. 转换完成后，您可以在输出目录中找到生成的 KiCad 封装文件

## 开发

### 技术栈

- 前端：React + TypeScript + Ant Design
- 后端：Rust + Tauri
- 构建工具：Vite

### 本地开发

1. 克隆仓库：
   ```bash
   git clone https://github.com/your-username/elibz2kicad.git
   cd elibz2kicad
   ```

2. 安装依赖：
   ```bash
   npm install
   ```

3. 运行开发服务器：
   ```bash
   npm run tauri dev
   ```

### 构建

```bash
npm run tauri build
```

## 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进这个项目。

## 许可证

本项目采用 MIT 许可证。详情请见 [LICENSE](LICENSE) 文件。
