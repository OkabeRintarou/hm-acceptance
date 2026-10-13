# C++ 类(class)使用教程

每份示例都是**独立可编译运行**的完整程序，重点都写在代码注释里。建议按编号顺序阅读，边看边跑：

```bash
make            # 编译并运行所有示例
g++ -std=c++17 -Wall -Wextra 01_basic_class.cpp -o 01 && ./01   # 单个运行
```

| 文件 | 内容 |
|---|---|
| `01_basic_class.cpp` | 类的定义、成员变量/函数、`class` 与 `struct`、栈/堆对象 |
| `02_constructors.cpp` | 构造函数、析构函数、初始化列表、拷贝构造的触发时机 |
| `03_access_control.cpp` | 封装、`public/protected/private`、类不变量、getter/setter |
| `04_static_const_this.cpp` | `const` 成员函数、`static` 成员、`this` 指针、`mutable` |
| `05_inheritance.cpp` | 继承、`virtual`/`override`、纯虚函数与抽象类、虚析构函数 |
| `06_operator_overloading.cpp` | 运算符重载(成员 vs 自由函数)、`friend` |
| `07_raii_rule_of_five.cpp` | RAII、Rule of Five、移动语义、智能指针 |

---

## 一句话总结

**类 = 数据(成员变量) + 行为(成员函数) + 访问控制**。它让你定义自己的类型，把「数据」和「保证数据合法的规则」绑在一起。

## 1. 定义与使用

```cpp
class Dog {
public:
    void speak() const { std::cout << name << ": 汪!\n"; }
    std::string name;
    int age = 0;              // 类内默认初始化，避免未定义值
};

Dog d{"旺财", 3};             // 栈上对象
d.speak();
auto p = std::make_unique<Dog>("来福", 2);   // 堆上交给智能指针管理
p->speak();
```

- `class` 与 `struct` 唯一区别：默认访问级别(`private` vs `public`)。C++ 风格建议：纯数据聚合用 `struct`，有不变量/行为用 `class`。
- 通过 `.` 访问对象成员，通过 `->` 访问指针成员(`p->x` 等价 `(*p).x`)。

## 2. 构造与析构

```cpp
class Player {
public:
    Player() : Player("无名", 1) {}          // 委托构造
    Player(std::string n, int lv) : name_(std::move(n)), level_(lv) {}  // 初始化列表
    Player(const Player& o) : name_(o.name_), level_(o.level_) {}       // 拷贝构造
    ~Player() {}                              // 析构：释放资源
private:
    std::string name_;
    int level_;
};
```

要点：

- **初始化列表**(`: name_(...)`)直接构造成员，比函数体内赋值更高效；`const` 成员和引用成员**只能**用它初始化。
- 对象销毁时机：离开作用域、`delete`、容器销毁。析构顺序与构造顺序**相反**。
- 按值传参/按值返回会触发拷贝构造；不想拷贝就传 `const T&`。

## 3. 封装与访问控制

| 级别 | 类内 | 派生类 | 类外 |
|---|---|---|---|
| `public` | ✅ | ✅ | ✅ |
| `protected` | ✅ | ✅ | ❌ |
| `private` | ✅ | ❌ | ❌ |

封装的意义不是「保密」，而是**保证不变量**：例如 `BankAccount` 的余额永不为负——把数据设为 `private`，所有修改都经过带检查的方法，错误使用根本无法编译通过。

## 4. `const`、`static`、`this`

- **`const` 成员函数**承诺不修改对象，`const` 对象只能调用 `const` 函数。只读接口一律标 `const`。
- **`static` 成员**属于类而非对象：全体对象共享一份数据；`static` 成员函数没有 `this`，只能访问 `static` 成员。
- **`this`** 指向当前对象；`return *this` 实现链式调用(`a += b += c`)。
- **`mutable`** 成员：即使在 `const` 函数里也能改(典型：缓存、互斥锁)。

## 5. 继承与多态

```cpp
class Shape {
public:
    virtual double area() const = 0;   // 纯虚函数 => 抽象类
    virtual ~Shape() = default;        // 有虚函数必给虚析构!
};
class Circle : public Shape {
public:
    double area() const override { return 3.14159 * r_ * r_; }
private:
    double r_;
};

std::vector<std::unique_ptr<Shape>> shapes;   // 面向基类编程
shapes.push_back(std::make_unique<Circle>(2));
shapes[0]->area();   // 运行时决定调用 Circle::area
```

- **`virtual`** 开启动态绑定：通过基类指针/引用调用时，执行对象**真实类型**的版本。
- **`override`** 必写：让编译器核对你确实覆盖了基类函数。
- **纯虚函数**(`= 0`)使类成为抽象类，不能实例化，强制派生类实现接口。
- **虚析构函数**：只要类会被多态地删除(`delete base_ptr`)，基类析构必须是 `virtual`，否则只析构基类部分，是未定义行为。
- 继承表达 **is-a**(圆是一种形状)；组合表达 **has-a**(车有引擎)，能用组合就不用继承。

## 6. 运算符重载与友元

```cpp
class Vec2 {
public:
    Vec2 operator+(const Vec2& r) const { return Vec2{x_+r.x_, y_+r.y_}; }
    Vec2& operator+=(const Vec2& r) { x_+=r.x_; y_+=r.y_; return *this; }
    bool operator==(const Vec2& r) const { return x_==r.x_ && y_==r.y_; }
    friend std::ostream& operator<<(std::ostream& os, const Vec2& v);
private:
    double x_, y_;
};
std::ostream& operator<<(std::ostream& os, const Vec2& v) { return os << "(" << v.x_ << ", " << v.y_ << ")"; }
```

- 左操作数是本类对象 → 可写成**成员函数**；需要对称性或左操作数不是本类(`<<`、`2.5 * v`)→ 写成**自由函数**。
- **`friend`** 授权某个函数/类访问自己的 `private`。它由类自己声明，是受控的封装例外，别滥用。

## 7. RAII 与 Rule of Five —— 现代 C++ 的灵魂

**RAII**：资源获取即初始化——构造函数拿资源，析构函数放资源。作用域结束(包括异常路径)资源必定释放，这就是 C++ 没有 `finally` 的原因：析构函数就是 `finally`。

**Rule of Five**：类若管理资源(自定义析构)，五个特殊成员要成套考虑——析构、拷贝构造、拷贝赋值、移动构造、移动赋值(见 `07_raii_rule_of_five.cpp` 的 `Buffer`)。

实践守则：

- **不管理资源就什么都不写**，五个全用编译器默认的(Rule of Zero，绝大多数类应如此)。
- 明确不要某行为就用 `= delete` 禁掉；要默认行为就 `= default`。
- 日常编程用 `std::unique_ptr`(独占)/`std::shared_ptr`(共享)管理堆内存，**手写 `new`/`delete` 应当罕见**。
- 移动构造/移动赋值标记 `noexcept`，否则 `vector` 扩容会退化为拷贝。

## 推荐的学习路径

1. 跑通 01–04，掌握**值语义**的类：构造、析构、拷贝、封装。
2. 跑通 05，理解**多态**：虚函数表、抽象接口、面向基类编程。
3. 跑通 06–07，进入**现代 C++**：运算符重载、移动语义、RAII、智能指针。
4. 延伸阅读：《C++ Primer》第 7、13、15 章；cppreference 上 [classes](https://en.cppreference.com/w/cpp/language/classes) 页面。
