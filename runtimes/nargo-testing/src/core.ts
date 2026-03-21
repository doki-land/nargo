import type { TestSuite, TestCase, TestFn, TestResult, Matchers, PromiseMatchers } from "./types.js";

let currentSuite: TestSuite | null = null;
let rootSuite: TestSuite | null = null;
let hasOnly = false;

function createSuite(name: string, parent: TestSuite | null = null): TestSuite {
    return {
        name,
        tests: [],
        suites: [],
        beforeAll: [],
        afterAll: [],
        beforeEach: [],
        afterEach: [],
        parent,
        skip: false,
        only: false,
    };
}

function getFullSuiteName(suite: TestSuite | null): string {
    if (!suite || !suite.parent) return suite?.name ?? "";
    const parentName = getFullSuiteName(suite.parent);
    return parentName ? `${parentName} > ${suite.name}` : suite.name;
}

export function describe(name: string, fn: () => void): void {
    const suite = createSuite(name, currentSuite);
    if (currentSuite) {
        currentSuite.suites.push(suite);
    } else {
        if (!rootSuite) {
            // 创建一个默认的根套件
            rootSuite = createSuite('Root Suite', null);
        }
        rootSuite.suites.push(suite);
    }
    const previousSuite = currentSuite;
    currentSuite = suite;
    try {
        fn();
    } finally {
        currentSuite = previousSuite;
    }
}

describe.skip = function (name: string, fn: () => void): void {
    const suite = createSuite(name, currentSuite);
    suite.skip = true;
    if (currentSuite) {
        currentSuite.suites.push(suite);
    } else {
        if (!rootSuite) {
            rootSuite = createSuite('Root Suite', null);
        }
        rootSuite.suites.push(suite);
    }
    const previousSuite = currentSuite;
    currentSuite = suite;
    try {
        fn();
    } finally {
        currentSuite = previousSuite;
    }
};

describe.only = function (name: string, fn: () => void): void {
    hasOnly = true;
    const suite = createSuite(name, currentSuite);
    suite.only = true;
    if (currentSuite) {
        currentSuite.suites.push(suite);
    } else {
        if (!rootSuite) {
            rootSuite = createSuite('Root Suite', null);
        }
        rootSuite.suites.push(suite);
    }
    const previousSuite = currentSuite;
    currentSuite = suite;
    try {
        fn();
    } finally {
        currentSuite = previousSuite;
    }
};

export function it(name: string, fn: TestFn, timeout = 5000): void {
    if (!currentSuite) throw new Error("it() must be called inside describe()");
    currentSuite.tests.push({ name, fn, skip: false, only: false, timeout });
}

it.skip = function (name: string, _fn: TestFn): void {
    if (!currentSuite) throw new Error("it.skip() must be called inside describe()");
    currentSuite.tests.push({ name, fn: async () => {}, skip: true, only: false, timeout: 0 });
};

it.only = function (name: string, fn: TestFn, timeout = 5000): void {
    hasOnly = true;
    if (!currentSuite) throw new Error("it.only() must be called inside describe()");
    currentSuite.tests.push({ name, fn, skip: false, only: true, timeout });
};

export const test = it;
test.skip = it.skip;
test.only = it.only;

export function beforeEach(fn: () => void | Promise<void>): void {
    if (!currentSuite) throw new Error("beforeEach() must be called inside describe()");
    currentSuite.beforeEach.push(fn);
}

export function afterEach(fn: () => void | Promise<void>): void {
    if (!currentSuite) throw new Error("afterEach() must be called inside describe()");
    currentSuite.afterEach.push(fn);
}

export function beforeAll(fn: () => void | Promise<void>): void {
    if (!currentSuite) throw new Error("beforeAll() must be called inside describe()");
    currentSuite.beforeAll.push(fn);
}

export function afterAll(fn: () => void | Promise<void>): void {
    if (!currentSuite) throw new Error("afterAll() must be called inside describe()");
    currentSuite.afterAll.push(fn);
}

function deepEqual(a: unknown, b: unknown): boolean {
    if (a === b) return true;
    if (a === null || b === null) return a === b;
    if (typeof a !== typeof b) return false;
    if (typeof a !== "object") return a === b;
    if (Array.isArray(a) && Array.isArray(b)) {
        return a.length === b.length && a.every((item, index) => deepEqual(item, b[index]));
    }
    if (Array.isArray(a) || Array.isArray(b)) return false;
    const aKeys = Object.keys(a as object);
    const bKeys = Object.keys(b as object);
    return aKeys.length === bKeys.length && aKeys.every((key) => deepEqual((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key]));
}

function formatValue(value: unknown): string {
    if (value === null) return "null";
    if (value === undefined) return "undefined";
    if (typeof value === "string") return `"${value}"`;
    if (typeof value === "symbol") return value.toString();
    if (typeof value === "function") return `[Function: ${value.name || "anonymous"}]`;
    if (Array.isArray(value)) return `[${value.map(formatValue).join(", ")}]`;
    if (typeof value === "object") {
        const entries = Object.entries(value as Record<string, unknown>).map(([k, v]) => `${k}: ${formatValue(v)}`);
        return `{ ${entries.join(", ")} }`;
    }
    return String(value);
}

