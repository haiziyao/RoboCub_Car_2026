## RoboCup 2026 视觉代码
### 项目简介
2026 RoboCup车型竞技机器人视觉组代码。部署在树莓派5的 Rust + OpenCV项目。集成颜色识别，二维码识别，GPIO状态灯，UART串口通信，cross检测。

### 项目版本简介
本项目维护有两个大的子版本偶系列和奇系列，此外还有仅供参考的0系列。

目前已发布的0.1.0系项目的第一个版本，是整个项目的概览。偶系列和奇系列的版本更迭不受0版本主宰，但0版本发布新版本时，奇偶版本需要做出大版本更新。

#### 奇版本
* 功能与设备相关联：在项目启动时，设备进行硬注册，后续每次调用相同功能所使用的设备相同

#### 偶版本
* 功能与设备不关联:采用动态代理解耦，在项目启动时，设备进行软注册，后续根据通信信息进行调用

### 开发部署
#### 开发者环境
* ubuntu 22.04
* opencv_version: 4.5.4

##### 相关命令参考

``` bash
# 查看系统各种信息
hostnamectl
# 查看opencv版本
opencv_version

```

#### 推荐正常部署流程
* 建议重装系统
* 本项目采用树莓派5 (Debin13 Trixie版本而非Debian12 bookworm版本)
* 由于上一条的原因，各种库下载极其缓慢，所以我推荐使用代理
* 开发时我使用的是llvm-14.0.0,openCv 0.75的rust Crate，但是在部署树莓派的时候发现，默认安装的是新版的llvm,所以在部署的时候认为调整了opencv的版本。
* 遇到llvm版本不协调问题，注意优先修改Rust的OpenCV crate版本，不要擅自修改openCV SDK的版本(在系统中混装两个版本的OpenCV就是自讨苦吃)
``` bash
# OpenCV，-E表示不清空代理配置 
sudo -E apt install -y \
  build-essential pkg-config cmake git \
  clang libclang-dev llvm \
  libopencv-dev \
  v4l-utils

# 安装rust
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustc -V
cargo -V

# 查看自己的llvm是什么版本
# 注意如果版本过低或者过高出现报错，请切换openCV crate的版本
llvm-config --version

# 运行
cargo build

```
