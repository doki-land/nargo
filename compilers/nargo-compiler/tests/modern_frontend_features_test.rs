use nargo_compiler::{CompileMode, CompileOptions, compile};
use nargo_types::Result;

#[test]
fn test_es6_plus_features() -> Result<()> {
    let source = r#"
<script>
    // ES6+ features test
    const message = 'Hello, ES6+!';
    let count = 0;
    
    // Arrow function
    const add = (a, b) => a + b;
    
    // Template literal
    const greeting = `Hello, ${message}!`;
    
    // Destructuring
    const { name, age } = { name: 'John', age: 30 };
    
    // Spread operator
    const arr1 = [1, 2, 3];
    const arr2 = [...arr1, 4, 5, 6];
    
    // Optional chaining
    const user = { address: { street: 'Main St' } };
    const street = user?.address?.street;
    
    // Nullish coalescing
    const value = null ?? 'default';
    
    // Async/await
    async function fetchData() {
        return 'Data';
    }
    
    // For-of loop
    for (const item of arr2) {
        console.log(item);
    }
    
    // Try/catch
    try {
        // Some code
    } catch (error) {
        console.error(error);
    }
    
    export default {
        name: 'TestComponent',
        setup() {
            return {
                message,
                count,
                add,
                greeting,
                name,
                age,
                arr2,
                street,
                value
            };
        }
    };
</script>

<template>
    <div>
        <h1>{{ message }}</h1>
        <p>{{ greeting }}</p>
        <p>Name: {{ name }}, Age: {{ age }}</p>
        <p>Street: {{ street }}</p>
        <p>Value: {{ value }}</p>
    </div>
</template>

<style>
    .container {
        display: flex;
        justify-content: center;
        align-items: center;
    }
</style>
"#;

    let result = compile("TestComponent", source)?;
    assert!(!result.code.is_empty());
    assert!(!result.css.is_empty());
    assert!(!result.html.is_empty());

    Ok(())
}

#[test]
fn test_typescript_features() -> Result<()> {
    let source = r#"
<script lang="ts">
    // TypeScript features test
    interface User {
        name: string;
        age: number;
        address?: string;
    }
    
    type UserId = string | number;
    
    enum Direction {
        Up,
        Down,
        Left,
        Right
    }
    
    const user: User = { name: 'John', age: 30 };
    const userId: UserId = 123;
    const direction: Direction = Direction.Up;
    
    function greet(name: string): string {
        return `Hello, ${name}!`;
    }
    
    async function fetchData<T>(url: string): Promise<T> {
        return {} as T;
    }
    
    export default {
        name: 'TypeScriptComponent',
        setup() {
            return {
                user,
                userId,
                direction,
                greet
            };
        }
    };
</script>

<template>
    <div>
        <h1>TypeScript Test</h1>
        <p>Name: {{ user.name }}, Age: {{ user.age }}</p>
        <p>User ID: {{ userId }}</p>
        <p>Direction: {{ direction }}</p>
    </div>
</template>
"#;

    let result = compile("TypeScriptComponent", source)?;
    assert!(!result.code.is_empty());
    assert!(!result.html.is_empty());

    Ok(())
}

#[test]
fn test_css_new_features() -> Result<()> {
    let source = r#"
<template>
    <div class="container">
        <div class="grid-item">1</div>
        <div class="grid-item">2</div>
        <div class="grid-item">3</div>
        <div class="grid-item">4</div>
    </div>
</template>

<style>
    :root {
        --primary-color: #3498db;
        --secondary-color: #2ecc71;
        --font-size: 16px;
    }
    
    .container {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        grid-gap: 20px;
        padding: 20px;
        background-color: var(--secondary-color);
    }
    
    .grid-item {
        background-color: var(--primary-color);
        color: white;
        padding: 20px;
        text-align: center;
        font-size: var(--font-size);
        border-radius: 8px;
        
        &:hover {
            transform: scale(1.05);
            transition: transform 0.3s ease;
        }
    }
    
    @container (max-width: 600px) {
        .grid-item {
            font-size: calc(var(--font-size) * 0.8);
        }
    }
</style>
"#;

    let result = compile("CssFeaturesComponent", source)?;
    assert!(!result.css.is_empty());
    assert!(!result.html.is_empty());

    Ok(())
}
