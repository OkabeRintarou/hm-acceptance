// 01 - 类的基本定义与使用
// 编译: g++ -std=c++17 -Wall -Wextra 01_basic_class.cpp -o 01 && ./01
//
// 类(class)是把「数据」和「操作这些数据的函数」打包在一起的工具。
// 类定义了一个「类型」，对象(object)是这个类型的一个「实例」。

#include <iostream>
#include <string>

// ── 最简单的类 ─────────────────────────────────────────────
// class 默认成员是 private，struct 默认成员是 public，其余完全相同。
class Dog {
public:                       // public: 外部代码可以访问
    // 成员函数：定义在类内部的行为
    void introduce() const {  // const 表示该函数不修改成员变量(见 04)
        std::cout << "我叫 " << name << "，今年 " << age << " 岁\n";
    }

    std::string name;         // 成员变量(属性)
    int age = 0;              // 类内默认初始化(C++11)，避免未初始化
};

// ── 带行为的类：数据被 private 保护，只能通过方法访问 ──────
class Counter {
public:
    void increment() { ++count_; }
    int  value() const { return count_; }
    void reset() { count_ = 0; }

private:
    int count_ = 0;           // 约定：私有成员以下划线结尾
};

int main() {
    // 对象创建在栈上，就像普通变量一样
    Dog dog;
    dog.name = "旺财";        // Dog 的成员是 public，可以直接赋值
    dog.age = 3;
    dog.introduce();

    // 也可以用初始化列表聚合初始化(要求成员是 public，按声明顺序)
    Dog dog2{"小黑", 5};
    dog2.introduce();

    // 堆上创建：用 new，记得 delete；推荐改用智能指针(见 07)
    Dog* heap_dog = new Dog{"来福", 2};
    heap_dog->introduce();    // 通过指针访问成员用 ->
    delete heap_dog;          // 必须 delete，否则内存泄漏

    // Counter 的数据是 private 的，外部无法直接改：
    // counter.count_ = -999;  // 编译错误！只能通过公有接口操作
    Counter counter;
    counter.increment();
    counter.increment();
    std::cout << "计数: " << counter.value() << '\n';
    counter.reset();
    std::cout << "重置后: " << counter.value() << '\n';
    return 0;
}
