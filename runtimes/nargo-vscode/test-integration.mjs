#!/usr/bin/env node

/**
 * 测试 VS Code 插件和语言服务器集成
 */
import { execSync } from 'child_process';
import { existsSync, readFileSync } from 'fs';
import { join } from 'path';

const rootDir = join(process.cwd(), '..', '..');
const vscodeDir = join(rootDir, 'editors', 'vscode');
const lspDir = join(rootDir, 'compilers', 'nargo-lsp');

function runCommand(command, cwd) {
    try {
        console.log(`执行命令: ${command}`);
        const output = execSync(command, { cwd, encoding: 'utf8' });
        console.log(output);
        return true;
    } catch (error) {
        console.error(`命令执行失败: ${error.message}`);
        return false;
    }
}

function checkFileExists(filePath) {
    const exists = existsSync(filePath);
    console.log(`${filePath}: ${exists ? '✓ 存在' : '✗ 不存在'}`);
    return exists;
}

function testVSCodeExtension() {
    console.log('\n=== 测试 VS Code 插件 ===');
    
    // 检查必要文件是否存在
    const requiredFiles = [
        'package.json',
        'tsconfig.json',
        'src/extension.ts',
        'language-configuration.json',
        'syntaxes/nargo.tmLanguage.json'
    ];
    
    let allFilesExist = true;
    for (const file of requiredFiles) {
        const filePath = join(vscodeDir, file);
        if (!checkFileExists(filePath)) {
            allFilesExist = false;
        }
    }
    
    if (!allFilesExist) {
        console.log('✗ 缺少必要文件');
        return false;
    }
    
    // 检查依赖
    console.log('\n=== 检查依赖 ===');
    if (!existsSync(join(vscodeDir, 'node_modules'))) {
        console.log('安装依赖...');
        if (!runCommand('npm install', vscodeDir)) {
            return false;
        }
    } else {
        console.log('✓ 依赖已安装');
    }
    
    // 编译插件
    console.log('\n=== 编译插件 ===');
    if (!runCommand('npm run compile', vscodeDir)) {
        return false;
    }
    
    return true;
}

function testLanguageServer() {
    console.log('\n=== 测试语言服务器 ===');
    
    // 检查必要文件是否存在
    const requiredFiles = [
        'src/main.rs',
        'Cargo.toml'
    ];
    
    let allFilesExist = true;
    for (const file of requiredFiles) {
        const filePath = join(lspDir, file);
        if (!checkFileExists(filePath)) {
            allFilesExist = false;
        }
    }
    
    if (!allFilesExist) {
        console.log('✗ 缺少必要文件');
        return false;
    }
    
    // 构建语言服务器
    console.log('\n=== 构建语言服务器 ===');
    if (!runCommand('cargo build', lspDir)) {
        return false;
    }
    
    // 验证语言服务器可执行文件
    const lspExecutable = join(rootDir, 'target', 'debug', 'nargo-lsp.exe');
    if (!checkFileExists(lspExecutable)) {
        return false;
    }
    
    // 测试语言服务器启动
    console.log('\n=== 测试语言服务器启动 ===');
    try {
        const output = execSync(`${lspExecutable} --help`, { encoding: 'utf8', timeout: 5000 });
        console.log(output);
        console.log('✓ 语言服务器启动成功');
    } catch (error) {
        console.error(`语言服务器启动失败: ${error.message}`);
        return false;
    }
    
    return true;
}

function main() {
    console.log('=== Nargo IDE 集成测试 ===');
    console.log(`测试目录: ${rootDir}`);
    
    const vscodeResult = testVSCodeExtension();
    const lspResult = testLanguageServer();
    
    console.log('\n=== 测试结果 ===');
    console.log(`VS Code 插件: ${vscodeResult ? '✓ 通过' : '✗ 失败'}`);
    console.log(`语言服务器: ${lspResult ? '✓ 通过' : '✗ 失败'}`);
    
    if (vscodeResult && lspResult) {
        console.log('\n🎉 所有测试通过！IDE 集成配置成功。');
        console.log('\n下一步：');
        console.log('1. 在 VS Code 中打开扩展开发主机（按 F5）');
        console.log('2. 在新窗口中打开 Nargo 项目');
        console.log('3. 测试语法高亮、代码补全等功能');
    } else {
        console.log('\n❌ 测试失败，请检查错误信息并修复问题。');
        process.exit(1);
    }
}

if (import.meta.url === `file://${process.argv[1]}`) {
    main();
}