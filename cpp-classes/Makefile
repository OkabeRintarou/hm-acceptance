# C++ 类学习示例 —— 编译并运行全部示例
#
#   make          编译并运行所有示例
#   make build    只编译(产物统一放在 build/ 下)
#   make clean    清理产物

CXX     ?= g++
CXXFLAGS ?= -std=c++17 -Wall -Wextra -Werror
SOURCES := $(sort $(wildcard 0*.cpp))

.PHONY: all build run clean
all: build run

build:
	@mkdir -p build
	@for f in $(SOURCES); do \
		echo "$(CXX) $(CXXFLAGS) $$f"; \
		$(CXX) $(CXXFLAGS) $$f -o build/$${f%.cpp} || exit 1; \
	done

run: build
	@for f in $(SOURCES); do \
		echo "===== build/$${f%.cpp} ====="; \
		./build/$${f%.cpp} || exit 1; \
	done

clean:
	rm -rf build
