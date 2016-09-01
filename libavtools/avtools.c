
#include <stdio.h>
#include <stdint.h>
#include <time.h>

#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libavformat/avformat.h>
#include <libavutil/imgutils.h>
#include <libavutil/parseutils.h>
#include <libavutil/samplefmt.h>
#include <libavutil/opt.h>

/*
Compile(OS X):
    cc -I /usr/local/Cellar/ffmpeg/3.0.1/include -Wall -g -c \
            -o avtools.o avtools.c
    cc  avtools.o -L /usr/local/Cellar/ffmpeg/3.0.1/lib \
            -lavdevice -lavformat -lavfilter -lavcodec \
            -lswresample -lswscale -lavutil  \
            -o avtools
*/

void hello() {
    printf("[avtools] Hello, World!\n");
}

void fill_yuv_image(uint8_t *data[4], int linesize[4],
                    int width, int height, int frame_index){
    int x, y;
    /* Y */
    for (y = 0; y < height; y++)
        for (x = 0; x < width; x++)
            data[0][y * linesize[0] + x] = x + y + frame_index * 3;
    /* Cb and Cr */
    for (y = 0; y < height / 2; y++) {
        for (x = 0; x < width / 2; x++) {
            data[1][y * linesize[1] + x] = 128 + y + frame_index * 2;
            data[2][y * linesize[2] + x] = 64 + x + frame_index * 5;
        }
    }
}

int rgb_to_ycbcr(uint8_t r, uint8_t g,  uint8_t b,
                 uint8_t ycbcr[3]){
    double rf, gf, bf;
    double yf, cbf, crf;

    rf = (double) r;
    gf = (double) g;
    bf = (double) b;

    yf  = 0.299*rf + 0.587*gf    + 0.114*bf;
    cbf = 128.0    - 0.168736*rf - 0.331364*gf + 0.5*bf;
    crf = 128.0    + 0.5*rf      - 0.418688*gf - 0.081312*bf;

    ycbcr[0] = (uint8_t) yf;
    ycbcr[1] = (uint8_t) cbf;
    ycbcr[2] = (uint8_t) crf;

    return 1;
}

int rgb24_to_yuv420p_with_slowspeed(uint8_t rgb24_data[], int src_w, int src_h, 
                                    uint8_t dst_data[],   int dst_w, int dst_h){

    if ( src_w != dst_w || src_h != dst_h ) {
        return 0;
    }

    int         h, w, idx, bidx, yidx, uidx, vidx;
    uint8_t     r=0, g=0 ,b=0, ycbcr[3];

    int size = dst_w*dst_h + dst_w*dst_h/2;

    for (h=0; h<dst_h; h++) {
        for (w=0; w<dst_w; w++){
            idx = w*h*3 + w*3;

            r = rgb24_data[idx];
            g = rgb24_data[idx+1];
            b = rgb24_data[idx+2];

            rgb_to_ycbcr(r,g,b, ycbcr);

            bidx = h*dst_w/4;

            if (h%2 > 0) {
                bidx = (h*dst_w - dst_w)/4;
            }

            yidx = h*dst_w + w;
            uidx = bidx + dst_w*dst_h;
            vidx = uidx + dst_w*dst_h/4;

            dst_data[yidx]       = ycbcr[0];
            dst_data[uidx + w/2] = ycbcr[1];
            dst_data[vidx + w/2] = ycbcr[2];
        }
    }

    return size;
}

int rgb24_to_yuv420p (uint8_t rgb24_data[], int src_w, int src_h, 
                      uint8_t *dst_data[4], int dst_w, int dst_h){

    enum AVPixelFormat src_pixfmt=AV_PIX_FMT_RGB24, dst_pixfmt=AV_PIX_FMT_YUV420P;

    // int src_bpp = av_get_bits_per_pixel(av_pix_fmt_desc_get(src_pixfmt));
    // int dst_bpp = av_get_bits_per_pixel(av_pix_fmt_desc_get(dst_pixfmt));
    
    uint8_t             *src_data[4];
    int                 src_linesize[4], dst_linesize[4];
    struct SwsContext   *img_convert_ctx;

    int ret=0;
    
    ret= av_image_alloc(src_data, src_linesize,src_w, src_h, src_pixfmt, 1);
    if ( ret < 0 ) {
        return -1;
    }
    ret = av_image_alloc(dst_data, dst_linesize,dst_w, dst_h, dst_pixfmt, 1);
    if ( ret < 0 ) {
        return -1;
    }

    img_convert_ctx = sws_getContext(src_w, src_h, src_pixfmt, 
                                     dst_w, dst_h, dst_pixfmt, 
                                     SWS_BICUBIC, NULL, NULL, NULL);

    // sws_scale(img_convert_ctx, (const uint8_t * const*)src_data, src_linesize,
    //           0, src_h, dst_data, dst_linesize);
    // memcpy(src_data[0], rgb24_data, src_w*src_h*3);

    src_data[0] = rgb24_data;

    sws_scale(img_convert_ctx, (const uint8_t *const *)src_data, src_linesize,
              0, src_h, dst_data, dst_linesize);

    sws_freeContext(img_convert_ctx);

    // av_freep(&src_data[0]);
    return 0;
}

