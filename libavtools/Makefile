CC     = cc
AR     = ar

CFLAGS       = -Wall -g -c 
INCLUDE_PATH = -I /usr/local/Cellar/ffmpeg/3.0.1/include
LINK_PATH    = -L /usr/local/Cellar/ffmpeg/3.0.1/lib
LINK_LIBS    = -lavdevice -lavformat -lavfilter -lavcodec \
			   -lswresample -lswscale -lavutil

LIB_SOURCES = avtools.c
BIN_SOURCES = avtools.c

LIB_TARGET  = libavtools.a
OBJ_TARGET  = avtools.o
BIN_TARGET  = avtools

default: bin

# cc -I /usr/local/Cellar/ffmpeg/3.0.1/include -Wall -g -c \
#         -o avtools.o avtools.c
# cc avtools.o -L /usr/local/Cellar/ffmpeg/3.0.1/lib \
#         -lavdevice -lavformat -lavfilter -lavcodec \
#         -lswresample -lswscale -lavutil  \
#         -o avtools

lib: $(LIB_TARGET) $(LIB_SOURCES)
bin: $(BIN_TARGET) $(BIN_SOURCES)

$(OBJ_TARGET): $(LIB_SOURCES)
	$(CC) $(INCLUDE_PATH) $(CFLAGS) -o $(OBJ_TARGET) $(LIB_SOURCES);

$(LIB_TARGET): $(OBJ_TARGET)
	$(AR) rcs $(LIB_TARGET) $(OBJ_TARGET);

$(BIN_TARGET): $(OBJ_TARGET)
	$(CC) $(OBJ_TARGET) $(LINK_PATH) $(LINK_LIBS) -Wl,-e,_entry -Wno-error=deprecated-declarations -o $(BIN_TARGET);

.PHONY: clean

clean:
	rm *.o *.a $(BIN_TARGET);