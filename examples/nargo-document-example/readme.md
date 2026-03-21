# Nargo Document 使用示例

这个示例展示了如何使用 Nargo Document 生成文档。

## 功能演示

1. **配置加载** - 加载和解析 nargodoc 配置文件
2. **文档生成** - 使用 Generator 生成 HTML 文档

## 项目结构

```
nargo-document-example/
├── Cargo.toml              # 项目配置文件
├── README.md               # 说明文档
├── nargodoc.config.toml    # Nargo Document 配置文件
├── src/
│   └── main.rs             # 主程序代码
└── docs/
    └── index.md            # 示例文档
```

## 运行示例

### 构建项目

```bash
cargo build
```

### 运行示例

```bash
cargo run
```

### 运行测试

```bash
cargo test
```

## 使用 Nargo Document CLI

### 构建文档

```bash
cd ../../compilers/nargo-document
cargo run -- build -c ../../examples/nargo-document-example/nargodoc.config.toml -o ../../examples/nargo-document-example/dist
```

### 启动开发服务器

```bash
cargo run -- dev -c ../../examples/nargo-document-example/nargodoc.config.toml -p 3000
```

## 配置说明

### nargodoc.config.toml

主要配置项：

- `general` - 通用配置（标题、作者、描述）
- `output` - 输出配置（目录、格式）
- `server` - 服务器配置（端口、主机）
- `plugins` - 插件配置
- `theme` - 主题配置

### 内置插件

- `katex` - 数学公式渲染
- `mermaid` - 图表渲染
- `shiki` - 代码高亮
- `container` - 自定义容器

## 代码说明

### 加载配置

```rust
let config_path = Path::new("nargodoc.config.toml");
let config = if config_path.exists() {
    Config::load(config_path)?
} else {
    Config::default()
};
```

### 生成文档

```rust
let mut generator = Generator::new(config);
generator.generate(input_dir, output_dir)?;
```

## 依赖项

- `nargo-document` - Nargo 文档生成器
- `nargo-types` - Nargo 类型系统
- `tokio` - 异步运行时
- `anyhow` - 错误处理

## 学习要点

1. 如何配置 Nargo Document
2. 如何加载和解析配置文件
3. 如何使用 Generator 生成文档
4. 如何编写 Markdown 文档
5. 如何使用插件扩展功能