// int rgb24_to_h264(uint8_t rgb24_data[], AVCodecContext *c, 
//                  int width, int height, AVPacket pkt ){
int rgb24_to_h264(uint8_t rgb24_data[], int width, int height, AVPacket pkt, int pts ){
    av_register_all();

    time_t          btime, etime;
    enum AVCodecID  codec_id = AV_CODEC_ID_H264;
    AVCodec         *codec;
    AVCodecContext  *c= NULL;
    int             i=0, ret, got_output;
    AVFrame         *frame;
    // AVPacket        pkt;
    // MPEG EOF Flag
    uint8_t         endcode[] = { 0, 0, 1, 0xb7 };

    btime = time(NULL);

    codec = avcodec_find_encoder(codec_id);
    if (!codec) {
        fprintf(stderr, "Codec not found\n");
        return 0;
    }

    // printf("Codec: %s\n", codec->name);

    c = avcodec_alloc_context3(codec);
    if (!c) {
        fprintf(stderr, "Could not allocate video codec context\n");
        return 0;
    }

    c->bit_rate     = 400000;
    c->width        = width;
    c->height       = height;
    c->time_base    = (AVRational){1,25};
    c->gop_size     = 10;
    c->max_b_frames = 1;
    c->pix_fmt      = AV_PIX_FMT_YUV420P;

    if (codec_id == AV_CODEC_ID_H264){
        av_opt_set(c->priv_data, "preset", "slow", 0);
    }
    if (avcodec_open2(c, codec, NULL) < 0) {
        fprintf(stderr, "Could not open codec\n");
        return 0;
    }
    // Init AVFrame
    frame = av_frame_alloc();
    if (!frame) {
        fprintf(stderr, "Could not allocate video frame\n");
        return 0;
    }

    frame->format = c->pix_fmt;
    frame->width  = c->width;
    frame->height = c->height;

    ret = av_image_alloc(frame->data, frame->linesize, c->width, 
                         c->height, c->pix_fmt, 32);
    if (ret < 0) {
        fprintf(stderr, "Could not allocate raw picture buffer\n");
        return 0;
    }
    
    // RGB24 Data
    uint8_t             *input_data[4];
    int                 input_linesize[4];
    struct  SwsContext  *sws_ctx;
    
    // Assign RGB24 PIXELS SEQ DATA.
    input_data[0] = rgb24_data;

    if ((ret = av_image_alloc(input_data, input_linesize, frame->width,
                              frame->height, AV_PIX_FMT_RGB24, 1)) < 0) {
        fprintf(stderr, "Could not allocate destination image\n");
        return 0;
    }

    // Convert RGB24 Picture To YUV420P(I420P).
    sws_ctx = sws_getContext(frame->width, frame->height, AV_PIX_FMT_RGB24,
                             frame->width, frame->height, AV_PIX_FMT_YUV420P,
                             SWS_BICUBIC, NULL, NULL, NULL);

    sws_scale(sws_ctx, (const uint8_t * const *)input_data, input_linesize, 
              0, frame->height, frame->data, frame->linesize);
    
    // Encode YUV420P(I420) Picture To Raw Video (Without Media Format).
    av_init_packet(&pkt);
    pkt.data = NULL;       // packet data will be allocated by the encoder
    pkt.size = 0;
    fflush(stdout);

    frame->pts = pts;  // picture seq index.

    ret = avcodec_encode_video2(c, &pkt, frame, &got_output);
    if (ret < 0) {
        fprintf(stderr, "Error encoding frame\n");
        return 0;
    }

    // Debug
    FILE *f;
    f = fopen("test.h264", "wb");
    if (!f) {
        fprintf(stderr, "Could not open %s\n", "test.h264");
        return 0;
    }

    if (got_output) {
        printf("Write frame %3d (size=%d)\n", pts, pkt.size);

        // printf("PKT Size: %lu\n", sizeof(pkt.size) );
        fwrite(pkt.data, 1, pkt.size, f);
        // av_packet_unref(&pkt);
    }

    // get the delayed frames
    for (got_output=1; got_output; i++) {
        fflush(stdout);
        ret = avcodec_encode_video2(c, &pkt, NULL, &got_output);
        if (ret < 0) {
            fprintf(stderr, "Error encoding frame\n");
            return 0;
        }
        if (got_output) {
            printf("Write frame %3d (size=%5d)\n", pts, pkt.size);

            fwrite(pkt.data, 1, pkt.size, f);
            // av_packet_unref(&pkt);
        }
    }

    // add sequence end code to have a real mpeg file
    fwrite(endcode, 1, sizeof(endcode), f);

    fclose(f);

    avcodec_close(c);
    av_free(c);
    av_freep(&frame->data[0]);
    av_frame_free(&frame);

    printf("\n");
    etime = time(NULL);
    printf("time: %.0f s\n", difftime(etime, btime)) ;
    return pkt.size;
}

