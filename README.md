# Dyadikos

Dyadikos is a game engine written in C++26.
It is made to be simple and easy-to-use.

## Usage

[See the basic example for more information](src/main.cpp).

## Compiling

You need a C++26 compatible compiler such as Clang, as well as Ninja and CMake 4.x.

```sh
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++
cmake --build build
```
