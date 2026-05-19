## RuboVision Engine

新版采用 `Source -> TaskListener -> TaskDispatcher -> TaskExecutor -> WebMessage` 的消息链路。
功能不再写死在主循环里，而是由配置决定触发源、设备和函数。

## 配置入口

* `config/bindings.toml`：配置来源命令到任务的映射，例如 UART 收到 `a1` 调用颜色识别，收到 `b2` 调用二维码识别。
* `config/device.toml`：配置进程级唯一 UART 和摄像头实例；摄像头统一使用 `Camera`，只描述 `path` 等硬件参数。
* `config/func_param.toml`：配置启动时注册哪些功能函数，以及每个功能自己的识别参数。
  颜色识别使用 `color.<name>=H_min,H_max,S_min,S_max,V_min,V_max`，颜色数量由配置条目数量决定。
  每个函数可以通过 `returns = { web = true, gpio = true }` 控制是否把结果送到 Web 通道、是否启用 GPIO 返回/状态输出；GPIO 引脚号仍放在对应函数的 `args` 里。
* `config/web.yaml`：配置 Web 调试面板是否开启。

## 已同步旧版功能

* 颜色识别：OpenCV 摄像头读取、圆形 ROI、HSV 筛选、面积比例筛选、连续稳定计数。
* 二维码识别：OpenCV 灰度预处理 + `quircs` 解码。
* UART 通信：`UartSource` 根据 `source_key` 监听电控命令并分发任务。
* UART 回写：识别结果使用 `device_param_config.uart_config` 这一份全局串口配置写回。
* GPIO 状态灯：任务执行期间拉低对应状态灯，结束后恢复。
* Web 消息：函数统一返回 `WebMessage`，Web 关闭时消息会被消费并写日志，避免执行端阻塞。

## 当前默认绑定

* `a1 -> color_camera + color_detect`
* `b2 -> qr_camera + qr_detect`

`cross_detect` 目前保留 `Camera + cross_detect` 占位，返回默认值 `0`，后续再接黑色轮廓识别。
