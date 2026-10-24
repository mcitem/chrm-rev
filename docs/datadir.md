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

AppData\com.mcitem.chrm-rev 存放 Chrm Rev 的数据库文件 `chrm-rev.sqlite.db`。系统配置和业务数据统一保存在此数据库中

## 系统配置

配置保存在数据库的 `app_config` 表中，可通过内置的配置管理界面修改。

数据库中尚无配置时，通过内置配置管理界面设置并保存。

[配置参考](./configloader.md)

## chrm-rev.sqlite.db

`chrm-rev.sqlite.db`是Chrm Rev的数据库文件（sqlite），存放系统配置、商品、学生、未导出的记录

软件启动时，此文件会被占用，无法被删除。

使用 [datagrip](./datagrip.md) 可以直接编辑数据库

备份和恢复仅处理数据库文件，系统配置随数据库一并保存和恢复。
