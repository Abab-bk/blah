包管理功能：

```brak.toml
[package]
name = "left-pad"
version = "0.1.0"

[dependencies]
xxx = "1.0.0"

[features]
space-5 = "space-5.bf"  # 使用空格覆盖接下来 5 个 data 的位置
```

基本上，这个 features 其实是一堆 snippet。
在 brainfuck 里这样用：

```brainfuck
>+++++
@left-pad/space-5 基本上就是直接粘贴 space-5.bf 的内容到这里，不做任何特殊处理
```