int rgb24_to_mpegts(){
    // NOTE:
    //      `AV_CODEC_ID_MPEG2TS` 是一个虚假 `CODEC`，
    //      如果你需要创建一个 `MPEG-TS` 的媒体文件，
    //      需要先编码 `视频流`，
    //      一般是 `AV_CODEC_ID_MPEG1VIDEO/AV_CODEC_ID_MPEG2VIDEO/AV_CODEC_ID_H264`，
    //      然后将该 `视频流` 添加到 `AVFormat`（即媒体容器）里面，视情况可以加入 `音频流`。
    return 0;
}

int image_to_avframe(const char* img_file_name, AVFrame* frame) {
    int     width=1440,height=900;

    AVFormatContext *fmt_ctx = avformat_alloc_context();

    if ( avformat_open_input(&fmt_ctx, img_file_name, NULL, NULL) != 0 ) {
        printf("Can't open image file '%s'\n", img_file_name);
        return 1;
    }

    AVCodecContext *codec_ctx; //  = avcodec_alloc_context3()

    codec_ctx          = fmt_ctx->streams[0]->codec;
    codec_ctx->width   = width;
    codec_ctx->height  = height;
    codec_ctx->pix_fmt = AV_PIX_FMT_RGB24;

    // Find the decoder for the video stream
    AVCodec *codec = avcodec_find_decoder(codec_ctx->codec_id);
    if (!codec) {
        printf("Codec not found\n");
        return 2;
    }

    // Open codec
    if(avcodec_open2(codec_ctx, codec, NULL)<0) {
        printf("Could not open codec\n");
        return 3;
    }

    if (!frame) {
        printf("Can't allocate memory for AVFrame\n");
        return 4;
    }

    int frameFinished;
    int numBytes = width*height*3;

    // Determine required buffer size and allocate buffer
    // numBytes = avpicture_get_size(AV_PIX_FMT_RGB24, codec_ctx->width, codec_ctx->height);
    uint8_t *buffer = (uint8_t *) av_malloc(numBytes * sizeof(uint8_t));

    av_image_fill_arrays(frame->data, frame->linesize,
                         buffer, AV_PIX_FMT_RGB24, 
                         codec_ctx->width, codec_ctx->height, 1);

    // Read frame
    AVPacket packet;
    // int framesNumber = 0;
    while (av_read_frame(fmt_ctx, &packet) >= 0) {
        if(packet.stream_index != 0)
            continue;
        int ret = avcodec_decode_video2(codec_ctx, frame, &frameFinished, &packet);
        if (ret > 0) {
            printf("Frame is decoded, size %d\n", ret);
            frame->quality = 4;
        } else {
            printf("Error [%d] while decoding frame: %s\n", ret, strerror(AVERROR(ret)));
        }
    }
    return 0;
}

