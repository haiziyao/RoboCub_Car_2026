## 未开发的功能
* device端，需要什么样式的device需要自己定义




## 目前在部分逻辑严重错误


* 在Fn的传递中，之前使用了OnceFn。这个十分不便于维护和使用
* 对于自定义的功能函数，我们要不要给一个
* 对于Img2Base64，这里目前仍然使用url的方式，后面加上cv或者其他的时候必须得改了
* 目前对于Config中的子config的命名，很烦躁，以后可能会改掉的吧



### 开发日记

#### 2026.5.12

> 旧版 `car_cv` 功能同步到新版架构

已完成：
* 同步旧版颜色识别：OpenCV 读取摄像头、圆形 ROI、HSV 筛选、面积比例筛选、连续稳定计数。
* 同步旧版二维码识别：OpenCV 灰度预处理 + `quircs` 解码。
* 同步旧版 UART 通信：`UartSource` 根据 `source_key` 监听电控命令，再通过配置分发到 `device_id + function_id`。
* 同步旧版串口结果回写：视觉函数执行完成后使用进程级唯一 UART 配置写回。
* 同步旧版 GPIO 状态灯：颜色/二维码任务运行时拉低对应任务灯和运行灯，结束后恢复高电平。
* Web 消息输出已接入：函数结果统一返回 `WebMessage`；Web 关闭时会 drain 消息并记录日志，避免执行端阻塞。
* 配置已改为新版驱动方式：`bindings.toml` 决定命令到任务，`device.toml` 决定 UART 和摄像头硬件实例，`func_param.toml` 决定功能注册和功能参数。
* 参数解析修正：`vision.rs` 不再为必要参数提供默认值；缺失、格式错误、范围错误会注册为无效设备或返回函数错误，最终进入日志和 WebMessage，不允许因为参数问题退出整个程序。
* 串口配置修正：`device_param_config.uart_config` 是整个进程唯一串口配置；`bindings.toml` 只负责命令到任务映射，视觉设备 args 不再携带串口参数。
* 颜色配置修正：颜色识别不再写死 5 个 HSV 字段，改为按 `color.<name>=H_min,H_max,S_min,S_max,V_min,V_max` 配置条目动态注册。
* 摄像头配置修正：颜色、二维码、路口识别不再注册为不同摄像头设备；设备侧统一为 `Camera`，HSV、debug、loop、ROI、灯光等参数归属对应 function。
* Web 状态修正：`/history` 只读取内存快照；消息进入 `WebState` 时统一补 `id/created_at_ms`，并通过独立 writer 追加到 `logs/web_messages.jsonl`，启动时恢复最近历史。

保留说明：
* 旧版 README 提到 `cross_detect`，但旧代码没有对应实现；新版已注册 `Camera + cross_detect` 占位，调用时会返回明确的未实现错误消息。

#### 2026.4.7 

> 在受不了了之后的第一次吐槽

真的，心累，这个框架涉及到好几个部分的联通，有很多地方涉及到统一回执，各个地方得协调，整个流程的调用链路也是极其夸张

可以这样说，我要死了，每次开始写这个项目，我都需要deep-thing for an hour 

骇死我力！！！没办法，只能肝了
