export interface TestSuite {
    name: string;
    tests: TestCase[];
    suites: TestSuite[];
    beforeAll: HookFn[];
    afterAll: HookFn[];
    beforeEach: HookFn[];
    afterEach: HookFn[];
    parent: TestSuite | null;
    skip: boolean;
    only: boolean;
}

export interface TestCase {
    name: string;
    fn: TestFn;
    skip: boolean;
    only: boolean;
    timeout: number;
}

export type TestFn = () => void | Promise<void>;
export type HookFn = () => void | Promise<void>;

export interface TestResult {
    suite: string;
    name: string;
    status: "passed" | "failed" | "skipped";
    duration: number;
    error?: string;
    actual?: unknown;
    expected?: unknown;
}

export interface TestSummary {
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    duration: number;
    results: TestResult[];
}

export interface Matchers<T = unknown> {
    toBe(expected: T): void;
    toEqual(expected: T): void;
    toBeTruthy(): void;
    toBeFalsy(): void;
    toBeNull(): void;
    toBeUndefined(): void;
    toBeDefined(): void;
    toBeNaN(): void;
    toBeGreaterThan(expected: number): void;
    toBeGreaterThanOrEqual(expected: number): void;
    toBeLessThan(expected: number): void;
    toBeLessThanOrEqual(expected: number): void;
    toBeCloseTo(expected: number, precision?: number): void;
    toBeInstanceOf(expected: Function): void;
    toContain(expected: unknown): void;
    toContainEqual(expected: unknown): void;
    toHaveLength(expected: number): void;
    toMatch(expected: string | RegExp): void;
    toHaveProperty(path: string | string[], expected?: unknown): void;
    toMatchObject(expected: Record<string, unknown>): void;
    toThrow(expected?: string | RegExp | Error): void;
    toThrowError(expected?: string | RegExp | Error): void;
    readonly not: Matchers<T>;
    readonly resolves: PromiseMatchers<T>;
    readonly rejects: PromiseMatchers<T>;
}

export interface PromiseMatchers<T = unknown> {
    toBe(expected: T): Promise<void>;
    toEqual(expected: T): Promise<void>;
    toThrow(expected?: string | RegExp | Error): Promise<void>;
}

export interface TestConfig {
    include: string[];
    exclude: string[];
    parallel: boolean;
    timeout: number;
    coverage: boolean;
}

export interface Reporter {
    onTestStart(suite: string, test: string): void;
    onTestResult(result: TestResult): void;
    onSuiteStart(suite: string): void;
    onSuiteEnd(suite: string, results: TestResult[]): void;
    onRunStart(): void;
    onRunEnd(summary: TestSummary): void;
}
