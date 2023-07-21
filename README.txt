告诉你一个生活开心的秘诀：少责备自己，多辱骂别人。被你骂的人都罪有应得，但别人骂你肯定是没素质！

----

RSDL 基于 Zephyr ASDL (https://www.cs.princeton.edu/~appel/papers/asdl97.pdf )，
用于从同一套类型定义生成不同语言的数据结构定义、序列化以及反序列化。RSDL 主要引入了预处理指令
#include，以及注解语法

目前支持的后端有:
  - Rust (by @NEETovo)

计划支持的后端有:
  - TypeScript (class-based)
  - TypeScript (interface-based) + typeAssert
  - C++ + Qt + QtJSON

----

注解文档
  - 总则: 无论何时，注解的行为总是由生成器决定的，注解本身只是起到一种“建议”作用

通用注解

  - [boxed]
    在一些像 C++ 和 Rust 这样的语言中，创建互递归的数据结构需要使用指针，boxed 用来
    要求生成器为指定的字段生成指针类型，而不是直接嵌入类型定义
  - [doc("docstring")] 或者 [doc = "docstring"]
    要求生成器为指定的实体生成文档注释
  - [doc_ctor("docstring")] 或者 [doc_ctor = "docstring"]
    在 SUM 类型中，要求生成器为指定的构造器对应生成的那个数据类型生成文档注释
  - [private]
    要求生成器将指定的实体设为私有
