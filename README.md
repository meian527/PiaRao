# PiaRao

非传统函数式 + 不完全面向对象 + 强表达力 玩具解释器

## 特性

- **多范式**: 融合函数式与面向对象特性
- **任意精度算术**: 基于 `rug` 的有理数运算
- **灵活语法**: 函数调用可选括号 `f(x)` 或 `f x`
- **表达式优先**: `if` 为表达式，块返回最后值

## 语法示例

### 变量声明
```piarao
let x = 10;
let name = "PiaRao";
```

### 可变性
PiaRao没有可变性, 不能够修改绑定, 需要修改可以使用覆盖或者遮蔽
```piarao
let x = 3;
let x = "str";

{
    let x = 6;
    println x;
}
println x;
```

### 函数定义（两种形式）
```piarao
let add = fn a b -> a + b;
let add a b = a + b;
```

### Lambda 表达式
```piarao
let square = fn x -> x * x;
let square_result = square 5;
let sub_result = (fn a b -> a - b)(3, 1);
```

### If 表达式
```piarao
let max a b = if a > b then a else b;

let result = max 4 6; 
```

### 块表达式
```piarao
let result = {
    let x = 1;
    let y = 2;
    x + y
};
```

### 递归（斐波那契）
```piarao
let fib n = if n <= 1 then n else fib(n - 1) + fib(n - 2);
let result = fib 30;
```

### Record
```piarao
type Point = record x y;

let point = Point 4 5;

println point.x point.y;
```
### 成员函数
```piarao
type Point = record x y {
    area self = self.x + self.y;
};

let point = Point 4 5;

println point.x " " point.y;
println(point.area());
```
### 运算符
| 类型 | 运算符                                |
|------|------------------------------------|
| 算术 | `+`, `-`, `*`, `/`, `%`, `^`(幂)    |
| 比较 | `>`, `<`, `>=`, `<=`, `==`, `!=`   |
| 逻辑 | `and`, `or`                        |
| 一元 | `-(expr)`(取负), `!`(非), `-num`负数字面量 |

### 内置函数
| 函数 | 说明 |
|------|------|
| `print(...)` | 打印不换行 |
| `println(...)` | 打印并换行 |
| `input()` | 读取用户输入 |
| `type_info(v)` | 获取类型信息 |
| `to_string(v)` | 转换为字符串 |

## 构建与运行

### 要求
- Rust 1.88+

### 构建
```bash
cargo build --release  # 优化构建
```

### 运行
```bash
cargo run
```

默认运行斐波那契数列计算示例。修改 `src/main.rs` 中的 `src` 变量以执行自定义代码。

## 项目结构
```
piarao/
├── src/
│   ├── main.rs         # 入口
│   ├── lexer.rs        # 词法分析器
│   ├── parser.rs       # 递归下降解析器
│   ├── ast.rs          # 抽象语法树
│   ├── interpreter.rs  # 树遍历解释器
│   ├── objects.rs      # 对象系统
│   ├── lang.rs         # 语言状态封装
│   └── builtins/
│       └── mod.rs      # 内置函数
├── Cargo.toml
└── README.md
```

## 架构

```
源码 → Lexer(词法分析) → Parser(语法分析) → AST → Interpreter(解释执行)
```

解释器采用树遍历（tree-walking）方式，使用栈式作用域管理变量与函数帧。
