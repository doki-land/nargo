import { NargoBench, createBenchmarkRunner } from './dist/index.js';

// Test 1: Using constructor
const bench1 = new NargoBench('./benchmarks');
console.log('Test 1: Using constructor');
bench1.run();

// Test 2: Using factory function
const bench2 = createBenchmarkRunner('./benchmarks');
console.log('\nTest 2: Using factory function');
bench2.run();

console.log('\nAll tests passed!');