class AssertionError extends Error {
    actual: unknown;
    expected: unknown;
    constructor(message: string, actual: unknown, expected: unknown) {
        super(message);
        this.name = "AssertionError";
        this.actual = actual;
        this.expected = expected;
    }
}

class MatchersImpl<T = unknown> {
    private _actual: T;
    private _isNot: boolean;

    constructor(actual: T, isNot = false) {
        this._actual = actual;
        this._isNot = isNot;
    }

    private _assert(pass: boolean, message: string, expected?: unknown): void {
        const result = this._isNot ? !pass : pass;
        if (!result) {
            throw new AssertionError(this._isNot ? `not ${message}` : message, this._actual, expected);
        }
    }

    toBe(expected: T): void {
        this._assert(Object.is(this._actual, expected), `expected ${formatValue(this._actual)} to be ${formatValue(expected)}`, expected);
    }

    toEqual(expected: T): void {
        this._assert(deepEqual(this._actual, expected), `expected ${formatValue(this._actual)} to equal ${formatValue(expected)}`, expected);
    }

    toBeTruthy(): void {
        this._assert(Boolean(this._actual), `expected ${formatValue(this._actual)} to be truthy`, true);
    }

    toBeFalsy(): void {
        this._assert(!Boolean(this._actual), `expected ${formatValue(this._actual)} to be falsy`, false);
    }

    toBeNull(): void {
        this._assert(this._actual === null, `expected ${formatValue(this._actual)} to be null`, null);
    }

    toBeUndefined(): void {
        this._assert(this._actual === undefined, `expected ${formatValue(this._actual)} to be undefined`, undefined);
    }

    toBeDefined(): void {
        this._assert(this._actual !== undefined, `expected ${formatValue(this._actual)} to be defined`, "defined");
    }

    toBeNaN(): void {
        this._assert(Number.isNaN(this._actual), `expected ${formatValue(this._actual)} to be NaN`, NaN);
    }

    toBeGreaterThan(expected: number): void {
        this._assert((this._actual as number) > expected, `expected ${this._actual} to be greater than ${expected}`, expected);
    }

    toBeGreaterThanOrEqual(expected: number): void {
        this._assert((this._actual as number) >= expected, `expected ${this._actual} to be greater than or equal to ${expected}`, expected);
    }

    toBeLessThan(expected: number): void {
        this._assert((this._actual as number) < expected, `expected ${this._actual} to be less than ${expected}`, expected);
    }

    toBeLessThanOrEqual(expected: number): void {
        this._assert((this._actual as number) <= expected, `expected ${this._actual} to be less than or equal to ${expected}`, expected);
    }

    toBeCloseTo(expected: number, precision = 2): void {
        const diff = Math.abs((this._actual as number) - expected);
        const threshold = Math.pow(10, -precision) / 2;
        this._assert(diff < threshold, `expected ${this._actual} to be close to ${expected} (precision: ${precision})`, expected);
    }

    toBeInstanceOf(expected: Function): void {
        this._assert(this._actual instanceof expected, `expected ${formatValue(this._actual)} to be instance of ${expected.name}`, expected);
    }

    toContain(expected: unknown): void {
        let pass = false;
        if (typeof this._actual === "string") {
            pass = this._actual.includes(String(expected));
        } else if (Array.isArray(this._actual)) {
            pass = this._actual.some((item) => deepEqual(item, expected));
        }
        this._assert(pass, `expected ${formatValue(this._actual)} to contain ${formatValue(expected)}`, expected);
    }

    toContainEqual(expected: unknown): void {
        const pass = Array.isArray(this._actual) && this._actual.some((item) => deepEqual(item, expected));
        this._assert(pass, `expected ${formatValue(this._actual)} to contain equal ${formatValue(expected)}`, expected);
    }

    toHaveLength(expected: number): void {
        const actual = (this._actual as { length?: number }).length;
        this._assert(actual === expected, `expected to have length ${expected}, but got ${actual}`, expected);
    }

    toMatch(expected: string | RegExp): void {
        const actual = String(this._actual);
        const pass = typeof expected === "string" ? actual.includes(expected) : expected.test(actual);
        this._assert(pass, `expected ${formatValue(actual)} to match ${formatValue(expected)}`, expected);
    }

    toHaveProperty(path: string | string[], expected?: unknown): void {
        const pathArray = Array.isArray(path) ? path : path.split(".");
        let current: unknown = this._actual;
        for (const key of pathArray) {
            if (current && typeof current === "object" && key in current) {
                current = (current as Record<string, unknown>)[key];
            } else {
                this._assert(false, `expected ${formatValue(this._actual)} to have property "${pathArray.join(".")}"`, path);
                return;
            }
        }
        if (expected !== undefined) {
            this._assert(deepEqual(current, expected), `expected property "${pathArray.join(".")}" to be ${formatValue(expected)}, but got ${formatValue(current)}`, expected);
        }
    }

