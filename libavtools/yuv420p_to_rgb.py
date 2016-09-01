#coding: utf8

import sys, time

try:
    import Image
except:
    from PIL import Image

# YUV420P To RGB
 
if len(sys.argv) != 4:
    print u"使用:"
    print u"\t$python yuv2rgb.py <.yuv file yuv420p> <width> <height>"
    sys.exit(1)


"""
    生成一个 YUV420P 的 RawVideo 文件:
        ffmpeg -i source.mp4 source.yuv
    
    播放 YUV420P 原始视频(需要你指定宽高，所以当你转换成 YUV420P 时，请记住 视频文件的宽高):
        ffplay -f rawvideo -video_size <width>x<height> test.yuv
        ffplay -f rawvideo -video_size 1920x1080 test.yuv

    或者使用 Mplayer 来播放(FPS值随便设定，默认24FPS/25FPS):
        mplayer -demuxer rawvideo -rawvideo w=1920:h=1080:format=i420 -loop 0 -fps 24 test.yuv

    Note:
        由于 YUV 文件没有使用压缩，所以体积非常大，
        10秒 的 720P 视频文件大概占用 800MB 左右存储空间，
        所以请尽可能使用 5秒 - 10秒 左右的视频文件测试。
"""

def ycbcr_to_rgb(y, cb, cr):
    r = y + 1.402   * (cr-128)
    g = y - 0.34414 * (cb-128) -  0.71414 * (cr-128)
    b = y + 1.772   * (cb-128)
    if b < 255: b += 1
    return int(r), int(g), int(b)

def rgb_to_ycbcr(r, g, b):
    assert(255>=r)
    assert(255>=g)
    assert(255>=b)
    y  = 0.299*r + 0.587*g    + 0.114*b
    cb = 128     - 0.168736*r - 0.331364*g + 0.5*b
    cr = 128     + 0.5*r      - 0.418688*g - 0.081312*b
    return int(y), int(cb), int(cr)

def write(fname, width, height):
    img = Image.open( fname )
    if img.size != (width, height): img.resize((width, height))
    assert(img.mode == 'RGB')
    
    # pixels = list(img.getdata())
    size  = int(width*height*1.5)
    frame = range(size)

    for i in range(height):
        for j in range(width):
            # Pixel
            # (r,g,b)   = pixels[i*width + j]
            (r,g,b) = img.getpixel((j, i))
            y, cb, cr = rgb_to_ycbcr(r, g, b)

            idx = i*width/4
            if i%2 > 0: idx = (i*width - width)/4

            yindex = i*width + j
            uindex = idx + width*height
            vindex = uindex + int(width*height*0.25)

            frame[yindex] = y
            frame[uindex + j/2] = cb
            frame[vindex + j/2] = cr
    return frame

def read(fname, width, height, frame_num):
    f     = open(fname, "rb")
    size  = int(width*height*1.5)
    f.seek(size*frame_num)
    

    frame = map(lambda c: ord(c), f.read(size))
    print "size: %d  frame size: %d" %(size, len(frame))
    
    img = Image.new("RGB", (width, height))
    for i in range(height):
        for j in range(width):

            # idx = (i/2) * (width/2)
            idx = i*width/4
            if i%2 > 0: idx = (i*width - width)/4
            
            y  = frame[i*width + j]

            idx1 = width*height + idx
            cb   = frame[idx1 + j/2 ]

            idx2 = idx1 + int(width*height*0.25)
            cr   = frame[idx2 + j/2 ]

            (r,g,b) = ycbcr_to_rgb(y, cb, cr)

            img.putpixel((j, i),(r,g,b))
    f.close()
    return img


def yuv2rgb(yuv_file_name, width, height):
    # 读取 24 帧
    for n in range(1):
        img = read(yuv_file_name, width, height, n)
        img.show()
        img.save("frame-%d.png" % n )
        print "[YUV TO RGB] Frame: %d Done." % n
    print u":: YUV To RGB Image Done."

def rgb2yuv(yuv_file_name, image_name_prefix, image_count, width, height):
    open(yuv_file_name, "w").write("")
    for n in range(image_count):
        # write YUV 420P
        image_name = "frame-%d.png" % n
        data = map(lambda c: chr(c), write(image_name, width, height) )
        open(yuv_file_name, "a").write( "".join(data) )
        print "[RGB TO YUV] Image: %s Done." % image_name

    print u":: RGB To YUV 420P Done."

if __name__ == '__main__':
    fname  = sys.argv[1]
    width  = int(sys.argv[2])
    height = int(sys.argv[3])

    print u"Width=%d, Height=%d" %(width, height)
    
    # btime = time.time()
    yuv2rgb(fname, width, height)
    # 写进 24 帧
    # rgb2yuv(fname, "frame", 1, width, height)
    # etime = time.time()
    # print "Time: ", (etime-btime), "s"
    
