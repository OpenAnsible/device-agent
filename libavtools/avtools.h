
void hello();

int bgra_to_yuv420p (uint8_t bgra_data[], int src_w, int src_h, 
                     uint8_t *dst_data[4], int dst_w, int dst_h);

int rgb24_to_yuv420p(uint8_t rgb24_data[], int src_w, int src_h, 
                     uint8_t *dst_data[4], int dst_w, int dst_h);

int rgb24_to_yuv420p_t(uint8_t rgb24_data[], int src_w, int src_h, uint8_t *dst_data[4]);
