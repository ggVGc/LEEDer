#ifndef __ICUBEDEFINES_H__
#define __ICUBEDEFINES_H__


//---------------------------------------------------------------------------
// Global Defines

#define		IC_SUCCESS		0
#define		IC_ERROR		1

// Additional error codes

#define		IC_IF_NOT_OPEN		-1
#define		IC_WRONG_PARAM		-2
#define		IC_OUT_OF_MEMORY	-3
#define		IC_ALREADY_DONE		-4
#define		IC_WRONG_CLOCK_VAL	-5
#define		IC_COM_LIB_INIT    	-6
#define		IC_NOT_IF_STARTED  	-7

// Video Mode definitions
// (used with: NETUSBCAM_SetMode,NETUSBCAM_GetMode)

#define		MAX_MODE_COUNT		10

#define		MOC_MODE_320x240	0
#define		MOC_MODE_640x480	1
#define		MOC_MODE_752x480	2
#define		MOC_MODE_800x600	3
#define		MOC_MODE_1024x768	4
#define		MOC_MODE_1280x1024	5
#define		MOC_MODE_1600x1200	6
#define		MOC_MODE_2048x1536	7
#define		MOC_MODE_2592x1944	8
#define		MOC_MODE_3840x2748	9

// BinSkip Definitions

#define MODE_SKIP		      	0
#define MODE_BIN		      	1

#define BIN_SKIP_OFF    	    	0
#define BIN_SKIP_2ND_PIXEL	    	1
#define BIN_SKIP_4TH_PIXEL	    	2

#define NET_1044_BIN_MIN_CLOCK_VAL	15

// Parameter definitions 
// (used with: NETUSBCAM_SetCamParameter,NETUSBCAM_GetCamParameter,
// NETUSBCAM_GetCamParameterRange,NETUSBCAM_GetParamAuto,NETUSBCAM_SetParamAuto,
// NETUSBCAM_SetParamAutoDef,NETUSBCAM_SetParamOnePush)

// Measure field definitions
// (used with REG_MEASURE_FIELD_AE)

#define MEASUREFIELD_100	     	2	// 100% (full frame)
#define MEASUREFIELD_60			1	// 60%
#define MEASUREFIELD_30			0	// 30%

// Shutter definitions
// (used with REG_SHUTTER)

#define SHUTTER_ROLLING	        	0	// all models
#define SHUTTER_GLOBAL_RESET		1	// (4133CU/BU,4133CU only)
#define SHUTTER_GLOBAL			2	// (4133CU/BU (default),4133CU (default) only)

// Parameter ON||OFF definitions

#define OFF                     	0
#define ON                      	1

// Trigger invert definitions
// hardware trigger will be detected on the defined edge
// (used with REG_TRIGGER_INVERT)

#define FALLING_EDGE			0	// (default)
#define RISING_EDGE			1	

// Software trigger mode definitions
// (used with REG_SW_TRIG_MODE)

#define DELAYED_TRIGGER_RETURN   	0  	// camera checks for frame arrival (default),
						// NETUSBCAM_SetTrigger(..,TRIG_SW_DO) returns this status
#define IMMEDIATE_TRIGGER_RETURN 	1	// no camera check

//----------------------------------------------------------
// Camera parameters definitions

#define REG_BRIGHTNESS			1	// all models
#define REG_CONTRAST			2	// all models
#define REG_GAMMA			3	// all models
#define REG_FLIPPED_V			4	// (ON||OFF); OFF (default)
#define REG_FLIPPED_H			5	// (ON||OFF); OFF (default)
#define REG_WHITE_BALANCE		6	// (one push)
#define REG_EXPOSURE_TIME		7	// all models
#define REG_EXPOSURE_TARGET		8	// sets the target-value for the auto exposure algorithm 
#define REG_RED				9	// only for color models; RGB Gain value
#define REG_GREEN			10	// only for color models; RGB Gain value
#define REG_BLUE			11	// only for color models; RGB Gain value
#define REG_BLACKLEVEL			12	// sensor blacklevel
#define REG_GAIN			13	// sensor gain
#define REG_COLOR			14	// (ON||OFF) read: color || bw model
#define REG_PLL				15	// all models
#define REG_STROBE_LENGHT		16	// length of strobe pulse output (msec)
#define REG_STROBE_DELAY		17	// delay before strobe pulse is executed (msec)
#define REG_TRIGGER_DELAY		18	// delay before hardware trigger is executed (msec)

#define REG_TRIGGER_INVERT		21	// (FALLING_EDGE (default) || RISING_EDGE)
#define REG_MEASURE_FIELD_AE		22	// measure field for auto exposure
#define REG_SHUTTER			26 	// (4133CU/BU,4133CU)
#define REG_ROI_ID			27	// (4133CU/BU,4133CU,fpga board cameras)
#define REG_ROI_CYCLE			28  	// (4133CU/BU,4133CU) number of repetitions of the particular roi

#define REG_DEFECT_COR			43	// (4133CU/BU,4133CU) DefectPixelCorrection (ON||OFF); OFF(default)

#define REG_SW_TRIG_MODE		94  	// (DELAYED_TRIGGER_RETURN (default) || IMMEDIATE_TRIGGER_RETURN)

#define REG_CALLBACK_BR_FRAMES	97		// Broken frames also triggering the callback function (ON||OFF); OFF(default)

#define REG_INVERT_PIXEL		113	// (ON||OFF)  bw cams only
#define REG_PIXEL_DEPTH			120	// 0==8bit/pixel, 1=16bit/pixel

#define REG_SENSOR_TIMING		123	// (4133CU/BU,4133CU)


//-------------------------------------------

#define DEFAULT_GAMMA			64


// Trigger definitions
// (used with: NETUSBCAM_SetTrigger, NETUSBCAM_GetTrigger)

#define TRIG_SW_START		0	// start software trigger
#define TRIG_SW_DO		1	// do a software trigger pulse
#define TRIG_HW_START 		2	// start hardware trigger
#define TRIG_STOP		3	// stop software || hardware trigger
#define TRIG_SW_START_2		4	// GlobalResetRelease shutter mode (1500CU/BU,1300CU)
#define TRIG_HW_START_2 	5	// GlobalResetRelease shutter mode (1500CU/BU,1300CU)


// Callback definitions
// (used with: NETUSBCAM_SetCallback)

#define CALLBACK_RAW		0	// 8bit/pixel data is received
					// (if the camera is a color model data == raw bayer)
#define CALLBACK_RGB		1	// 24bit/pixel data is received
#define CALLBACK_Y_8		2	// 8bit/pixel data is received (Y-channel only)


//----------------------------------------------------------
// Roi definitions
// (used with: REG_ROI_ID)

// ROI ID: (4133CU/BU)
// SIMR: Single Integration Multiple Roi  (4133CU/BU only)
// 1, 2 or 4 areas of interest to be acquired within the same integrated image
// MIMR: Multiple Integration Multiple Roi (4133CU/BU only)
// 
#define ROI_ID_1_2	        0	// SIMR (4133CU/BU only)
#define ROI_ID_1_1	        1	// default mode for all models (1st roi)
#define ROI_ID_2	        2   	// MIMR additonal (4133CU/BU only)
#define ROI_ID_3	        3
#define ROI_ID_4	        4




#endif // __ICUBEDEFINES_H__
