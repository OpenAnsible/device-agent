
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

static void fill_yuv_image(uint8_t *data[4], int linesize[4],
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

int rgb_to_ycbcr(uint8_t r, uint8_t g,  uint8_t b, // input
                 uint8_t ycbcr[3]){ // output
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

int rgb24_to_i420 (char rgb24data[], int width, int height, 
                   uint8_t output_bytes[]){
    // 软解
    int         h, w;
    // int size = width*height + width*height/2;
    uint8_t     r=0 ,g=0 ,b=0;
    // uint8_t     y=0,cb=0,cr=0;
    uint8_t     ycbcr[3];
    int         idx, bidx, yidx, uidx, vidx;
    for (h=0; h<height; h++) {
        for (w=0; w<width; w++){
            idx = w*h*3 + w*3;
            // printf("IDX: %d\n", idx);
            r = (uint8_t) rgb24data[idx];
            g = (uint8_t) rgb24data[idx+1];
            b = (uint8_t) rgb24data[idx+2];
            rgb_to_ycbcr(r,g,b, ycbcr);
            // printf("(%d,%d,%d) -> (%d, %d, %d)\n", r,g,b, ycbcr[0],ycbcr[1],ycbcr[2] );
            bidx = h*width/4;
            if (h%2 > 0) {
                bidx = (h*width - width)/4;
            }

            yidx = h*width + w;
            uidx = bidx + width*height;
            vidx = uidx + width*height/4;

            output_bytes[yidx]       = ycbcr[0];
            output_bytes[uidx + w/2] = ycbcr[1];
            output_bytes[vidx + w/2] = ycbcr[2];
        }
    }
    return 1;
}

int rgb24_to_yuv420p (uint8_t rgb24_data[], int src_w, int src_h, 
                      uint8_t *dst_data[4],  int dst_w, int dst_h){

    enum AVPixelFormat src_pixfmt=AV_PIX_FMT_RGB24, dst_pixfmt=AV_PIX_FMT_YUV420P;

    // int src_bpp = av_get_bits_per_pixel(av_pix_fmt_desc_get(src_pixfmt));
    // int dst_bpp = av_get_bits_per_pixel(av_pix_fmt_desc_get(dst_pixfmt));
    
    uint8_t             *src_data[4];
    int                 src_linesize[4], dst_linesize[4];
    struct SwsContext   *img_convert_ctx;

    int ret=0;
    
    ret= av_image_alloc(src_data, src_linesize,src_w, src_h, src_pixfmt, 1);
    if ( ret < 0 ) {
        printf("Error.\n");
        return -1;
    }
    ret = av_image_alloc(dst_data, dst_linesize,dst_w, dst_h, dst_pixfmt, 1);
    if ( ret < 0 ) {
        printf("Error.\n");
        return -1;
    }

    img_convert_ctx = sws_getContext(src_w, src_h, src_pixfmt, 
                                     dst_w, dst_h, dst_pixfmt, 
                                     SWS_BICUBIC, NULL, NULL, NULL);

    // const uint8_t *const
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

/*

    Return: packet size.
        
*/
int rgb24_to_h264(uint8_t rgb24data[], int width, int height, AVPacket pkt ){
    av_register_all();

    time_t          btime, etime;
    enum AVCodecID  codec_id = AV_CODEC_ID_H264;
    AVCodec         *codec;
    AVCodecContext  *c= NULL;
    int             i=0, ret, got_output;
    AVFrame         *frame;
    // AVPacket        pkt;
    // MPEG EOF Flag
    // uint8_t         endcode[] = { 0, 0, 1, 0xb7 };

    btime = time(NULL);

    codec = avcodec_find_encoder(codec_id);
    if (!codec) {
        fprintf(stderr, "Codec not found\n");
        return 0;
    }
    printf("Codec: %s\n", codec->name);

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
    input_data[0] = rgb24data;

    if ((ret = av_image_alloc(input_data, input_linesize, frame->width,
                              frame->height, AV_PIX_FMT_RGB24, 1)) < 0) {
        fprintf(stderr, "Could not allocate destination image\n");
        return 0;
    }

    // Convert RGB24 Picture To YUV420P(I420P).
    sws_ctx = sws_getContext(frame->width, frame->height, AV_PIX_FMT_RGB24,
                             frame->width, frame->height, AV_PIX_FMT_YUV420P,
                             SWS_FAST_BILINEAR, 0, 0, 0);
    sws_scale(sws_ctx, (const uint8_t * const*)input_data, input_linesize, 
              0, frame->height, frame->data, frame->linesize);
    
    // Encode YUV420P(I420) Picture To Raw Video (Without Media Format).
    av_init_packet(&pkt);
    pkt.data = NULL;       // packet data will be allocated by the encoder
    pkt.size = 0;
    fflush(stdout);

    frame->pts = 1;  // picture seq index.

    ret = avcodec_encode_video2(c, &pkt, frame, &got_output);
    if (ret < 0) {
        fprintf(stderr, "Error encoding frame\n");
        return 0;
    }

    // Debug
    FILE            *f;
    f = fopen("test.h264", "wb");
    if (!f) {
        fprintf(stderr, "Could not open %s\n", "test.h264");
        return 0;
    }

    if (got_output) {
        printf("Write frame %3d (size=%5d)\n", 1, pkt.size);
        // Return Video Packet.
        // output_bytes[0] = pkt.size;
        // output_bytes = pkt.data;

        fwrite(pkt.data, 1, pkt.size, f);
        av_packet_unref(&pkt);
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
            printf("Write frame %3d (size=%5d)\n", i, pkt.size);
            // Return Video Packet.
            // *output_bytes[0] = pkt.size;
            // output_bytes = pkt.data;

            fwrite(pkt.data, 1, pkt.size, f);
            av_packet_unref(&pkt);
        }
    }

    // add sequence end code to have a real mpeg file
    // fwrite(endcode, 1, sizeof(endcode), f);

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
int Image2Frame(const char* imageFileName, AVFrame* frame) {
    int     width=1440,height=900;

    AVFormatContext *pFormatCtx = avformat_alloc_context();

    if (avformat_open_input(&pFormatCtx, imageFileName, NULL, NULL)!=0) {
        printf("Can't open image file '%s'\n", imageFileName);
        return 1;
    }

    AVCodecContext *pCodecCtx;

    pCodecCtx = pFormatCtx->streams[0]->codec;
    pCodecCtx->width = width;
    pCodecCtx->height = height;
    pCodecCtx->pix_fmt = AV_PIX_FMT_RGB24;

    // Find the decoder for the video stream
    AVCodec *pCodec = avcodec_find_decoder(pCodecCtx->codec_id);
    if (!pCodec) {
        printf("Codec not found\n");
        return 2;
    }

    // Open codec
    if(avcodec_open2(pCodecCtx, pCodec, NULL)<0) {
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
    // numBytes = avpicture_get_size(AV_PIX_FMT_RGB24, pCodecCtx->width, pCodecCtx->height);
    uint8_t *buffer = (uint8_t *) av_malloc(numBytes * sizeof(uint8_t));

    av_image_fill_arrays(frame->data, frame->linesize,
                         buffer, AV_PIX_FMT_RGB24, 
                         pCodecCtx->width, pCodecCtx->height, 1);

    // Read frame
    AVPacket packet;
    // int framesNumber = 0;
    while (av_read_frame(pFormatCtx, &packet) >= 0) {
        if(packet.stream_index != 0)
            continue;
        int ret = avcodec_decode_video2(pCodecCtx, frame, &frameFinished, &packet);
        if (ret > 0) {
            printf("Frame is decoded, size %d\n", ret);
            frame->quality = 1;
        } else {
            printf("Error [%d] while decoding frame: %s\n", ret, strerror(AVERROR(ret)));
        }
    }
    return 0;
}
int test_rgb24_to_i420(){
    av_register_all();
    int     width=1440,height=900;
    FILE    *output_file, *input_file;

    int     output_data_size=width*height + width*height/2;
    uint8_t output_data[output_data_size];

    char *buffer;
    long filelen;
    
    input_file = fopen("test.rgb", "rb");

    fseek(input_file, 0, SEEK_END);
    filelen = ftell(input_file);
    rewind(input_file);
    buffer = (char *)malloc((filelen+1)*sizeof(char));
    fread(buffer, filelen, 1, input_file);

    fclose(input_file);

    time_t  btime, etime;
    btime = time(NULL);
    for (int i=0; i<25; i++) {
        rgb24_to_i420(buffer, width, height, output_data);
    }
    etime = time(NULL);
    printf("time: %.0f s\n", difftime(etime, btime)) ;

    output_file = fopen("test.yuv", "wb");
    if (!output_file) {
        fprintf(stderr, "Could not open %s\n", "test.yuv");
        exit(1);
    }
    fwrite(output_data, 1, output_data_size, output_file);

    printf("[Done] : ffplay -f rawvideo -pix_fmt yuv420p -video_size 1440x900 test.yuv\n");

    return 0;
}
int test_rgb24_to_yuv420p(){
    av_register_all();

    uint8_t *dst_data[4];
    
    FILE *dst_file = fopen("me.yuv", "wb");

    AVFrame *frame = av_frame_alloc();
    Image2Frame("me.png", frame);

    int src_w=1440, src_h=900;
    int dst_w=1920, dst_h=1080;

    uint8_t *temp_buffer=(uint8_t *)malloc(src_w*src_h*3);
    memcpy(temp_buffer, frame->data[0], src_w*src_h*3);

    rgb24_to_yuv420p(temp_buffer, src_w, src_h, 
                     dst_data, dst_w, dst_h);

    fwrite(dst_data[0],1,dst_w*dst_h,  dst_file); // Y
    fwrite(dst_data[1],1,dst_w*dst_h/4,dst_file); // U
    fwrite(dst_data[2],1,dst_w*dst_h/4,dst_file); // V

    fclose(dst_file);

    free(temp_buffer);
    av_frame_free(&frame);
    av_freep(&dst_data[0]);

    printf("[Done] : ffplay -f rawvideo -pixel_format yuv420p -video_size %dx%d me.yuv\n", dst_w, dst_h);
    return 0;
}

int test_image_to_rgb24(){
    av_register_all();

    int     width=1440,height=900;
    FILE    *dst_file;

    AVFrame *frame1 = av_frame_alloc();
    // AVFrame *frame2 = av_frame_alloc();

    Image2Frame("me.png", frame1);
    // Image2Frame("tree.png", frame2);

    dst_file = fopen("me.rgb", "wb");

    fwrite(frame1->data[0], 1, width*height*3, dst_file);
    // fwrite(frame2->data[0], 1, width*height*3, dst_file);

    return 0;
}
int test_rgb24_to_h264(){
    av_register_all();

    // int     width=1440,height=900;

    AVFrame *frame1 = av_frame_alloc();

    Image2Frame("me.png", frame1);

    // rgb24_to_h264(frame1->data[0], width, height);
    return 0;
}

int entry(int argc, char const *argv[]){
    // test_image_to_rgb24();
    // test_rgb24_to_i420();
    test_rgb24_to_yuv420p();
    // test_rgb24_to_h264();
    return 1;
}

