# FUSE(Filesystem in Userspace,用户空间文件系统)
## FUSE概述
FUSE是一个实现在用户空间的文件系统框架，通过FUSE内核模块的支持，使用者只需要根据fuse提供的接口实现具体的文件操作就可以实现一个文件系统。 
在fuse出现以前，Linux中的文件系统都是完全实现在内核态，仅仅在现有传统文件系统上添加一个小小的功能也很困难，因为是在内核中实现仍需要做很大的工作量。fuse本身是内核提供的一个功能，内核开启fuse支持后，会在/dev目录下，生成fuse设备节点，应用层可以通过该设备节点完成用户态文件系统。
libfuse是一个对fuse功能封装的库，提供一系列的api，使用户可以更方便、更简单的使用fuse功能。编写FUSE文件系统时，只需要内核加载了fuse内核模块即可，不需要重新编译内核。
## FUSE组成
fuse主要由三部分组成：FUSE内核模块、用户空间库libfuse以及挂载工具fusermount。

- fuse内核模块：实现了和VFS的对接，实现了一个能被用户空间进程打开的设备，当VFS发来文件操作请求之后，将请求转化为特定格式，并通过设备传递给用户空间进程，用户空间进程在处理完请求后，将结果返回给fuse内核模块，内核模块再将其还原为Linux kernel需要的格式，并返回给VFS。
- fuse库libfuse：负责和内核空间通信，接收来自`/dev/fuse`的请求，并将其转化为一系列的函数调用，将结果写回到`/dev/fuse`；提供的函数可以对fuse文件系统进行挂载卸载、从linux内核读取请求以及发送响应到内核。libfuse提供了两个APIs：一个“high-level”同步API 和一个“low-level” 异步API 。这两种API 都从内核接收请求传递到主程序（fuse_main函数），主程序使用相应的回调函数进行处理。当使用high-level
 API时，回调函数使用文件名（file names）和路径（paths）工作，而不是索引节点inodes，回调函数返回时也就是一个请求处理的完成。使用low-level API 时，回调函数必须使用索引节点inode工作，响应发送必须显示的使用一套单独的API函数。
- 挂载工具：实现对用户态文件系统的挂载。
## FUSE主要代码文件
in kernel:
- `kernel/inode.c` —> 主要完成fuse文件驱动模块的注册，提供对supper block的维护函数以及其它(驱动的组织开始文件)
- `kernel/dev.c` —> fuse 的(虚拟)设备驱动
- `kernel/control.c` —> 提供对于dentry的维护及其它
- `kernel/dir.c` —> 主要提供对于目录inode索引节点的维护
- `kernel/file.c` —> 主要提供对于文件inode索引节点的维护

in userspace：

- `lib/helper.c` —> “fuse_main()”调用的主入口
- `lib/fuse_kern_chan.c`—>主要实现fuse应用层访问(读写)fuse driver的功能
- `lib/fuse_mt.c` —> fuse 的mount管理
- `lib/fuse.c` —> lib库主框架文件，实现了主要框架及对”用户实现的文件系统操作代码”的封装
- `lib/fuse_lowlevel.c` –> 实现比较底层的函数封装，供`fuse.c`等使用
- `lib/fuse_loop.c` —> fuse lib循环监视”fuse driver”的通信缓存
- `lib/fuse_loop_mt.c` —> 同`fuse_loop.c`
- `lib/fuse_session.c` —> fuse会话管理
## Fuse如何工作？

1. fuse库
   1. 在用户态程序调用fuse_main() （lib/helper.c）时，先调用fuse_setup_common()该函数先解析用户态程序传递过来的参数，然后调用fuse_mount_common()（该函数是fuse_kern_mount()函数的封装，lib/mount.c）。fuse_main()是一个宏定义（include/fuse.h）,如下：
```
#define fuse_main(argc, argv, op, user_data)  \
fuse_main_real(argc, argv, op, sizeof(*(op)), user_data)
```

   2. fuse_kern_mount()函数中调用fuse_mount_fusermount()使用socketpair()创建一个UNIX域套接字，然后使用创建子进程执行fusermount程序，将FUSE_COMMFD_ENV环境变量中套接字的一端传递给它。
   3. fusermount（util/fusermount.c）确保fuse 模块已经被加载，然后打开/dev/fuse并通过一个UNIX套接字发送文件处理句柄。父进程等待子进程执行完毕回收，然后返回fuse_mount_fusermount()函数。
   4. fuse_kern_mount()通过/dev/fuse返回文件句柄给fuse_kern_chan_new()负责处理内核数据，然后返回到fuse_mount_common()函数。
   5. fuse_setup_common()函数调用fuse_new_common（lib/fuse.c）,fuse_new_common()函数分配fuse数据结构，存储并维护一个文件系统数据镜像缓存cached，返回到fuse_main()。
   6. 最后，fuse_main()调用fuse_loop（lib/fuse.c）或者fuse_loop_mt()（lib/fuse_mt.c），这两个函数都可以从设备/dev/fuse读取文件系统调用，调用fuse_main()之前调用存储在fuse_operations结构体中的用户态函数。这些调用的结果回写到/dev/fuse设备（这个设备可以转发给系统调用）。


2. 内核模块

内核模块由2个部分组成：
1. proc文件系统组件（在kernel/dev.c中）；
2. 文件系统调用（kernel/file.c、kernel/inode.c、kernel/dir.c）。

文件系统调用要么调用request_send()，要么调用request_send_noreply()或者request_send_nonblock()。大部分都是调用request_send()函数，它添加请求到“list of request”结构体（fc->pending）,然后等待一个响应。request_send_noreply()和request_send_nonblock()与request_send(）函数相似，除了是非阻塞的和不响应一个回复。 

kernel/dev.c中的proc文件系统组件响应文件IO请求，fuse_dev_read()处理文件读，并从请求列表结构体（list of requests）返回命令到调用程序。fuse_dev_write()处理文件写， 完成数据写并放入req->out结构体（它能返回系统调用通过请求列表结构体和request_send()函数）。

**用户进程和操作系统进行交互（read文件为例）：**

该fuse文件系统挂载在现有ext4文件系统之上. 
1. 一个用户进程发出read文件请求； 
2. 该请求被转换为一个内核系统调用，内核VFS层调用fuse文件系统内核模块； 
3. fuse 内核模块通过/dev/fuse，将read请求传递到fuse 用户态进程； 
4. fuse daemon根据用户实现的read接口，产生新的系统调用，最终调用ext4文件系统的read操作函数，从存储介质中提取读操作要求的数据（page cache中有，直接从其中获取，否则读磁盘）； 
5. 内核将数据返回给fuse文件系统； 
6. 用户级文件系统再次调用内核操作，把数据返回给用户进程； 
7. 内核将数据传给用户进程完成操作。

**库函数fuse_main()具体处理流程：**

1. 打开设备文件/dev/fuse； 
2. 挂载FUSE文件系统； 
3. 产生FUSE文件系统指针； 
4. 初始化FUSE文件系统的操作函数集： 
5. 初始化信号处理函数集； 
6. 进入等待循环：
 从设备文件/dev/fuse中读取来自内核模块的请求； 
 运行相应的操作函数，并获取返回结果； 
 将返回给内核的应答结果写入设备文件/dev/fuse中。