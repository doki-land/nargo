use nargo_ir::IRModule;
use nargo_types::Result;

/// WebAssembly 写入器
#[derive(Default)]
pub struct WasmWriter {
    inner: Vec<u8>,
}

impl WasmWriter {
    /// 创建新的 WebAssembly 写入器
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// 写入字节
    pub fn write(&mut self, bytes: &[u8]) {
        self.inner.extend_from_slice(bytes);
    }

    /// 完成写入并返回结果
    pub fn finish(self) -> Vec<u8> {
        self.inner
    }
}

/// WebAssembly 后端
pub struct WasmBackend {
    /// 是否启用优化
    pub optimize: bool,
}

impl WasmBackend {
    /// 创建新的 WebAssembly 后端
    pub fn new(optimize: bool) -> Self {
        Self { optimize }
    }

    /// 生成 WebAssembly 代码
    pub fn generate(&self, ir: &IRModule) -> Result<Vec<u8>> {
        let mut writer = WasmWriter::new();

        // 1. 生成 WebAssembly 模块头部
        writer.write(&[0x00, 0x61, 0x73, 0x6d]); // magic number
        writer.write(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // 2. 生成类型段
        self.generate_type_section(&mut writer, ir)?;

        // 3. 生成函数段
        self.generate_function_section(&mut writer, ir)?;

        // 4. 生成内存段
        self.generate_memory_section(&mut writer, ir)?;

        // 5. 生成导出段
        self.generate_export_section(&mut writer, ir)?;

        // 6. 生成代码段
        self.generate_code_section(&mut writer, ir)?;

        Ok(writer.finish())
    }

    /// 生成类型段
    fn generate_type_section(&self, writer: &mut WasmWriter, _ir: &IRModule) -> Result<()> {
        // 类型段 ID: 1
        writer.write(&[0x01]);

        // 类型数量: 1 (main 函数类型)
        writer.write(&[0x01]);

        // 函数类型: 0x60
        writer.write(&[0x60]);

        // 参数数量: 0
        writer.write(&[0x00]);

        // 返回值数量: 0
        writer.write(&[0x00]);

        Ok(())
    }

    /// 生成函数段
    fn generate_function_section(&self, writer: &mut WasmWriter, _ir: &IRModule) -> Result<()> {
        // 函数段 ID: 3
        writer.write(&[0x03]);

        // 函数数量: 1
        writer.write(&[0x01]);

        // 函数类型索引: 0
        writer.write(&[0x00]);

        Ok(())
    }

    /// 生成内存段
    fn generate_memory_section(&self, writer: &mut WasmWriter, _ir: &IRModule) -> Result<()> {
        // 内存段 ID: 5
        writer.write(&[0x05]);

        // 内存数量: 1
        writer.write(&[0x01]);

        // 内存标志: 0 (无限制)
        writer.write(&[0x00]);

        // 初始页大小: 1 (64KB)
        writer.write(&[0x01]);

        Ok(())
    }

    /// 生成导出段
    fn generate_export_section(&self, writer: &mut WasmWriter, _ir: &IRModule) -> Result<()> {
        // 导出段 ID: 7
        writer.write(&[0x07]);

        // 导出数量: 2 (memory 和 main)
        writer.write(&[0x02]);

        // 导出内存
        writer.write(&[0x06]); // 名称长度: "memory".len()
        writer.write(b"memory");
        writer.write(&[0x02]); // 导出类型: 内存
        writer.write(&[0x00]); // 内存索引: 0

        // 导出 main 函数
        writer.write(&[0x04]); // 名称长度: "main".len()
        writer.write(b"main");
        writer.write(&[0x00]); // 导出类型: 函数
        writer.write(&[0x00]); // 函数索引: 0

        Ok(())
    }

    /// 生成代码段
    fn generate_code_section(&self, writer: &mut WasmWriter, _ir: &IRModule) -> Result<()> {
        // 代码段 ID: 10
        writer.write(&[0x0a]);

        // 代码块大小: 11
        writer.write(&[0x0b]);

        // 函数数量: 1
        writer.write(&[0x01]);

        // 函数体大小: 8
        writer.write(&[0x08]);

        // 局部变量数量: 0
        writer.write(&[0x00]);

        // 指令: end (0x0b)
        writer.write(&[0x0b]);

        Ok(())
    }
}
