## 数据目录

Chrm Rev 的数据目录位于

```
C:\Users\<用户名>\AppData\Roaming\com.mcitem.chrm-rev
```

其中用户名为windows系统登录所用的账号名称，常见为`Administrator`

![datadir](./assets/datadir.png)

你可以通过左上角`菜单-管理-数据目录`快捷打开此目录

## AppData

AppData 是隐藏的文件夹，需要在文件资源管理器中启用显示隐藏的项目

AppData\com.mcitem.chrm-rev 存放 Chrm Rev 的配置文件`chrm-rev.config.json`和数据库文件`chrm-rev.sqlite.db`

## chrm-rev.config.json

`chrm-rev.config.json`是Chrm Rev的配置文件（json格式），此文件只会在启动时被读取，启动后不会监听文件变化，如需编辑请使用内置的编辑工具或先关闭软件

[配置参考](./configloader.md)

## chrm-rev.sqlite.db

`chrm-rev.sqlite.db`是Chrm Rev的数据库文件（sqlite），存放商品、学生、未导出的记录

软件启动时，此文件会被占用，无法被删除。

使用 [datagrip](./datagrip.md) 可以直接编辑数据库
