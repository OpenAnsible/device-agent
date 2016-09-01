CC     = cc

CFLAGS       = -Wall -g -c 
INCLUDE_PATH = -I /usr/local/Cellar/ffmpeg/3.0.1/include
LINK_PATH    = -L /usr/local/Cellar/ffmpeg/3.0.1/lib
LINK_LIBS    = -lavdevice -lavformat -lavcodec -lavutil \
			   -lswscale -lswresample -lavfilter 

default: all

all: decoding_encoding scaling_video transcoding remuxing

decoding_encoding: decoding_encoding.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o decoding_encoding.o decoding_encoding.c;
	$(CC) decoding_encoding.o $(LINK_PATH) $(LINK_LIBS) -o decoding_encoding;

scaling_video: scaling_video.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o scaling_video.o scaling_video.c;
	$(CC) scaling_video.o $(LINK_PATH) $(LINK_LIBS) -o scaling_video;

transcoding: transcoding.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o transcoding.o transcoding.c;
	$(CC) transcoding.o $(LINK_PATH) $(LINK_LIBS) -o transcoding;

# muxing: muxing.c
# 	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o muxing.o muxing.c;
# 	$(CC) muxing.o $(LINK_PATH) $(LINK_LIBS) -o muxing;

remuxing: remuxing.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o remuxing.o remuxing.c;
	$(CC) remuxing.o $(LINK_PATH) $(LINK_LIBS) -o remuxing;

.PHONY: clean

clean:
	rm *.o;
	rm decoding_encoding scaling_video transcoding remuxing
	rm test*.pgm test.h264 test.mp2 test.sw test.mpg;