int test_rgb24_to_yuv420p_with_slowspeed (){
    av_register_all();

    int src_w=1440,  src_h=900;
    int dst_w=src_w, dst_h=src_h; // 不支持更改分辨率

    AVFrame *frame = av_frame_alloc();
    FILE *dst_file = fopen("me.yuv", "wb");

    image_to_avframe("tree.png", frame);

    // uint8_t *temp_buffer=(uint8_t *)malloc(src_w*src_h*3);
    // memcpy(temp_buffer, frame->data[0], src_w*src_h*3);

    int dst_size=dst_w*dst_h + dst_w*dst_h/2;
    uint8_t dst_data[dst_size];

    int size = rgb24_to_yuv420p_with_slowspeed (frame->data[0], src_w, src_h, 
                                                dst_data, dst_w, dst_h);

    if ( dst_size != size ) {
        printf("Error: dst_size(%d) != size(%d) \n", dst_size, size);
    } else {
        fwrite(dst_data, 1, dst_size,  dst_file);
    }

    fclose(dst_file);

    // free(temp_buffer);
    av_frame_free(&frame);

    printf("[Done] : ffplay -f rawvideo -pixel_format yuv420p -video_size %dx%d me.yuv\n", 
            dst_w, dst_h);

    return 1;
}
int test_rgb24_to_yuv420p(){
    av_register_all();

    FILE *dst_file = fopen("me.yuv", "wb");
    AVFrame *frame = av_frame_alloc();

    image_to_avframe("me.png", frame);

    int src_w=1440, src_h=900;
    int dst_w=1440, dst_h=900;

    uint8_t *temp_buffer=(uint8_t *)malloc(src_w*src_h*3);
    memcpy(temp_buffer, frame->data[0], src_w*src_h*3);

    uint8_t *dst_data[4];

    rgb24_to_yuv420p(temp_buffer, src_w, src_h, 
                     dst_data, dst_w, dst_h);

    fwrite(dst_data[0],1,dst_w*dst_h,  dst_file); // Y
    fwrite(dst_data[1],1,dst_w*dst_h/4,dst_file); // U
    fwrite(dst_data[2],1,dst_w*dst_h/4,dst_file); // V

    fclose(dst_file);

    free(temp_buffer);
    av_frame_free(&frame);
    av_freep(&dst_data[0]);

    printf("[Done] : ffplay -f rawvideo -pixel_format yuv420p -video_size %dx%d me.yuv\n", 
            dst_w, dst_h);
    return 0;
}

int test_image_to_rgb24(){
    // Image File To RGB24 RawVideo
    av_register_all();

    int     width=1440,height=900;
    FILE    *dst_file;

    AVFrame *frame1 = av_frame_alloc();
    AVFrame *frame2 = av_frame_alloc();

    image_to_avframe("me.png", frame1);
    image_to_avframe("tree.png", frame2);

    dst_file = fopen("me.rgb", "wb");

    fwrite(frame1->data[0], 1, width*height*3, dst_file);
    fwrite(frame2->data[0], 1, width*height*3, dst_file);

    printf("[Done] : ffplay -f rawvideo -pixel_format rgb24 -video_size %dx%d me.rgb\n", 
            width, height);
    return 0;
}

