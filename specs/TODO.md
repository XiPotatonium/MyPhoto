# BUGFIX

- [x] 点击文件夹时ui会卡死。需要await在后台读取
- [x] 竖屏拍摄图像在小图展示时没有旋转
- [ ] raf格式竖拍没有旋转
- [x] 读取图片速度慢会导致ui卡死。没有异步
- [x] 图片浏览器一列只会展示一张图。css太窄了
- [x] exif中的镜头信息解析有问题。display_value的问题
- [x] write_rating并没有实现
- [x] 文件夹如果没有子文件夹，会不显示。前端没有显示根目录。
- [x] 删除照片没有触发

## 前端图片信息大更新

- [x] 前端的图片信息需要包括路径，thumbnail，文件创建时间以及一些exif信息（可以用于检索排序）。这些信息除了thumbnail之外，都需要在list_images时就需要得到。
- [x] 图片打分时打分对于exif的修改需要同步到前端图片信息中。打分空间显示图片打分不要每次读取exif，而是直接显示前端存储的exif信息中的分数。
- [x] 排序在前端完成而不是每次调用rust
- [x] 删除图片后立刻重新渲染，在前端渲染而不是重新调用rust的list_images
- [x] 按时间排序优先读exif中的时间，如果没有则使用文件创建时间，不要使用文件修改时间

# 新内容

- [x] 新增一个按照文件名进行排序的规则。
- [x] 支持富士.raf格式
- [x] rust后端实现一个LRU cache，方便从缓存中快速读取图片，避免每次点击图片的时候都重新从硬盘读取，读取缩略图也要走这个cache，注意这个cache需要判断本地文件的修改日期防止缓存失效
- [x] 支持gps位置写入
- [ ] raw文件的exif读取是有问题的。


# ui美化

使用shadcn控件库，支持深色和浅色模式

- [x] 深色模式部分控件完全看不见，深色模式需要进一步优化：背景不应该全黑，而是灰色，字体需要变成白色，选中高亮应该变成灰色或者深蓝色。
- [x] 切换jpg和raw的图标有点难看，需要换掉
- [x] 切换排序的toggle是透明的，影响观看。加入悬浮特效。
- [x] gps添加面板(src/components/common/GPSDialog.vue)展示有问题，背景是透明的
- [x] 控件的padding需要优化，很多控件过于紧凑
- [x] 用Field https://www.shadcn-vue.com/docs/components/field 优化gps对话框
- [x] 选择文件夹按钮以及排序select过于紧凑的问题

# 图像加载速度优化

暂时不考虑解决，似乎是debug模式下就是很慢

## rust读取缓慢

看看为什么这么慢

https://www.reddit.com/r/rust/comments/k1wjix/why_opening_of_images_is_so_slow/

## 前端展示缓慢

即使缓存命中，前端似乎也需要几秒钟来渲染，导致看起来非常慢
