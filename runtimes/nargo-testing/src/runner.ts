import { TestSuite, TestResult, TestSummary, TestConfig, Reporter } from './types.js';
import { runTests as runCoreTests, getRootSuite, resetState } from './core.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

class DefaultReporter implements Reporter {
    onTestStart(suite: string, test: string): void {
        process.stdout.write(`  ${test}... `);
    }

    onTestResult(result: TestResult): void {
        switch (result.status) {
            case 'passed':
                process.stdout.write('✓\n');
                break;
            case 'failed':
                process.stdout.write('✗\n');
                if (result.error) {
                    console.error(`    Error: ${result.error}`);
                    if (result.actual !== undefined && result.expected !== undefined) {
                        console.error(`    Expected: ${JSON.stringify(result.expected)}`);
                        console.error(`    Actual: ${JSON.stringify(result.actual)}`);
                    }
                }
                break;
            case 'skipped':
                process.stdout.write('⏭\n');
                break;
        }
    }

    onSuiteStart(suite: string): void {
        console.log(`\n${suite}`);
    }

    onSuiteEnd(suite: string, results: TestResult[]): void {
        const passed = results.filter(r => r.status === 'passed').length;
        const failed = results.filter(r => r.status === 'failed').length;
        const skipped = results.filter(r => r.status === 'skipped').length;
        console.log(`  ${passed} passed, ${failed} failed, ${skipped} skipped`);
    }

    onRunStart(): void {
        console.log('🧪 Running tests...');
    }

    onRunEnd(summary: TestSummary): void {
        console.log('\n📊 Test Summary:');
        console.log(`  Total: ${summary.total}`);
        console.log(`  Passed: ${summary.passed}`);
        console.log(`  Failed: ${summary.failed}`);
        console.log(`  Skipped: ${summary.skipped}`);
        console.log(`  Duration: ${summary.duration}ms`);
        console.log('');

        if (summary.failed > 0) {
            console.log('❌ Some tests failed!');
            process.exit(1);
        } else {
            console.log('✅ All tests passed!');
        }
    }
}

class HTMLReporter implements Reporter {
    private results: TestResult[] = [];
    private startTime: number = 0;

    onTestStart(suite: string, test: string): void {
        // Do nothing
    }

    onTestResult(result: TestResult): void {
        this.results.push(result);
    }

    onSuiteStart(suite: string): void {
        // Do nothing
    }

    onSuiteEnd(suite: string, results: TestResult[]): void {
        // Do nothing
    }

    onRunStart(): void {
        this.startTime = Date.now();
    }

