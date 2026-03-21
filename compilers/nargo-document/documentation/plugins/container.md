# Container 自定义容器插件

Container 插件用于创建自定义的提示框、警告框等容器。

## 使用方法

使用 `:::` 语法来创建容器：

```markdown
:::tip
这是一个提示信息
:::
```

## 容器类型

### Tip（提示）

```markdown
:::tip
这是一个有用的提示信息
:::
```

### Warning（警告）

```markdown
:::warning
请注意这个警告信息
:::
```

### Danger（危险）

```markdown
:::danger
这是一个危险警告
:::
```

### Info（信息）

```markdown
:::info
这是一条信息
:::
```

### 自定义标题

可以为容器添加自定义标题：

```markdown
:::tip 自定义标题
这是一个带有自定义标题的提示
:::
```

## 配置

在配置文件中启用 Container 插件：

```toml
[plugins]
container = true

[plugins.container]
types = ["tip", "warning", "danger", "info"]
```

## 示例

### 提示框示例

:::tip
使用 nargo-document 可以快速生成美观的文档！
:::

### 警告框示例

:::warning
请注意，此功能仍在开发中，API 可能会发生变化。
:::

### 危险框示例

:::danger
此操作不可逆，请谨慎执行！
:::

### 信息框示例

:::info
更多信息请参考官方文档。
:::
