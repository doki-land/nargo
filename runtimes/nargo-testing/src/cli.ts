#!/usr/bin/env node

import { runTestSuite } from './index.js';
import { program } from 'commander';

program
    .name('nargo-testing')
    .description('Nargo testing utility')
    .version('1.0.0');

program
    .command('run')
    .description('Run tests')
    .option('--include <patterns...>', 'Include test files matching patterns')
    .option('--exclude <patterns...>', 'Exclude test files matching patterns')
    .option('--no-parallel', 'Run tests sequentially')
    .option('--timeout <ms>', 'Test timeout in milliseconds', '5000')
    .option('--coverage', 'Generate coverage report')
    .option('--filter <pattern>', 'Run tests matching pattern')
    .action(async (options: any) => {
        try {
            await runTestSuite({
                include: options.include || ['**/*.test.{ts,js,mjs}'],
                exclude: options.exclude || ['node_modules/**', 'dist/**', 'build/**'],
                parallel: options.parallel !== false,
                timeout: parseInt(options.timeout),
                coverage: options.coverage
            });
        } catch (error) {
            console.error('Error running tests:', error);
            process.exit(1);
        }
    });

program.parse();
