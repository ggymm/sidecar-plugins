# System Information Tool - 设计规划

定时输出系统信息的Rust工具，支持跨平台硬件和软件信息展示。

## 技术选型

### 主要依赖库

- **sysinfo** - 跨平台系统信息获取（推荐）
- **sys-info** - 备选系统信息库
- **clap** - 命令行参数解析
- **serde/serde_json** - JSON序列化输出
- **chrono** - 时间处理

### 参考工具分析

- **neofetch** - 经典系统信息工具（已停维）
- **fastfetch** - neofetch的现代替代品
- **glances** - 跨平台系统监控
- **htop** - 进程监控工具

## 系统信息分类

### 基础信息 (Static Info) - 不频繁更新

#### 硬件规格

- [ ] CPU型号名称 (sysinfo::Cpu::brand)
- [ ] CPU架构 (x86_64, arm64, etc.)
- [ ] CPU核心数 (物理核心数)
- [ ] CPU线程数 (逻辑核心数)
- [ ] CPU主频 (基础频率)
- [ ] CPU最大频率 (boost频率)
- [ ] 总内存大小 (sysinfo::System::total_memory)
- [ ] 交换分区总大小 (sysinfo::System::total_swap)

#### 存储设备规格

- [ ] 磁盘列表 (sysinfo::System::disks)
- [ ] 每个磁盘的总容量
- [ ] 磁盘类型 (SSD/HDD)
- [ ] 文件系统类型 (NTFS, ext4, APFS, etc.)
- [ ] 挂载点

#### 网络接口规格

- [ ] 网络接口列表 (sysinfo::Networks)
- [ ] MAC地址
- [ ] IP地址 (IPv4/IPv6)

#### 操作系统信息

- [ ] 操作系统名称 (sysinfo::System::name)
- [ ] 操作系统版本 (sysinfo::System::os_version)
- [ ] 内核版本 (sysinfo::System::kernel_version)
- [ ] 主机名 (sysinfo::System::host_name)
- [ ] 系统架构
- [ ] 系统启动时间

#### 运行环境

- [ ] 当前用户名
- [ ] 用户主目录
- [ ] Shell类型及版本
- [ ] 终端类型

### 实时信息 (Dynamic Info) - 需要定时更新

#### CPU动态状态

- [ ] CPU使用率 (实时百分比)
- [ ] CPU温度 (如果可获取)
- [ ] 系统负载平均值 (1分钟, 5分钟, 15分钟)

#### 内存动态状态

- [ ] 已用内存 (sysinfo::System::used_memory)
- [ ] 可用内存 (sysinfo::System::available_memory)
- [ ] 内存使用率百分比
- [ ] 交换分区已用 (sysinfo::System::used_swap)

#### 存储动态状态

- [ ] 每个磁盘的已用空间
- [ ] 每个磁盘的可用空间
- [ ] 磁盘I/O统计

#### 网络动态状态

- [ ] 网络数据传输统计
- [ ] 网络接口状态
- [ ] 网络I/O统计

#### 进程动态状态

- [ ] 总进程数
- [ ] 运行进程数
- [ ] 睡眠进程数
- [ ] 僵尸进程数
- [ ] 系统运行时长

#### 电源和传感器

- [ ] 电池状态 (笔记本电脑)
- [ ] 电源管理模式
- [ ] 系统温度传感器数据

## 跨平台兼容性考虑

### 支持平台

- ✅ Linux (x86_64, arm64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows (x86_64, arm64)
- ✅ FreeBSD

### 平台特定限制

- **Android**: 新版本限制非系统应用访问系统信息
- **Docker/WSL**: 无法获取宿主机硬件信息
- **虚拟机**: 某些硬件传感器信息不可用

### 信息获取优先级

1. **高优先级** - 基本系统信息 (OS, CPU, 内存)
2. **中优先级** - 存储和网络信息
3. **低优先级** - 温度传感器和电源信息

## 输出格式设计

### JSON格式 (默认)

```json
{
  "timestamp": "2025-08-30T10:30:00Z",
  "static": {
    "hardware": {
      "cpu": {
        "brand": "Apple M2 Pro",
        "architecture": "arm64",
        "physical_cores": 10,
        "logical_cores": 10,
        "base_frequency": 3200,
        "max_frequency": 3200
      },
      "memory": {
        "total": 17179869184,
        "total_swap": 0
      },
      "disks": [...],
      "network_interfaces": [...]
    },
    "software": {
      "os": {
        "name": "macOS",
        "version": "14.6.1",
        "kernel_version": "23.6.0",
        "hostname": "MacBook-Pro",
        "architecture": "arm64",
        "boot_time": "2025-08-30T08:00:00Z"
      },
      "environment": {...}
    }
  },
  "dynamic": {
    "cpu": {
      "usage_percent": 15.2,
      "temperature": 45.0,
      "load_average": [1.2, 1.5, 1.8]
    },
    "memory": {
      "used": 8589934592,
      "available": 8589934592,
      "usage_percent": 50.0,
      "swap_used": 0
    },
    "disks": [...],
    "network": [...],
    "processes": {
      "total": 342,
      "running": 2,
      "sleeping": 340,
      "zombie": 0
    },
    "uptime_seconds": 86400
  }
}
```

### 文本格式 (类似neofetch)

- 简洁美观的文本输出
- 支持颜色和图标
- 分类展示信息

## 命令行接口设计

### 基本用法

```bash
# 显示所有系统信息 (基础+实时)
./system

# 只显示基础信息 (不变的硬件规格)
./system --static

# 只显示实时信息 (动态变化的状态)
./system --dynamic

# JSON格式输出
./system --json

# 定时输出实时信息 (每5秒)
./system --dynamic --interval 5

# 输出到文件
./system --output system_info.json

# 一次性完整信息收集
./system --static --json --output specs.json
```

### 命令行参数

- `--static` - 只显示基础信息 (硬件规格、OS版本等)
- `--dynamic` - 只显示实时信息 (CPU使用率、内存使用等)
- `--json` - JSON格式输出
- `--interval <seconds>` - 定时输出间隔 (仅适用于动态信息)
- `--output <file>` - 输出到文件
- `--compact` - 紧凑显示模式
- `--verbose` - 详细信息模式

## 实现计划

### 第一阶段 - 基础功能

1. 集成sysinfo crate
2. 实现基本硬件信息获取
3. 实现操作系统信息获取
4. 命令行参数解析

### 第二阶段 - 完整功能

1. 网络和存储信息
2. JSON输出格式
3. 定时输出功能
4. 文本格式美化

### 第三阶段 - 优化增强

1. 性能优化
2. 错误处理完善
3. 跨平台测试
4. 配置文件支持