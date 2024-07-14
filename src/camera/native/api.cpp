#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "ICubeDefines.h"
#include "NETUSBCAM_API.h"

static const size_t EXPECTED_IMG_SIZE = 3932160;
static const size_t MAX_IMG_SIZE = EXPECTED_IMG_SIZE * 10;
static const int CamIndex = 0;

unsigned int good_count = 0;
unsigned int bad_count = 0;
unsigned int last_image_size = 0;
unsigned char last_image_bytes[MAX_IMG_SIZE + 1];

/*
void save_raw(unsigned char *buffer, unsigned int buffersize,
              const char *cName) {
  // printf("Saving image!\n");
  FILE *outfile = fopen(cName, "wb");
  if (!outfile) {
    // printf("Error fopen\n");
    return;
  }
  fwrite(buffer, 1, buffersize, outfile);
  fclose(outfile);
}
*/
int on_frame_buffer(void *buffer, unsigned int buffersize, void * /*context*/) {
  // printf("Got frame!\n");
  if (buffersize == 0 ||
      buffersize >= MAX_IMG_SIZE) { // badframe arrived (this happens here,
                                    // when (REG_CALLBACK_BR_FRAMES==1)
    bad_count++;
    last_image_size = 0;
  } else {
    good_count++;
    last_image_size = buffersize;
    mempcpy(last_image_bytes, buffer, buffersize * sizeof(char));

    NETUSBCAM_SaveToFile(CamIndex, "live_image.bmp");
    // save_raw((unsigned char *)buffer, buffersize, "last_image.raw");
  }

  /*
  printf("Got Image;  GoodFr: %d ,BadFr: %d , Size %d  \n", good_count,
  bad_count, buffersize);
  */

  return 0;
}

void print_props() {
  PARAM_PROPERTY prop;
  unsigned long cur;
#define show_param(param)                                                      \
  NETUSBCAM_GetCamParameterRange(CamIndex, param, &prop);                      \
  NETUSBCAM_GetCamParameter(CamIndex, param, &cur);                            \
  printf("%s: cur = %lu, min =%u, max = %lu, default = %u\n", #param, cur,     \
         prop.nMin, prop.nMax, prop.nDef);

  show_param(REG_SENSOR_TIMING);
  show_param(REG_FLIPPED_H);
  show_param(REG_GAIN);
  show_param(REG_PLL);
  show_param(REG_BRIGHTNESS);
  show_param(REG_GAMMA);
  show_param(REG_CONTRAST);
  show_param(REG_BLACKLEVEL);
}

extern "C" {

int8_t api_camera_init() {
  int result = NETUSBCAM_Init(); // look for ICubes
  if (result == 0) {
    return 0;
  }

#define checked(body)                                                          \
  result = body;                                                               \
  if (result != EXIT_SUCCESS) {                                                \
    return 0;                                                                  \
  }

  checked(NETUSBCAM_Open(CamIndex));

  // set the camera clock to 20Mhz
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_PLL, 20));

  // if active, badframes are sent to the callback with buffersize = 0
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_CALLBACK_BR_FRAMES, 1));

  // set the callback to get the frame buffer
  checked(
      NETUSBCAM_SetCallback(CamIndex, CALLBACK_RGB, &on_frame_buffer, NULL));
  checked(NETUSBCAM_SetMode(CamIndex, 4)); // 1024x758
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_FLIPPED_V, 1));
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_GAIN, 0));
  checked(NETUSBCAM_SetExposure(CamIndex, 300));

  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_BRIGHTNESS, 128));
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_CONTRAST, 256));
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_GAMMA, 56));
  checked(NETUSBCAM_SetCamParameter(CamIndex, REG_BLACKLEVEL, 128));

  print_props();

  return 1;
}

int8_t api_camera_start() {
  const int result = NETUSBCAM_Start(CamIndex);

  if (result != 0) {
    // printf("Error: Start; Result = %d\n", result);
    return 0;
  }

  return 1;
}

int8_t api_camera_stop() {
  int result = NETUSBCAM_Stop(CamIndex);
  if (result != 0) {
    // printf("Error: Stop; Result = %d\n", result);
    return 0;
  }

  // close camera
  result = NETUSBCAM_Close(CamIndex);
  if (result != 0) {
    // printf("Error: Close; Result = %d\n", result);
    return 0;
  }

  return 1;
}

int32_t api_camera_good_images() {
  return good_count;
}

int32_t api_camera_bad_images() {
  return bad_count;
}

int32_t api_camera_save_file(const char *path) {
  const int result = NETUSBCAM_SaveToFile(CamIndex, path);

  if (result != IC_SUCCESS) {
    return 0;
  }

  return 1;
}

int8_t api_camera_set_exposure(int32_t milliseconds) {
  const int result = NETUSBCAM_SetExposure(CamIndex, milliseconds);
  if (result != 0) {
    // printf("Error: Start; Result = %d\n", result);
    return 0;
  }

  return 1;
}
}
