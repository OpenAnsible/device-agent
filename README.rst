Device Agent
=================

:Date: 05/09 2016



.. contents::


Access and control devices from anywhere with web browsers, mobile or tablet apps ...


介绍
-------

设备代理（Device Agent）允许你向远端分享你的设备，使远端可以读写你的设备，即:

*   查看你的设备     (如: 查看摄像头，查看屏幕，读取硬盘，跟踪鼠标位置，监视键盘输出 ...)
*   对设备进行写操作（如：硬盘写入，设定鼠标位置，模拟键盘输出，控制Shell ...）


编译
------

`OS X` 下面的 `C Lib` 问题:

.. code:: bash
    
    export C_INCLUDE_PATH=/usr/local/include
    export DYLD_LIBRARY_PATH=/usr/local/lib
    export LIBRARY_PATH=/usr/local/lib

