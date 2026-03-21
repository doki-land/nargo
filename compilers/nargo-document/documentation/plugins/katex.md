# KaTeX 数学公式插件

KaTeX 插件用于在文档中渲染数学公式。

## 使用方法

### 行内公式

使用单个美元符号 `$` 包裹行内公式：

```markdown
这是一个行内公式：$E = mc^2$
```

### 块级公式

使用双美元符号 `$$` 包裹块级公式：

```markdown
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$
```

## 配置

在配置文件中启用 KaTeX 插件：

```toml
[plugins]
katex = true
```

## 示例

### 二次方程求根公式

```markdown
$$
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
$$
```

### 矩阵

```markdown
$$
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
$$
```

### 求和与积分

```markdown
$$
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
$$
```
