import sys
from PIL import Image

file_name = sys.argv[1]
# 打开图像文件
img = Image.open(file_name)

# 将图像转换为RGB模式
img = img.convert("RGB")

# 获取图像的数据
data = img.getdata()

# 创建一个新的列表用于存储交换后的数据
new_data = [(b, 255-r, g) for (r, g, b) in data]

# 更新图像的数据
img.putdata(new_data)

# 保存修改后的图像
print(file_name.replace('_s', '_mr'))
img.save(file_name.replace('_s.png', '_mr.png'))

# The blue channel contains metallic values,
# and the green channel contains the roughness values.
