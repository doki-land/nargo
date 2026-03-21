import { describe, it, expect, runTests } from './dist/index.js';

describe('Test Suite 1', () => {
    it('should pass a simple test', () => {
        console.log('Running test: should pass a simple test');
        expect(1 + 1).toBe(2);
    });

    it('should fail a test', () => {
        console.log('Running test: should fail a test');
        expect(1 + 1).toBe(3);
    });

    it.skip('should skip this test', () => {
        console.log('Running test: should skip this test');
        expect(true).toBe(false);
    });
});

describe('Test Suite 2', () => {
    it('should test objects', () => {
        console.log('Running test: should test objects');
        expect({ a: 1, b: 2 }).toEqual({ a: 1, b: 2 });
    });

    it('should test arrays', () => {
        console.log('Running test: should test arrays');
        expect([1, 2, 3]).toContain(2);
    });
});

// 手动运行测试，看看是否能正确执行
runTests().then(results => {
    console.log('Manual test run results:', results);
});
