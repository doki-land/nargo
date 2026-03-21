/**
 * Nargo benchmarking module.
 */

/**
 * Benchmark runner.
 */
export class NargoBench {
  private root: string;

  /**
   * Create a new benchmark runner.
   * @param root The root directory for benchmarks
   */
  constructor(root: string) {
    this.root = root;
  }

  /**
   * Run benchmarks.
   */
  run(): void {
    console.log(`Running benchmarks in ${this.root}...`);
  }
}

/**
 * Legacy type alias for compatibility.
 */
export type BenchmarkRunner = NargoBench;

/**
 * Create a new benchmark runner.
 * @param root The root directory for benchmarks
 */
export function createBenchmarkRunner(root: string): NargoBench {
  return new NargoBench(root);
}