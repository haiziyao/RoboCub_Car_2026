``` bash
sudo apt install socat
sudo socat PTY,link=/dev/ttyV0,mode=777 PTY,link=/dev/ttyV1,mode=777

screen /dev/ttyV1 9600
screen /dev/ttyAMA0 9600 -L # 禁用回显
```

/dev/ttyAMA0 是树莓派上的默认串口设备（通常使用 GPIO14 (TX) 和 GPIO15 (RX) 引脚