    toMatchObject(expected: Record<string, unknown>): void {
        const pass = Object.keys(expected).every((key) => deepEqual((this._actual as Record<string, unknown>)[key], expected[key]));
        this._assert(pass, `expected ${formatValue(this._actual)} to match object ${formatValue(expected)}`, expected);
    }

    toThrow(expected?: string | RegExp | Error): void {
        let pass = false;
        let error: Error | null = null;
        try {
            (this._actual as () => void)();
        } catch (e) {
            error = e as Error;
            if (expected === undefined) {
                pass = true;
            } else if (typeof expected === "string") {
                pass = error.message.includes(expected);
            } else if (expected instanceof RegExp) {
                pass = expected.test(error.message);
            } else if (expected instanceof Error) {
                pass = error.constructor === expected.constructor;
            }
        }
        this._assert(pass, expected === undefined ? "expected function to throw" : `expected function to throw ${formatValue(expected)}`, expected);
    }

    toThrowError(expected?: string | RegExp | Error): void {
        this.toThrow(expected);
    }

    get not(): Matchers<T> {
        return new MatchersImpl(this._actual, !this._isNot) as unknown as Matchers<T>;
    }

    get resolves(): PromiseMatchers<T> {
        return {
            toBe: async (expected: T) => {
                const value = await (this._actual as Promise<T>);
                new MatchersImpl(value, this._isNot).toBe(expected as unknown as Awaited<T>);
            },
            toEqual: async (expected: T) => {
                const value = await (this._actual as Promise<T>);
                new MatchersImpl(value, this._isNot).toEqual(expected as unknown as Awaited<T>);
            },
            toThrow: async (expected?: string | RegExp | Error) => {
                try {
                    await (this._actual as Promise<T>);
                    throw new AssertionError("expected promise to reject", this._actual, expected);
                } catch (e) {
                    if ((e as Error).name === "AssertionError") throw e;
                    new MatchersImpl(() => { throw e; }, this._isNot).toThrow(expected);
                }
            },
        };
    }

    get rejects(): PromiseMatchers<T> {
        return {
            toBe: async (expected: T) => {
                try {
                    await (this._actual as Promise<T>);
                    throw new AssertionError("expected promise to reject", this._actual, expected);
                } catch (e) {
                    if ((e as Error).name === "AssertionError") throw e;
                    new MatchersImpl(e, this._isNot).toBe(expected);
                }
            },
            toEqual: async (expected: T) => {
                try {
                    await (this._actual as Promise<T>);
                    throw new AssertionError("expected promise to reject", this._actual, expected);
                } catch (e) {
                    if ((e as Error).name === "AssertionError") throw e;
                    new MatchersImpl(e, this._isNot).toEqual(expected);
                }
            },
            toThrow: async (expected?: string | RegExp | Error) => {
                try {
                    await (this._actual as Promise<T>);
                    throw new AssertionError("expected promise to reject", this._actual, expected);
                } catch (e) {
                    if ((e as Error).name === "AssertionError") throw e;
                    new MatchersImpl(() => { throw e; }, this._isNot).toThrow(expected);
                }
            },
        };
    }
}

export function expect<T>(actual: T): Matchers<T> {
    return new MatchersImpl(actual) as unknown as Matchers<T>;
}

export async function runTests(suite: TestSuite | null = rootSuite): Promise<TestResult[]> {
    const results: TestResult[] = [];
    if (!suite) return results;
    if (suite.skip) return results;

    if (hasOnly && !suite.only && !suite.tests.some((t) => t.only)) {
        for (const childSuite of suite.suites) {
            results.push(...(await runTests(childSuite)));
        }
        return results;
    }

    const suiteName = getFullSuiteName(suite);

    for (const hook of suite.beforeAll) {
        await hook();
    }

    for (const test of suite.tests) {
        if (test.skip || (hasOnly && !test.only && !suite.only)) {
            results.push({ suite: suiteName, name: test.name, status: "skipped", duration: 0 });
            continue;
        }

        const startTime = Date.now();
        try {
            for (const hook of suite.beforeEach) {
                await hook();
            }
            await test.fn();
            for (const hook of suite.afterEach) {
                await hook();
            }
            results.push({
                suite: suiteName,
                name: test.name,
                status: "passed",
                duration: Date.now() - startTime,
            });
        } catch (e) {
            const error = e as Error;
            results.push({
                suite: suiteName,
                name: test.name,
                status: "failed",
                duration: Date.now() - startTime,
                error: error.message || String(error),
                actual: (error as AssertionError).actual,
                expected: (error as AssertionError).expected,
            });
        }
    }

    for (const childSuite of suite.suites) {
        results.push(...(await runTests(childSuite)));
    }

    for (const hook of suite.afterAll) {
        await hook();
    }

    return results;
}

export function getRootSuite(): TestSuite | null {
    return rootSuite;
}

export function resetState(): void {
    currentSuite = null;
    rootSuite = null;
    hasOnly = false;
}
