// 04 - const 成员函数、static 成员与 this 指针

#include <iostream>
#include <string>
#include <vector>

class Player {
public:
    Player(std::string name) : name_(std::move(name)) {
        ++alive_count_;                       // 每创建一个对象，计数 +1
    }

    ~Player() { --alive_count_; }             // 每销毁一个对象，计数 -1

    // const 成员函数：承诺不修改任何成员变量(编译器强制)。
    // 只有 const 成员函数才能被 const 对象调用。
    const std::string& name() const { return name_; }
    // int bad() const { return ++serial_; }  // 编译错误：const 函数不能改成员

    // 非 const 成员函数：会修改对象状态
    void rename(const std::string& new_name) { name_ = new_name; }

    // static 成员函数：属于类而非对象，没有 this，不能访问普通成员
    static int alive() { return alive_count_; }

    // static 成员变量：整个类共享一份，必须在类外定义(见下方)
    static int alive_count_;

private:
    std::string name_;
    inline static int serial_ = 0;            // C++17 起可在类内初始化
    int id_ = ++serial_;
};

int Player::alive_count_ = 0;                 // 类外定义 static 数据成员

// ── mutable：即使在 const 函数里也可以改 ────────────────────
class Cached {
public:
    int data() const {
        // 模拟「缓存计算结果」：逻辑上不改变对象，只是缓存
        ++lookups_;                           // lookups_ 是 mutable，合法
        return 42;
    }
    int lookups() const { return lookups_; }

private:
    mutable int lookups_ = 0;                 // mutable: 在 const 函数中可修改
};

int main() {
    // this 指针：每个非静态成员函数里都隐含 this，指向当前对象。
    // name() 其实等价于 this->name_。返回 *this 可以实现链式调用：
    Player p("凯尔");
    p.rename("凯尔·凯特尼姆");                // p.rename(...) 里的 p 就是 this 指向的对象
    std::cout << p.name() << '\n';

    // static 成员通过「类名::」访问，不需要对象
    std::cout << "存活玩家: " << Player::alive() << '\n';
    {
        Player a("安娜");
        Player b("鲍勃");
        std::cout << "存活玩家: " << Player::alive() << '\n';
    }                                         // a、b 离开作用域被析构
    std::cout << "存活玩家: " << Player::alive() << '\n';

    // const 对象只能调用 const 成员函数
    const Player readonly("只读玩家");
    std::cout << readonly.name() << '\n';     // OK: name() 是 const
    // readonly.rename("x");                  // 编译错误: rename() 不是 const

    Cached c;
    c.data();
    c.data();
    std::cout << "查询次数: " << c.lookups() << '\n';
    return 0;
}
