#!coding: utf8

"""
    Read Raw RGB24 Seq.

"""
try:
    import Image
except:
    from PIL import Image

f = open("test.rgb", "rb").read()
rgb_bytes = map(lambda c: ord(c), f)

im = Image.new("RGB", (1440, 900))
width, height = im.size
for h in range(height):
    for w in range(width):
        idx = w*h*3 + w*3
        r   = rgb_bytes[idx]
        g   = rgb_bytes[idx+1]
        b   = rgb_bytes[idx+2]
        # print r,g,b
        im.putpixel((w,h), (r,g,b) )
im.show()