// 03 - 封装：访问控制与类不变量
// 封装(encapsulation)的意义：把数据藏起来，通过方法暴露「受控的访问」，
// 从而保证「类不变量」—— 任何时刻对象都必须满足的规则。

#include <iostream>
#include <stdexcept>
#include <string>

class BankAccount {
public:
    explicit BankAccount(std::string owner, long balance_cents = 0)
        : owner_(std::move(owner)), balance_cents_(balance_cents) {}

    // 存款：金额必须为正 —— 规则检查只写在这一处
    void deposit(long cents) {
        if (cents <= 0) throw std::invalid_argument("存款金额必须为正");
        balance_cents_ += cents;
    }

    // 取款：不能透支
    void withdraw(long cents) {
        if (cents <= 0) throw std::invalid_argument("取款金额必须为正");
        if (cents > balance_cents_) throw std::runtime_error("余额不足");
        balance_cents_ -= cents;
    }

    // 只读接口(getter)：注意返回 const 引用，避免拷贝字符串
    const std::string& owner() const { return owner_; }
    long balance() const { return balance_cents_; }

private:
    std::string owner_;
    long balance_cents_;      // 不变量: 永远 >= 0
};

// ── 三种访问级别 ────────────────────────────────────────────
// public    : 谁都能访问
// private   : 只有本类(和友元)能访问
// protected : 本类 + 派生类能访问(见 05)
class Base {
public:
    int pub = 1;
protected:
    int prot = 2;
private:
    int priv = 3;             // 派生类也访问不了
};

class Derived : public Base {
public:
    int sum() const { return pub + prot; }   // 可以用 pub 和 prot
    // int bad() const { return priv; }      // 编译错误：private 不可见
};

int main() {
    BankAccount account("李雷", 10000);

    // account.balance_cents_ = -1;   // 编译错误！数据被保护，不可能被意外破坏
    account.deposit(500);
    std::cout << account.owner() << " 余额: " << account.balance() << " 分\n";

    try {
        account.withdraw(999999);
    } catch (const std::exception& e) {
        std::cout << "取款失败: " << e.what() << '\n';
    }
    std::cout << "余额不变: " << account.balance() << " 分\n";

    Derived d;
    std::cout << "Derived::sum() = " << d.sum() << '\n';
    std::cout << "外部可访问 d.pub = " << d.pub << '\n';
    return 0;
}
