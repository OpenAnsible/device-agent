#include <stdio.h>
#include <stdint.h>
#include "avtools.h"

#include <libavutil/opt.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libavutil/imgutils.h>
#include <libavutil/samplefmt.h>
#include <time.h>
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

int rgb24_to_yuv420p (uint8_t rgb24data[], int width, int height, uint8_t output_bytes[]){
    uint8_t *input_data[4];
    uint8_t *output_data[4];
    int     input_linesize[4];
    int     output_linesize[4];
    struct  SwsContext *sws_ctx;
    int     ret = 0;

    // int     output_bytes_size = width*height + width*height/2; // YUV420P (I420) Pixels
    // uint8_t output_byte[output_bytes_size];
    printf("RGB24 Pixels: W=%d * H=%d *3 = %d\n", width, height, width*height*3);
    input_data[0]     = rgb24data; // RGB24 have one plane
    input_linesize[0] = 3*width;   // RGB stride

    ret = av_image_alloc(input_data, input_linesize, width, height, 
                         AV_PIX_FMT_RGB24, 1);
    if ( ret< 0) {
        fprintf(stderr, "Could not allocate raw picture buffer\n");
        return ret;
    }
    ret = av_image_alloc(output_data, output_linesize, width, height,
                         AV_PIX_FMT_YUV420P, 32);
    if (ret < 0) {
        fprintf(stderr, "Could not allocate destination image\n");
        return 0;
    }
    
    // SWS_FAST_BILINEAR, SWS_BILINEAR, SWS_BICUBIC, SWS_X, SWS_POINT, 
    // SWS_AREA, SWS_BICUBLIN, SWS_GAUSS, SWS_SINC, SWS_LANCZOS, SWS_SPLINE
    sws_ctx = sws_getContext(width, height, AV_PIX_FMT_RGB24, 
                             width, height, AV_PIX_FMT_YUV420P, 
                             SWS_FAST_BILINEAR, 0, 0, 0);
    sws_scale(sws_ctx, (const uint8_t * const*)input_data, input_linesize, 0, 
              height, output_data, output_linesize);

    // height*output_linesize[0] + (height/2*output_linesize[1]) + (height*output_linesize[2])
    int ysize, usize, vsize;
    int i, y, idx;
    ysize = height*width;    // Y Planle pixels
    usize = height*width/4;  // U Planle pixels
    vsize = height*width/4;  // V Planle pixels
    printf("Merge YUV Color Pixels.\n");
    printf("Y size: %d U size: %d V size: %d\n", ysize, usize, vsize );
    printf("Y size: %d U size: %d V size: %d\n", output_linesize[0], output_linesize[1], output_linesize[2] );
    idx = 0;
    // for (i=0; i<ysize; i++,idx++){
    //     output_bytes[idx] = output_data[0][i];
    // }
    // for (i=0; i<usize; i++, idx++){
    //     output_bytes[idx] = output_data[1][i];
    // }
    // for (i=0; i<vsize; i++, idx++){
    //     output_bytes[idx] = output_data[2][i];
    // }
    for(i=0;i<height;i++){
        uint8_t *p = output_data[0]+output_linesize[0]*i;
        for (y=0; y<width; y++, idx++){
            output_bytes[idx] = p[y];
        }
    }
    for(i=0;i<height/2;i++){
        uint8_t *p = output_data[1]+output_linesize[1]*i;
        for (y=0; y<width/2; y++, idx++){
            output_bytes[idx] = p[y];
        }
    }
    for(i=0;i<height/2;i++){
        uint8_t *p = output_data[2]+output_linesize[2]*i;
        for (y=0; y<width/2; y++, idx++){
            output_bytes[idx] = p[y];
        }
    }

    // The offical demo code.
    // FILE *fp_out;
    // fp_out = fopen("test0.yuv", "wb");
    // for(i=0;i<height;i++){
    //     fwrite(output_data[0]+output_linesize[0]*i,1,width,fp_out);
    // }
    // for(i=0;i<height/2;i++){
    //     fwrite(output_data[1]+output_linesize[1]*i,1,width/2,fp_out);
    // }
    // for(i=0;i<height/2;i++){
    //     fwrite(output_data[2]+output_linesize[2]*i,1,width/2,fp_out);
    // }
    // return 1;
}

int rgb24_to_h264(uint8_t rgb24data[], int width, int height, uint8_t output_bytes[]){
    time_t          btime, etime;
    enum AVCodecID  codec_id = AV_CODEC_ID_H264;
    AVCodec         *codec;
    AVCodecContext  *c= NULL;
    int             i, ret, got_output;
    AVFrame         *frame;
    AVPacket        pkt;
    // MPEG EOF Flag
    // uint8_t         endcode[] = { 0, 0, 1, 0xb7 };

    btime = time(NULL);
    printf("Encode video file %s\n", filename);

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

    frame->pts = i;

    ret = avcodec_encode_video2(c, &pkt, frame, &got_output);
    if (ret < 0) {
        fprintf(stderr, "Error encoding frame\n");
        return 0;
    }

    // Debug
    FILE            *f;
    f = fopen("test.h264", "wb");
    if (!f) {
        fprintf(stderr, "Could not open %s\n", filename);
        return 0;
    }

    if (got_output) {
        printf("Write frame %3d (size=%5d)\n", i, pkt.size);
        // Return Video Packet.
        output_data[0] = pkt.size;
        output_data[1] = pkt.data;

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
            output_data[0] = pkt.size;
            output_data[1] = pkt.data;
            
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
}
int rgb24_to_mpegts(){

}
// int main(int argc, char const *argv[]){
//     int     width=1440,height=900;
//     FILE    *output_file, *input_file;

//     uint8_t input_data[width*height*3];

//     // uint8_t output_data[width*height + width*height/2];
//     int output_data_size = width*height + width*height/2;
//     uint8_t output_data[output_data_size];

//     input_file = fopen("test.rgb", "rb");
//     fread(input_data, 1, width*height*3, input_file);

//     int ret = rgb24_to_yuv420p(input_data, width, height, output_data);
//     printf("ret code: %d\n", ret);
//     // for (int i=0; i<output_data_size; i++){
//     //     printf("%d, ", output_data[i]);
//     // }
//     // printf("%d,%d,%d, %d,%d,%d\n", 
//     //         output_data[0],output_data[1],output_data[2], output_data[3],output_data[4],output_data[5]);
//     output_file = fopen("test.yuv", "wb");
//     if (!output_file) {
//         fprintf(stderr, "Could not open %s\n", "test.yuv");
//         exit(1);
//     }
//     fwrite(output_data, 1, output_data_size, output_file);
//     return 0;
// }