int test_rgb24_to_h264(){
    av_register_all();

    int width=1440,height=900;

    AVFrame *frame1 = av_frame_alloc();

    image_to_avframe("tree.png", frame1);


    uint8_t *temp_buffer=(uint8_t *)malloc(width*height*3);
    memcpy(temp_buffer, frame1->data[0], width*height*3);

    AVPacket pkt;
    int i;

    for (i=0; i<50; i++){
        // printf("PKT Size: %lu\n", sizeof(pkt.size) );
        rgb24_to_h264(temp_buffer, width, height, pkt, i);
        // printf("PKT Size: %lu\n", sizeof(pkt.size) );
        // av_packet_unref(&pkt);
    }
    return 1;
}
void video_encode_without_format(const char *filename, int codec_id) {
    av_register_all();
        
    AVCodec         *codec;
    AVCodecContext  *c=NULL;
    int i, ret, got_output;
    FILE        *f;
    AVFrame     *frame;
    AVPacket    pkt;

    uint8_t     endcode[] = { 0, 0, 1, 0xb7 };

    printf("Encode video file %s\n", filename);

    codec = avcodec_find_encoder(codec_id);
    if (!codec) {
        fprintf(stderr, "Codec not found\n");
        exit(1);
    }

    c = avcodec_alloc_context3(codec);
    if (!c) {
        fprintf(stderr, "Could not allocate video codec context\n");
        exit(1);
    }

    // H264 Bitrates ( `megapixels` 代表百万像素 ):
    //      http://www.lighterra.com/papers/videoencodingh264/
    //      240p (424x240, 0.10 megapixels)
    //      360p (640x360, 0.23 megapixels)
    //      432p (768x432, 0.33 megapixels)
    //      480p (848x480, 0.41 megapixels, "SD" or "NTSC widescreen")
    //      576p (1024x576, 0.59 megapixels, "PAL widescreen")
    //      720p (1280x720, 0.92 megapixels, "HD")
    //      1080p (1920x1080, 2.07 megapixels, "Full HD")
    c->bit_rate  = 2000000;
    /* resolution must be a multiple of two */
    c->width     = 1440;
    c->height    = 900;
    c->time_base = (AVRational){1,25}; // FPS
    /* emit one intra frame every ten frames
     * check frame pict_type before passing frame
     * to encoder, if frame->pict_type is AV_PICTURE_TYPE_I
     * then gop_size is ignored and the output of encoder
     * will always be I frame irrespective to gop_size
     */
    c->gop_size     = 60;
    c->max_b_frames = 10;
    c->pix_fmt      = AV_PIX_FMT_YUV420P;

    if (codec_id == AV_CODEC_ID_H264){
        // { "ultrafast", "superfast", "veryfast", "faster", 
        //   "fast", "medium", "slow", "slower", "veryslow", 
        //   "placebo", 0 };
        av_opt_set(c->priv_data, "preset", "veryslow", 0);
    }

    if (avcodec_open2(c, codec, NULL) < 0) {
        fprintf(stderr, "Could not open codec\n");
        exit(1);
    }

    f = fopen(filename, "wb");
    if (!f) {
        fprintf(stderr, "Could not open %s\n", filename);
        exit(1);
    }

    frame = av_frame_alloc();
    if (!frame) {
        fprintf(stderr, "Could not allocate video frame\n");
        exit(1);
    }
    /* the image can be allocated by any means and av_image_alloc() is
     * just the most convenient way if av_malloc() is to be used */
    ret = av_image_alloc(frame->data, frame->linesize, 
                         c->width,    c->height,
                         c->pix_fmt,  32);
    if (ret < 0) {
        fprintf(stderr, "Could not allocate raw picture buffer\n");
        exit(1);
    }
    frame->format = c->pix_fmt; // AV_PIX_FMT_YUV420P
    frame->width  = c->width;
    frame->height = c->height;


    AVFrame *image_frame = av_frame_alloc();

    image_to_avframe("me.png", image_frame);

    int src_w=c->width, src_h=c->height;
    int dst_w=src_w, dst_h=src_h;

    uint8_t *temp_buffer=(uint8_t *)malloc(src_w*src_h*3);
    memcpy(temp_buffer, image_frame->data[0], src_w*src_h*3);

    rgb24_to_yuv420p(temp_buffer, src_w, src_h, 
                     frame->data, dst_w, dst_h);

    int ysize = dst_w*dst_h;
    int usize = dst_w*dst_h/4;
    int vsize = dst_w*dst_h/4;
    
    printf("Y size: %d\t U size:%d \t V size: %d\n",  ysize, usize, vsize );

    for (i = 0; i < 50; i++) {
        av_init_packet(&pkt);
        pkt.data = NULL;
        pkt.size = 0;

        fflush(stdout);
        
        frame->pts = i;

        ret = avcodec_encode_video2(c, &pkt, frame, &got_output);

        if (ret < 0) {
            fprintf(stderr, "Error encoding frame\n");
            exit(1);
        }

        if (got_output) {
            // printf("Write frame %3d (size=%5d)\n", i, pkt.size);
            fwrite(pkt.data, 1, pkt.size, f);
            av_packet_unref(&pkt);
        }
    }

    /* get the delayed frames */
    for (got_output = 1; got_output; i++) {
        fflush(stdout);

        ret = avcodec_encode_video2(c, &pkt, NULL, &got_output);
        if (ret < 0) {
            fprintf(stderr, "Error encoding frame\n");
            exit(1);
        }

        if (got_output) {
            // printf("Write frame(delayed) %3d (size=%5d)\n", i, pkt.size);
            fwrite(pkt.data, 1, pkt.size, f);
            av_packet_unref(&pkt);
        }
    }

    /* add sequence end code to have a real MPEG file */
    fwrite(endcode, 1, sizeof(endcode), f);
    fclose(f);

    avcodec_close(c);
    av_free(c);
    av_freep(&frame->data[0]);
    av_frame_free(&frame);
    printf("[Done] ffplay test.h264\n");
}

int entry(int argc, char const *argv[]){
    av_log_set_level(AV_LOG_QUIET);

    // test_image_to_rgb24();
    // test_rgb24_to_yuv420p();
    // test_rgb24_to_yuv420p_with_slowspeed();
    video_encode_without_format("test.h264", AV_CODEC_ID_H264);
    // test_rgb24_to_h264();
    return 1;
}

