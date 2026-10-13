// 05 - 继承与多态(面向对象的核心)
// 继承: 派生类获得基类的成员，表达「is-a」关系。
// 多态: 通过基类指针/引用调用虚函数时，实际执行的是对象真实类型的版本。

#include <iostream>
#include <memory>
#include <vector>

class Shape {
public:
    // 虚函数: 允许派生类覆盖(override)，调用时按对象真实类型分派
    virtual std::string name() const { return "形状"; }

    // 纯虚函数: "= 0" 表示没有默认实现，派生类必须实现
    virtual double area() const = 0;

    // 非虚函数: 所有派生类共享同一个实现，不参与多态
    void describe() const {
        std::cout << name() << " 面积 = " << area() << '\n';  // 内部调用也走虚分派
    }

    // 虚析构函数！通过基类指针 delete 派生类对象时，
    // 若析构函数不是 virtual，只会调用基类析构 => 资源泄漏(未定义行为)。
    // 规则: 只要类有虚函数，就给它 virtual 析构函数。
    virtual ~Shape() = default;
};

class Circle final : public Shape {           // final: 禁止再被继承
public:
    explicit Circle(double r) : r_(r) {}

    // override 关键字(C++11): 让编译器检查「确实覆盖了基类虚函数」，
    // 拼错名字或签名不一致会直接报编译错误。写虚函数一律带上它。
    std::string name() const override { return "圆形"; }
    double area() const override { return 3.14159265358979 * r_ * r_; }

private:
    double r_;
};

class Rect : public Shape {
public:
    Rect(double w, double h) : w_(w), h_(h) {}
    std::string name() const override { return "矩形"; }
    double area() const override { return w_ * h_; }

protected:
    double w_, h_;
};

class Square : public Rect {                  // 继承链可以多层: Square is-a Rect is-a Shape
public:
    explicit Square(double side) : Rect(side, side) {}
    std::string name() const override { return "正方形"; }
};

// 多态的实际价值: 面向基类编程，新增形状无需修改这个函数
static void print_all(const std::vector<std::unique_ptr<Shape>>& shapes) {
    for (const auto& s : shapes) {
        s->describe();        // 同一行代码，运行时调用不同派的实现
    }
}

int main() {
    std::vector<std::unique_ptr<Shape>> shapes;   // 智能指针管理，自动 delete
    shapes.push_back(std::make_unique<Circle>(2.0));
    shapes.push_back(std::make_unique<Rect>(3.0, 4.0));
    shapes.push_back(std::make_unique<Square>(5.0));
    print_all(shapes);

    // Shape s;              // 编译错误: 含纯虚函数的类是抽象类，不能实例化
    Shape* p = shapes[0].get();               // 裸指针只是「观察」，不负责释放
    std::cout << "通过基类指针调用: " << p->name() << '\n';
    return 0;
}
