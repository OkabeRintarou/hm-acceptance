// 07 - 资源管理: RAII 与 Rule of Five(现代 C++ 核心)
// RAII: 把资源(内存/文件/锁)的生命周期绑定到对象生命周期上，
//       构造时获取、析构时释放 —— 即使抛异常也绝不泄漏。

#include <algorithm>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <utility>
#include <vector>

// ── Rule of Five: 只要你需要自定义下面之一，通常五个都要考虑 ──
//   析构函数 / 拷贝构造 / 拷贝赋值 / 移动构造 / 移动赋值
class Buffer {
public:
    explicit Buffer(size_t n) : size_(n), data_(new int[n]{}) {
        std::cout << "  构造: 持有 " << size_ << " 个 int\n";
    }

    // 1. 析构函数: 释放资源
    ~Buffer() {
        std::cout << "  析构: 释放 " << size_ << " 个 int\n";
        delete[] data_;
    }

    // 2. 拷贝构造: 深拷贝(默认的浅拷贝会让两个对象 delete 同一块内存!)
    Buffer(const Buffer& other) : size_(other.size_), data_(new int[other.size_]) {
        std::copy(other.data_, other.data_ + size_, data_);
        std::cout << "  拷贝构造: 深拷贝 " << size_ << " 个 int\n";
    }

    // 3. 拷贝赋值: copy-and-swap 惯用法 —— 自赋值安全、异常安全
    Buffer& operator=(const Buffer& other) {
        std::cout << "  拷贝赋值\n";
        Buffer tmp(other);                    // 先拷出临时副本
        swap(tmp);                            // 再和副本交换
        return *this;                         // 离开时 tmp 析构旧资源
    }

    // 4. 移动构造: 「窃取」临时对象的资源，原对象置空 —— 避免深拷贝
    Buffer(Buffer&& other) noexcept           // noexcept 很重要，vector 扩容才会用移动
        : size_(other.size_), data_(other.data_) {
        other.data_ = nullptr;
        other.size_ = 0;
        std::cout << "  移动构造: 窃取资源，原对象置空\n";
    }

    // 5. 移动赋值
    Buffer& operator=(Buffer&& other) noexcept {
        std::cout << "  移动赋值\n";
        if (this != &other) {
            delete[] data_;                   // 先释放自己现有的资源
            data_ = other.data_;
            size_ = other.size_;
            other.data_ = nullptr;
            other.size_ = 0;
        }
        return *this;
    }

    void swap(Buffer& other) noexcept {
        std::swap(size_, other.size_);
        std::swap(data_, other.data_);
    }

    size_t size() const { return size_; }
    int& operator[](size_t i) { return data_[i]; }

private:
    size_t size_;
    int* data_;
};

// ── RAII 守卫: 用对象管理任意资源 ────────────────────────────
class ScopedMessage {
public:
    explicit ScopedMessage(const char* msg) : msg_(msg) {
        std::cout << "  进入: " << msg_ << '\n';
    }
    ~ScopedMessage() { std::cout << "  离开: " << msg_ << '\n'; }
    ScopedMessage(const ScopedMessage&) = delete;             // C++11: 显式禁止拷贝
    ScopedMessage& operator=(const ScopedMessage&) = delete;

private:
    const char* msg_;
};

int main() {
    std::cout << "-- 拷贝 vs 移动 --\n";
    Buffer a(3);
    Buffer b = a;                             // 深拷贝
    Buffer c = std::move(a);                  // 移动: a 之后不再有效，不要再使用
    std::cout << "  b.size = " << b.size() << ", c.size = " << c.size() << '\n';

    std::cout << "-- vector 扩容时优先移动 --\n";
    std::vector<Buffer> vec;
    vec.reserve(1);                           // 先reserve，避免扩容干扰演示
    vec.emplace_back(2);
    vec.push_back(Buffer(5));                 // 临时对象被移动进容器，而非拷贝

    std::cout << "-- RAII 与异常 --\n";
    try {
        ScopedMessage guard("try 块");
        throw std::runtime_error("出错了");
    } catch (const std::exception& e) {
        std::cout << "  捕获: " << e.what() << '\n';
    }                                         // 即使异常跳过正常返回路径，guard 也被析构

    std::cout << "-- 智能指针: 别再手写 new/delete --\n";
    auto owned = std::make_unique<Buffer>(4); // 独占所有权，离开作用域自动释放
    std::shared_ptr<Buffer> shared = std::move(owned);  // 转移所有权
    std::cout << "  引用计数 = " << shared.use_count() << '\n';
    return 0;
}