    onRunEnd(summary: TestSummary): void {
        const html = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Report</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1000px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
        }
        .summary {
            display: flex;
            justify-content: space-around;
            margin: 20px 0;
            padding: 10px;
            background-color: #f0f0f0;
            border-radius: 4px;
        }
        .summary-item {
            text-align: center;
        }
        .summary-item .value {
            font-size: 24px;
            font-weight: bold;
        }
        .summary-item.passed .value {
            color: #4CAF50;
        }
        .summary-item.failed .value {
            color: #f44336;
        }
        .summary-item.skipped .value {
            color: #ff9800;
        }
        .test-suite {
            margin: 20px 0;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
        }
        .suite-header {
            background-color: #f5f5f5;
            padding: 10px;
            font-weight: bold;
            border-bottom: 1px solid #e0e0e0;
        }
        .test-case {
            padding: 10px;
            border-bottom: 1px solid #f0f0f0;
        }
        .test-case:last-child {
            border-bottom: none;
        }
        .test-case.passed {
            color: #4CAF50;
        }
        .test-case.failed {
            color: #f44336;
            background-color: #ffebee;
        }
        .test-case.skipped {
            color: #ff9800;
            background-color: #fff3e0;
        }
        .error-message {
            margin-top: 5px;
            padding: 5px;
            background-color: #ffebee;
            border-left: 3px solid #f44336;
            font-size: 14px;
        }
        .duration {
            float: right;
            font-size: 14px;
            color: #666;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Test Report</h1>
        <div class="summary">
            <div class="summary-item">
                <div class="value">${summary.total}</div>
                <div>Total</div>
            </div>
            <div class="summary-item passed">
                <div class="value">${summary.passed}</div>
                <div>Passed</div>
            </div>
            <div class="summary-item failed">
                <div class="value">${summary.failed}</div>
                <div>Failed</div>
            </div>
            <div class="summary-item skipped">
                <div class="value">${summary.skipped}</div>
                <div>Skipped</div>
            </div>
            <div class="summary-item">
                <div class="value">${summary.duration}ms</div>
                <div>Duration</div>
            </div>
        </div>
        ${this.generateSuiteHTML()}
    </div>
</body>
</html>
        `;

        fs.writeFileSync('nargo-test-report.html', html);
        console.log('📊 Test report generated: nargo-test-report.html');
    }

    private generateSuiteHTML(): string {
        const suites = new Map<string, TestResult[]>();
        
        for (const result of this.results) {
            if (!suites.has(result.suite)) {
                suites.set(result.suite, []);
            }
            suites.get(result.suite)?.push(result);
        }

        let html = '';
        for (const [suite, results] of suites) {
            html += `
            <div class="test-suite">
                <div class="suite-header">${suite}</div>
                ${results.map(result => `
                <div class="test-case ${result.status}">
                    ${result.name}
                    <span class="duration">${result.duration}ms</span>
                    ${result.error ? `<div class="error-message">${result.error}</div>` : ''}
                </div>
                `).join('')}
            </div>
            `;
        }

        return html;
    }
}

export class TestRunner {
    private config: TestConfig;
    private reporter: Reporter;

    constructor(config: Partial<TestConfig> = {}) {
        this.config = {
            include: config.include || ['**/*.test.{ts,js,mjs}'],
            exclude: config.exclude || ['node_modules/**', 'dist/**', 'build/**'],
            parallel: config.parallel ?? true,
            timeout: config.timeout ?? 5000,
            coverage: config.coverage ?? false
        };
        this.reporter = new DefaultReporter();
    }

    setReporter(reporter: Reporter): void {
        this.reporter = reporter;
    }

    async run(): Promise<TestSummary> {
        this.reporter.onRunStart();
        const startTime = Date.now();

        try {
            // 查找测试文件
            const testFiles = this.findTestFiles();
            if (testFiles.length === 0) {
                console.log('No test files found!');
                return this.createEmptySummary();
            }

            // 导入测试文件
            await this.importTestFiles(testFiles);

            // 运行测试
            const results = await runCoreTests();

            // 生成摘要
            const summary = this.createSummary(results, Date.now() - startTime);

            // 报告结果
            this.reporter.onRunEnd(summary);

            // 生成HTML报告
            if (this.config.coverage) {
                const htmlReporter = new HTMLReporter();
                results.forEach(result => htmlReporter.onTestResult(result));
                htmlReporter.onRunEnd(summary);
            }

            return summary;
        } finally {
            // 重置状态
            resetState();
        }
    }

    private findTestFiles(): string[] {
        const testFiles: string[] = [];
        const cwd = process.cwd();
        console.log(`Current directory: ${cwd}`);
        console.log(`Include patterns: ${this.config.include}`);
        console.log(`Exclude patterns: ${this.config.exclude}`);

        // 简单实现：查找所有 .test.ts, .test.js, .test.mjs 文件
        const walk = (dir: string) => {
            console.log(`Walking directory: ${dir}`);
            const files = fs.readdirSync(dir);
            for (const file of files) {
                const fullPath = path.join(dir, file);
                console.log(`Checking file: ${fullPath}`);
                const stat = fs.statSync(fullPath);

                if (stat.isDirectory()) {
                    // 检查是否需要排除
                    const shouldExclude = this.config.exclude.some(pattern => this.matchesPattern(fullPath, pattern));
                    console.log(`Directory ${fullPath} should be excluded: ${shouldExclude}`);
                    if (!shouldExclude) {
                        walk(fullPath);
                    }
                } else {
                    const isTest = this.isTestFile(fullPath);
                    console.log(`File ${fullPath} is test file: ${isTest}`);
                    if (isTest) {
                        testFiles.push(fullPath);
                    }
                }
            }
        };

        walk(cwd);
        console.log(`Found test files: ${testFiles}`);
        return testFiles;
    }

    private isTestFile(filePath: string): boolean {
        return this.config.include.some(pattern => this.matchesPattern(filePath, pattern));
    }

    private matchesPattern(filePath: string, pattern: string): boolean {
        // 简单的模式匹配实现，支持 **/*.test.{ts,js,mjs} 这种模式
        let regexPattern = pattern
            .replace(/\./g, '\\.')
            .replace(/\*/g, '.*')
            .replace(/\?/g, '.');
        
        // 处理 {ts,js,mjs} 这种模式
        regexPattern = regexPattern.replace(/\{([^}]+)\}/g, (match, group) => {
            const options = group.split(',').map((opt: string) => opt.trim());
            return `(${options.join('|')})`;
        });
        
        const regex = new RegExp(regexPattern);
        return regex.test(filePath.replace(/\\/g, '/'));
    }

    private async importTestFiles(files: string[]): Promise<void> {
        for (const file of files) {
            try {
                await import(`file://${file}`);
            } catch (error) {
                console.error(`Error importing test file ${file}:`, error);
            }
        }
    }

    private createSummary(results: TestResult[], duration: number): TestSummary {
        const total = results.length;
        const passed = results.filter(r => r.status === 'passed').length;
        const failed = results.filter(r => r.status === 'failed').length;
        const skipped = results.filter(r => r.status === 'skipped').length;

        return {
            total,
            passed,
            failed,
            skipped,
            duration,
            results
        };
    }

    private createEmptySummary(): TestSummary {
        return {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration: 0,
            results: []
        };
    }
}

export async function runTests(config: Partial<TestConfig> = {}): Promise<TestSummary> {
    const runner = new TestRunner(config);
    return runner.run();
}
