// 06 - 运算符重载与友元
// 让自定义类型像内置类型一样使用 + - == < << 等运算符。
// 本质是函数：名字是 operatorX，可以写成成员函数或普通(自由)函数。

#include <iostream>

class Vec2 {
public:
    Vec2(double x = 0, double y = 0) : x_(x), y_(y) {}

    // ── 成员函数形式：左操作数必须是本类对象，隐式传入 this ──
    Vec2 operator+(const Vec2& rhs) const {   // a + b 等价于 a.operator+(b)
        return Vec2{x_ + rhs.x_, y_ + rhs.y_};
    }
    Vec2 operator-() const {                  // 一元负号: -a
        return Vec2{-x_, -y_};
    }
    Vec2& operator+=(const Vec2& rhs) {       // 复合赋值惯例: 返回*this，返回引用
        x_ += rhs.x_;
        y_ += rhs.y_;
        return *this;
    }

    // 惯例: == 和 != 互相配套，< 基于 ==
    bool operator==(const Vec2& rhs) const { return x_ == rhs.x_ && y_ == rhs.y_; }
    bool operator!=(const Vec2& rhs) const { return !(*this == rhs); }

    double x() const { return x_; }
    double y() const { return y_; }

    // 友元声明：允许下面的自由函数访问本类的 private 成员
    // (不破坏封装的度：它是这个类「自己授权」的访问者)
    friend std::ostream& operator<<(std::ostream& os, const Vec2& v);

private:
    double x_, y_;
};

// ── 自由函数形式 ────────────────────────────────────────────
// operator<< 的左操作数是 std::ostream，不可能是成员函数，只能写成自由函数
std::ostream& operator<<(std::ostream& os, const Vec2& v) {
    return os << '(' << v.x_ << ", " << v.y_ << ')';   // 能访问 private，因为是友元
}

// 对称的二元运算符(如 标量*向量)写成自由函数，允许左操作数发生隐式转换
static Vec2 operator*(double k, const Vec2& v) {
    return Vec2{k * v.x(), k * v.y()};
}

// ── friend 函数：不是成员，但可以访问 private ───────────────
class Secret {
    friend void peek(const Secret&);          // 授权 peek 函数
public:
    explicit Secret(int v) : value_(v) {}
private:
    int value_;
};

// 注意: 友元首次声明在类里，链接性由类外这次定义决定；本例单文件，无需 static
void peek(const Secret& s) {
    std::cout << "友元函数读到 private: " << s.value_ << '\n';
}

int main() {
    Vec2 a{1, 2};
    Vec2 b{3, 4};

    std::cout << "a + b = " << a + b << '\n';
    std::cout << "-a    = " << -a << '\n';
    std::cout << "2.5*b = " << 2.5 * b << '\n';

    a += b;                                    // 链式: (a += b) 返回 a 自身
    std::cout << "a += b => " << a << '\n';
    std::cout << "a == b ? " << (a == b ? "yes" : "no") << '\n';

    peek(Secret(42));
    return 0;
}
