告诉你一个生活开心的秘诀：少责备自己，多辱骂别人。被你骂的人都罪有应得，但别人骂你肯定是没素质！

----

RSDL 基于 Zephyr ASDL (https://www.cs.princeton.edu/~appel/papers/asdl97.pdf )，
用于从同一套类型定义生成不同语言的数据结构定义、序列化以及反序列化。RSDL 主要引入了预处理指令
#include，以及注解语法

目前支持的后端有:
  - Rust (by @NEETovo)

计划支持的后端有:
  - TypeScript (interface-based)
  - typeAssert
  - C++ + Qt + QtJSON

----

注解文档
  - 总则: 无论何时，注解的行为总是由生成器决定的，注解本身只是起到一种“建议”作用

通用注解
  - [boxed]
    在一些像 C++ 和 Rust 这样的值类型语言中，创建互递归的数据结构需要使用指针来
    引入间接性。boxed 用来建议生成器为指定的字段生成一个指针
  - [doc("docstring")] 或者 [doc = "docstring"]
    建议生成器为指定的实体生成文档注释
  - [doc_ctor("docstring")] 或者 [doc_ctor = "docstring"]
    在 SUM 类型中，建议生成器为指定的构造器对应生成的那个数据类型生成文档注释
    例如对于 Rust 目标，构造器上用 doc 指定的注释会被放置在对应的 enum variant 上
    而用 doc_ctor 指定的注释会被放置在为这个 enum variant 生成的 struct 类型上
  - [private]
    建议生成器将指定的实体设为私有
  - [inline]
    对别名类型生效，建议生成器不生成别名类型，而是将被别名的类型放置在使用到别名的地方

Rust 后端支持的注解
  - [rust_derive(traits)]
    为类型派生指定的 traits，traits 之间用逗号分隔
    如果对 SUM 类型使用，每个 enum variant 对应的 struct 类型都会被派生指定的 traits
  - [rust_attr(attributes)]
    为类型或者字段添加指定的 Rust attribute，attribute 语法和 Rust 总体上一致
