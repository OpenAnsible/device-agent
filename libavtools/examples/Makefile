CC     = cc

CFLAGS       = -Wall -g -c 
INCLUDE_PATH = -I /usr/local/Cellar/ffmpeg/3.0.1/include
LINK_PATH    = -L /usr/local/Cellar/ffmpeg/3.0.1/lib
LINK_LIBS    = -lavdevice -lavformat -lavfilter -lavcodec \
			   -lswresample -lswscale -lavutil

default: all

all: decoding_encoding scaling_video

decoding_encoding: decoding_encoding.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o decoding_encoding.o decoding_encoding.c;
	$(CC) decoding_encoding.o $(LINK_PATH) $(LINK_LIBS) -o decoding_encoding;

scaling_video: scaling_video.c
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o scaling_video.o scaling_video.c;
	$(CC) scaling_video.o $(LINK_PATH) $(LINK_LIBS) -o scaling_video;

.PHONY: clean

clean:
	rm decoding_encoding.o;rm decoding_encoding;
	rm scaling_video.o;    rm scaling_video;
