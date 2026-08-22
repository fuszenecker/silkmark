# Math III

## Cases

```math
f(x)=\begin{cases}
x^2 & x \ge 0 \\
-x & x < 0
\end{cases}
```

## Limits and named operators

$$
\lim_{x\to0}\frac{\sin x}{x}=1
$$

$$
y = \log x + \ln x + \exp x + \cos x + \tan x
$$

## Scalable-style delimiter syntax

The renderer accepts the common LaTeX delimiter syntax:

$$
\left( \frac{a+b}{c} \right)
$$

$$
\left\langle \vec{x}, \vec{y} \right\rangle
$$

Invisible delimiters are also accepted:

$$
\left. x^2 \right|_0^1
$$

SilkMark intentionally remains a compact LaTeX-like subset rather than a full TeX engine.
