# 代码高亮插件

nargo-document 提供两种代码高亮方案：Shiki 和 Prism。

## Shiki 插件

Shiki 是一个基于 TextMate 语法的代码高亮器。

### 配置

```toml
[plugins]
shiki = true
```

### 使用

```markdown
```rust
fn main() {
    println!("Hello, World!");
}
```
```

## Prism 插件

Prism 是一个轻量级的代码高亮库。

### 配置

```toml
[plugins]
prism = true
```

### 使用

```markdown
```javascript
function hello() {
    console.log('Hello, World!');
}
```
```

## 支持的语言

两种插件都支持多种编程语言，包括但不限于：
- Rust
- JavaScript / TypeScript
- Python
- Go
- Java
- C / C++
- HTML / CSS
- SQL
- Bash

## 示例

### Rust 示例

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    if let Some(value) = map.get("key") {
        println!("Value: {}", value);
    }
}
```

### TypeScript 示例

```typescript
interface User {
    id: number;
    name: string;
    email: string;
}

function getUser(id: number): Promise<User> {
    return fetch(`/api/users/${id}`)
        .then(res => res.json());
}
```

### Python 示例

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

for i in range(10):
    print(fibonacci(i))
```
