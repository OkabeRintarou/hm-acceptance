// 02 - 构造函数与析构函数
// 构造函数：对象创建时自动调用，负责初始化。
// 析构函数：对象销毁时自动调用，负责清理资源。

#include <iostream>
#include <string>
#include <vector>

class Player {
public:
    // 默认构造函数：不传参数时使用
    Player() : Player("无名玩家", 1) {   // 委托构造(C++11)：复用另一个构造函数
        std::cout << "  (默认构造)\n";
    }

    // 带参构造函数；冒号后是「成员初始化列表」，比在函数体内赋值更高效
    Player(std::string name, int level)
        : name_(std::move(name)),        // 直接构造成员，避免先默认再赋值
          level_(level) {
        std::cout << "  构造: " << name_ << " Lv." << level_ << '\n';
    }

    // 拷贝构造函数：用另一个同类型对象初始化自己
    // 形参必须是引用，否则「拷贝实参」本身又要调用拷贝构造，无限递归
    Player(const Player& other)
        : name_(other.name_), level_(other.level_) {
        std::cout << "  拷贝构造: " << name_ << '\n';
    }

    // 析构函数：名字是 ~类名，无参数无返回值；每个类最多一个
    ~Player() {
        std::cout << "  析构: " << name_ << '\n';
    }

    void show() const { std::cout << name_ << " Lv." << level_ << '\n'; }

private:
    std::string name_;
    int level_;
};

// 演示拷贝发生的时机：按值传参 / 按值返回 都会拷贝
static void by_value(Player) {     // 参数不需要名字: 只为了演示拷贝构造/析构的时机
    std::cout << "  函数内\n";
}

int main() {
    std::cout << "-- 构造与析构顺序 --\n";
    Player a;                                 // 默认构造
    Player b("亚历克斯", 30);                 // 带参构造
    Player c = b;                             // 拷贝构造(等价于 Player c(b))

    std::cout << "-- 按值传参触发拷贝 --\n";
    by_value(b);

    std::cout << "-- 容器中的对象 --\n";
    std::vector<Player> team;
    team.reserve(2);                          // 预留空间，避免扩容时的拷贝/移动
    team.emplace_back("泰勒", 12);            // 在容器内直接构造，不产生临时对象
    team.push_back(Player("山姆", 8));        // 构造临时对象 -> 移动/拷贝进容器 -> 析构临时

    std::cout << "-- 离开作用域，逆序析构 --\n";
    return 0;
    // main 结束时：局部对象按构造的相反顺序依次析构
}
