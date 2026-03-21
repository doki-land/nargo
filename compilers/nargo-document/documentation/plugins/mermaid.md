# Mermaid 图表插件

Mermaid 插件用于在文档中渲染各种类型的图表。

## 使用方法

使用 ```mermaid 代码块来定义图表：

```markdown
```mermaid
graph TD
    A[开始] --> B{判断}
    B -->|是| C[处理]
    B -->|否| D[结束]
    C --> D
```
```

## 配置

在配置文件中启用 Mermaid 插件：

```toml
[plugins]
mermaid = true
```

## 图表类型

### 流程图

```mermaid
graph LR
    A[用户] --> B(前端)
    B --> C{API网关}
    C -->|认证| D[认证服务]
    C -->|数据| E[数据服务]
    D --> F[(数据库)]
    E --> F
```

### 时序图

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant Database
    
    Client->>Server: HTTP 请求
    Server->>Database: 查询数据
    Database-->>Server: 返回结果
    Server-->>Client: 响应数据
```

### 类图

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +String breed
        +bark()
    }
    class Cat {
        +String color
        +meow()
    }
    Animal <|-- Dog
    Animal <|-- Cat
```

### 甘特图

```mermaid
gantt
    title 项目进度
    dateFormat  YYYY-MM-DD
    section 设计
    需求分析     :a1, 2024-01-01, 10d
    系统设计     :a2, after a1, 15d
    section 开发
    前端开发     :b1, after a2, 20d
    后端开发     :b2, after a2, 25d
    section 测试
    单元测试     :c1, after b1, 10d
    集成测试     :c2, after b2, 10d
```
