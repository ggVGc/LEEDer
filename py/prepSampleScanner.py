import tango
import serial
import time
import sys 
import os
import datetime
import warnings
warnings.filterwarnings("ignore")
import numpy as np
import glob
import matplotlib.pyplot as plt
import matplotlib.colorbar as cbar



manipulatorAxis={}
manipulatorAxis["PM_x"]=tango.DeviceProxy("B110A-EA04/DIA/MP-01-X")
manipulatorAxis["PM_y"]=tango.DeviceProxy("B110A-EA04/DIA/MP-01-Y")
manipulatorAxis["PM_z"]=tango.DeviceProxy("B110A-EA04/DIA/MP-01-Z")
manipulatorAxis["PM_p"]=tango.DeviceProxy("B110A-EA04/DIA/MP-01-P")

manipulatorAxis["PS_x"]=tango.DeviceProxy("B110A-EA06/DIA/MP-01-X")
manipulatorAxis["PS_y"]=tango.DeviceProxy("B110A-EA06/DIA/MP-01-Y")
manipulatorAxis["PS_z"]=tango.DeviceProxy("B110A-EA06/DIA/MP-01-Z")
manipulatorAxis["PS_p"]=tango.DeviceProxy("B110A-EA06/DIA/MP-01-P")

pressure={}
pressure["PM"]=tango.DeviceProxy("B110A-EA04/VAC/VGH-01")
pressure["PS"]=tango.DeviceProxy("B110A-EA06/VAC/VGH-01")

def getPressure(whichGauge):
	return whichGauge.read_attribute('Pressure',wait=True).value

def serial_ports():
	#ports = ['COM%s' % (i+1) for i in range(256)]
	ports = glob.glob('/dev/tty[A-Za-z]*')
	result = []

	for port in ports:
		try:
			s = serial.Serial(port)
			s.close()
			#result.append(int(port.lstrip('COM')))
			result.append(port)
		except(OSError,serial.SerialException):
			#print("Problem opening port",port)
			pass
	return result

def getCurrent(serialPort):
	attempts=0
	while attempts<5:
		serialPort.setDTR(0)
		serialPort.write("MEAS?\n")
		response = serialPort.readline().split(",")		
		value = response[0][:-1]
		try:return float(value)
		except:pass
		attempts=attempts+1

# Tango seems to lie to me sometimes about whether the motors have stopped moving.
# So don't accept a 'yes' until you've had three in a row.
def isMovementFinished(manipulatorAxis):
	numberOfConfirmations =0
	while True:
		manipulatorMoving = manipulatorAxis.read_attribute('StatusMoving',wait=True).value
		if manipulatorMoving==True:
			return 0
		else:
			numberOfConfirmations=numberOfConfirmations+1
			if numberOfConfirmations==3:
				return 1
			time.sleep(0.05)

def waitForMotionToFinish(manipulatorAxis):
	delay_time=0.3
	timeout = 30
	counter=0
	while isMovementFinished(manipulatorAxis)==False:
		time.sleep(delay_time)
		counter+=1
		if counter>=timeout:
			print("Timed out waiting for motion to finish")
			exit()

def initiateManipulatorMovement(manipulatorAxis,destination):
	manipulatorAxis.write_attribute('position',destination)

# Axis 1 is the outer loop
# Axes 2 and 3 are in the inner loop.
def threeAxisRaster(ax1,ax1_name,ax1_start,ax1_end,ax1_step,ax2,ax2_name,ax2_start,ax2_end,ax2_step,ax3,ax3_name,ax3_start,ax3_end,ax3_step,recordPressure,pressureChannel,recordCurrent,serialPort,runForever,saveFile):
	ax1_positions=[]
	ax2_positions=[]
	ax3_positions=[]

	for ii in range(int(np.abs(ax1_end-ax1_start)/ax1_step)+1):
		if ax1_start>ax1_end:
			ax1_positions.append(ax1_start-(ii*ax1_step))
		else:
			ax1_positions.append(ax1_start+(ii*ax1_step))
	for ii in range(int(np.abs(ax2_end-ax2_start)/ax2_step)+1):
		if ax2_start>ax2_end:
			ax2_positions.append(ax2_start-(ii*ax2_step))
		else:
			ax2_positions.append(ax2_start+(ii*ax2_step))
		if ax3_start>ax3_end:
			ax3_positions.append(ax3_start-(ii*ax3_step))
		else:
			ax3_positions.append(ax3_start+(ii*ax3_step))

	if runForever==True:
		print("This will keep looping until you stop it")

	# ------------------------------------------------------
	# ---- Do the scan
	# ------------------------------------------------------
	direction_outerLoop=1
	direction_innerLoop=1


	print("Moving to initial position...")
	while isMovementFinished(ax1)==0 or isMovementFinished(ax2)==0 or isMovementFinished(ax3)==0:	
		time.sleep(0.25)		
	initiateManipulatorMovement(ax1,ax1_positions[0])
	initiateManipulatorMovement(ax2,ax2_positions[0])
	initiateManipulatorMovement(ax3,ax3_positions[0])

	while (isMovementFinished(ax1)==0 or isMovementFinished(ax2)==0 or isMovementFinished(ax3)==0):
		time.sleep(0.25)

		

	if recordPressure and recordCurrent:	
		currentMap=np.zeros((len(ax1_positions), len(ax2_positions)))
		pressureMap=np.zeros((len(ax1_positions), len(ax2_positions)))
		fig,plotAxis = plt.subplots(1,1)
		im=plotAxis.imshow(currentMap,aspect='auto',cmap='gray_r')
		plt.ion()
		fig.show()		
		print("\n{}\t{}\t{}\tCurrent\tPressure".format(ax1_name,ax2_name,ax3_name))	

	elif recordCurrent:	
		currentMap=np.zeros((len(ax1_positions), len(ax2_positions)))
		fig,plotAxis = plt.subplots(1,1)
		im=plotAxis.imshow(currentMap,aspect='auto',cmap='gray_r')
		plt.ion()
		fig.show()
		print("\n{}\t{}\t{}\tCurrent".format(ax1_name,ax2_name,ax3_name))	
	else:
		print("\n{}\t{}\t{}".format(ax1_name,ax2_name,ax3_name))	

	while True:
		if direction_outerLoop==1:
			for ax1_position in ax1_positions:

				initiateManipulatorMovement(ax1,ax1_position)
				waitForMotionToFinish(ax1)

				if direction_innerLoop==1:
					for ax2_position,ax3_position in zip(ax2_positions,ax3_positions):
						initiateManipulatorMovement(ax2,ax2_position)
						initiateManipulatorMovement(ax3,ax3_position)
						waitForMotionToFinish(ax2)
						waitForMotionToFinish(ax3)
						
						if recordPressure and recordCurrent:
							pressure,current=getPressure(pressureChannel),getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							pressureMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=pressure
							plt.cla()
							im=plotAxis.imshow(currentMap/pressureMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{}\t{}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current,pressure))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{}\t{}\n".format(ax1_position,ax2_position,ax3_position,current,pressure))
						elif recordCurrent:
							current=getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							plt.cla()
							im=plotAxis.imshow(currentMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.ylabel(ax1_name)
							plt.xlabel(ax2_name)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current))
						else:
							print("{:.2f}\t{:.2f}\t{:.2f}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position))

					direction_innerLoop=0
				elif direction_innerLoop==0:
					for ax2_position,ax3_position in zip(reversed(ax2_positions),reversed(ax3_positions)):
						initiateManipulatorMovement(ax2,ax2_position)
						initiateManipulatorMovement(ax3,ax3_position)
						waitForMotionToFinish(ax2)
						waitForMotionToFinish(ax3)

						if recordPressure and recordCurrent:
							pressure,current=getPressure(pressureChannel),getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							pressureMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=pressure
							plt.cla()
							im=plotAxis.imshow(currentMap/pressureMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{}\t{}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current,pressure))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{}\t{}\n".format(ax1_position,ax2_position,ax3_position,current,pressure))
						elif recordCurrent:
							current=getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							plt.cla()
							im=plotAxis.imshow(currentMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.ylabel(ax1_name)
							plt.xlabel(ax2_name)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current))
						else:
							print("{:.2f}\t{:.2f}\t{:.2f}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position))
					direction_innerLoop=1
			direction_outerLoop=0

		elif direction_outerLoop==0:
			for ax1_position in ax1_positions:

				initiateManipulatorMovement(ax1,ax1_position)
				waitForMotionToFinish(ax1)	
				if direction_innerLoop==1:
					for ax2_position,ax3_position in zip(ax2_positions,ax3_positions):
						initiateManipulatorMovement(ax2,ax2_position)
						initiateManipulatorMovement(ax3,ax3_position)
						waitForMotionToFinish(ax2)
						waitForMotionToFinish(ax3)

						if recordPressure and recordCurrent:
							pressure,current=getPressure(pressureChannel),getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							pressureMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=pressure
							plt.cla()
							im=plotAxis.imshow(currentMap/pressureMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current,pressure))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current,pressure))
						elif recordCurrent:
							current=getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							plt.cla()
							im=plotAxis.imshow(currentMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.ylabel(ax1_name)
							plt.xlabel(ax2_name)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current))
						else:
							print("{:.2f}\t{:.2f}\t{:.2f}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position))

					direction_innerLoop=0
				elif direction_innerLoop==0:
					for ax2_position,ax3_position in zip(reversed(ax2_positions),reversed(ax3_positions)):
						initiateManipulatorMovement(ax2,ax2_position)
						initiateManipulatorMovement(ax3,ax3_position)
						waitForMotionToFinish(ax2)
						waitForMotionToFinish(ax3)

						if recordPressure and recordCurrent:
							pressure,current=getPressure(pressureChannel),getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							pressureMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=pressure
							plt.cla()
							im=plotAxis.imshow(currentMap/pressureMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current,pressure))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current,pressure))
						elif recordCurrent:
							current=getCurrent(serialPort)
							currentMap[ax1_positions.index(ax1_position)][ax2_positions.index(ax2_position)]=current
							plt.cla()
							im=plotAxis.imshow(currentMap,aspect='auto',interpolation='none',cmap='gray_r')
							plt.yticks(range(len(ax1_positions)))
							plt.xticks(range(len(ax2_positions)))
							plotAxis.set_yticklabels(ax1_positions)
							plotAxis.set_xticklabels(ax2_positions)
							plt.ylabel(ax1_name)
							plt.xlabel(ax2_name)
							plt.gca().invert_yaxis()
							plt.pause(0.001)
							fig.show()
							print("{:.2f}\t{:.2f}\t{:.2f}\t{:.2e}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position,current))
							saveFile.write("{:.3f}\t{:.3f}\t{:.3f}\t{:.3e}\n".format(ax1_position,ax2_position,ax3_position,current))
						else:
							print("{:.2f}\t{:.2f}\t{:.2f}\t\t(ctrl+c to abort)".format(ax1_position,ax2_position,ax3_position))

					direction_innerLoop=1
			direction_outerLoop=1

		if runForever==False:
			temp=raw_input("\nDone! Press any key to exit")
			break



##########################################
if __name__ == '__main__':
##########################################
	print("\n-------------------------\nBLOCH sample scanner\n-------------------------")
	print("Options:")
	print("<Prep main>")
	print("\t1) Profile the LEED spot")
	print("\t2) Profile the sputter spot")
	print("\t3) Raster sample in front of sputter beam")
	print("\t4) Raster sample in front of LEED")
	print("<Prep Sec>")
	print("\t5) Profile the LEED spot")
	print("\t6) Profile the sputter spot")
	print("\t7) Raster sample in front of sputter beam")
	print("\t8) Raster sample in front of LEED")
	try:
		programMode=raw_input("\nChoose an option (1-8):  ")
		if(int(programMode)<1 or int(programMode)>8):
			print("Invalid response")
			exit()		
	except:
		print("Invalid response")
		exit()

	### Test connection to electrometer
	if int(programMode) in [1,2,5,6]:
		print("This option requires monitoring a beam current, so you need to tell me how to talk to the 6485 picoammeter.")
		
		foundCorrectCOM=False
		while foundCorrectCOM==False:

			print("\nCOM port numbers on this computer are:")
			portList = serial_ports()
			print(portList)

			response=raw_input("\nWhich one goes to the 6485 picoammeter?  (Trial and error is OK if you don't know)\n")
			if response in portList:
				try:
					#portname="COM"+str(int(response))
					serialPort = serial.Serial(
						port=response,
						baudrate=2400,
						parity=serial.PARITY_NONE,
						bytesize=serial.EIGHTBITS,
						stopbits=serial.STOPBITS_ONE,
						timeout=2
					)		

					serialPort.setDTR(0)
					serialPort.write("*CLS\n")
					serialPort.write("*RST\n")
					print("----> Connection OK <---- \nPicoammeter reading right now is: {}".format(getCurrent(serialPort)))
					foundCorrectCOM=True
				except:
					serialPort.close()
					print("NOPE! Couldn't connect to the picoammeter. \n\tIs this the correct port? \n\tIs the meter plugged in and turned on?\n\tTried rebooting the electrometer?\n (use ctrl+c to quit)")
			else:
				print("\nNOPE! That wasn't one of the options I offered you. (use ctrl+c to quit)")
		temp=raw_input("Press any key to continue  ")
		print("\n\n\n\n\n\n\n")
	### Test connection to pressure
	if int(programMode) ==2 :
		print("\nThis option also requires monitoring the pressure in prep main.")
		print("Right now I read that as {}".format(getPressure(pressure["PM"])))
		print("\n(note: PLC reading can differ from MG-15 reading)")
		temp=raw_input("Press any key to continue  ")
		print("\n\n\n\n\n\n\n")
	if int(programMode) ==6 :
		print("\nThis option also requires monitoring the pressure in prep sec.")
		print("Right now I read that as {}".format(getPressure(pressure["PS"])))
		print("\n(note: PLC reading can differ from MG-15 reading)")
		temp=raw_input("Press any key to continue  ")
		print("\n\n\n\n\n\n\n")

	#********************************************************
	if int(programMode) ==1:
	#********************************************************
		current_x_position=manipulatorAxis["PM_x"].read_attribute('position',wait=True).value
		print("\n\n--- Profile the Prep Main LEED spot ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the LEED beam")
		print("(x) will be left at its current value of {}".format(current_x_position))
		print("\nYou need to tell me the central (y,z) coordinate, the y- and z- total travel range and the step size\n\n")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you.\n\n")


		parametersApproved=False
		while parametersApproved==False:
			try:
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				y_range=float(raw_input("What should the total travel in y be?  "))
				z_range=float(raw_input("What should the total travel in z be?  "))
				y_step=float(raw_input("What should the y stepsize be?  "))
				z_step=float(raw_input("What should the z stepsize be?  "))
				if(y_range<0 or z_range<0 or y_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center-(y_range/2)
					y_end=y_center+(y_range/2)
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)			
					numDatapoints=((z_range/z_step)+1)*((y_range/y_step)+1)
					print("\n OK, so scanning from y,z = ({},{}) to ({},{}), measuring a total of {} positions".format(y_start,z_start,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
		saveFileName="PM_LEED_profile({0}-{1:02}-{2:02}_{3:02d}.{4:02d}.{5:02d}).txt".format(localtime.tm_year,localtime.tm_mon,localtime.tm_mday,localtime.tm_hour,localtime.tm_min,localtime.tm_sec)
		saveFile=open(saveFileName,'w')	
		saveFile.write("--- Profile the Prep Main LEED spot ---\n")
		saveFile.write("Measurement start time="+str(datetime.datetime.now())+"\n")
		saveFile.write("Scan axis 1 name=Y\n")
		saveFile.write("Scan axis 1 start={0}\n".format(y_start))
		saveFile.write("Scan axis 1 stop={0}\n".format(y_end))
		saveFile.write("Scan axis 1 step={0}\n".format(y_step))
		saveFile.write("Scan axis 2 name=Z\n")
		saveFile.write("Scan axis 2 start={0}\n".format(z_start))
		saveFile.write("Scan axis 2 stop={0}\n".format(z_end))
		saveFile.write("Scan axis 2 step={0}\n".format(z_step))
		saveFile.write("Scan axis 3 name=X\n")
		saveFile.write("Scan axis 3 start={0}\n".format(current_x_position))
		saveFile.write("Scan axis 3 stop={0}\n".format(current_x_position))
		saveFile.write("Scan axis 3 step=0\n")
		
		threeAxisRaster(
			ax1=manipulatorAxis["PM_y"],
			ax1_name="y",
			ax1_start=y_start,
			ax1_end=y_end,
			ax1_step=y_step,
			ax2=manipulatorAxis["PM_z"],
			ax2_name="z",
			ax2_start=z_start,
			ax2_end=z_end,
			ax2_step=z_step,
			ax3=manipulatorAxis["PM_x"],
			ax3_name="x",
			ax3_start=current_x_position,
			ax3_end=current_x_position,
			ax3_step=0,
			recordCurrent=True,
			serialPort=serialPort,
			recordPressure=False,
			pressureChannel=0,
			runForever=False,
			saveFile=saveFile)

	#********************************************************
	if int(programMode) ==2:
	#********************************************************
		print("\n\n--- Profile the Prep Main sputter spot ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the sputter beam")
		print("\n\nIf we say that the sample surface facing upwards corresponds to a polar angle of 33 degrees, then the sample surface is normal to the sputter gun at a polar angle of (33-90)= -57 deg")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing to the right from the perspective of the sputter gun)")
		print("\t(x,y,z)=(0,y,0) \t(pointing down from the perspective of the sputter gun.\n\n")
		
		parametersApproved=False
		while parametersApproved==False:
			#try:
			x_center=float(raw_input("What central x position should I use?  "))
			y_center=float(raw_input("What central y position should I use?  "))
			z_center=float(raw_input("What central z position should I use?  "))
				
			z_range=float(raw_input("What should the total horizontal (z) travel in the normal plane be?  "))
			y_range=float(raw_input("What should the total vertical (y) travel in the normal plane be?  "))
			z_step=float(raw_input("What should the horizontal (z) stepsize be?  "))
			y_step=float(raw_input("What should the vertical (y) stepsize be?  "))
				
			if(y_range<0 or z_range<0 or y_step<0 or z_step<0):
				print("Oops! Only positive numbers for the range and stepsize please")
				print("\nLet's try again:")
			else:
				y_start=y_center-(y_range/2)
				y_end=y_center+(y_range/2)
				x_start=x_center
				x_end=x_center
				x_step=0
				z_start=z_center-(z_range/2)
				z_end=z_center+(z_range/2)	

				numDatapoints=((z_range/z_step)+1)*((y_range/y_step)+1)
				print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
				response = raw_input("Type y or Y if that's OK, q or Q to quit or anything else to try again:  ")
				if response=='y' or response=='Y':
					parametersApproved=True
				if response=='q' or response=='Q':
					exit()
				else:
					print("\nOK then, let's try again:")	
			#except:
				#print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
		saveFileName="PM_sputter_profile({0}-{1:02}-{2:02}_{3:02d}.{4:02d}.{5:02d}).txt".format(localtime.tm_year,localtime.tm_mon,localtime.tm_mday,localtime.tm_hour,localtime.tm_min,localtime.tm_sec)
		saveFile=open(saveFileName,'w')	
		saveFile.write("--- Profile the Prep Main sputter spot ---\n")
		saveFile.write("Measurement start time="+str(datetime.datetime.now())+"\n")
		saveFile.write("Scan axis 1 name=Z\n")
		saveFile.write("Scan axis 1 start={0}\n".format(z_start))
		saveFile.write("Scan axis 1 stop={0}\n".format(z_end))
		saveFile.write("Scan axis 1 step={0}\n".format(z_step))
		saveFile.write("Scan axis 2 name=Y\n")
		saveFile.write("Scan axis 2 start={0}\n".format(y_start))
		saveFile.write("Scan axis 2 stop={0}\n".format(y_end))
		saveFile.write("Scan axis 2 step={0}\n".format(y_step))
		saveFile.write("Scan axis 3 name=X\n")
		saveFile.write("Scan axis 3 start={0}\n".format(x_start))
		saveFile.write("Scan axis 3 stop={0}\n".format(x_end))
		saveFile.write("Scan axis 3 step={0}\n".format(x_step))	
		
		threeAxisRaster(
			ax1=manipulatorAxis["PM_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PM_y"],
			ax2_name="y",
			ax2_start=y_start,
			ax2_end=y_end,
			ax2_step=y_step,
			ax3=manipulatorAxis["PM_x"],
			ax3_name="x",
			ax3_start=x_start,
			ax3_end=x_end,
			ax3_step=x_step,
			recordCurrent=True,
			serialPort=serialPort,
			recordPressure=True,
			pressureChannel=pressure["PM"],
			runForever=False,
			saveFile=saveFile)


	#********************************************************
	if int(programMode) ==3:
	#********************************************************

		print("\n\n--- Raster the sample in front of the Prep Main sputter spot ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the sputter beam")
		print("\n\nIf we say that the sample surface facing upwards corresponds to a polar angle of 33 degrees, then the sample surface is normal to the sputter gun at a polar angle of (33-90)= -57 deg")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing to the right from the perspective of the sputter gun)")
		print("\t(x,y,z)=(0,y,0) \t(pointing down from the perspective of the sputter gun.\n\n")
		
		
		parametersApproved=False
		while parametersApproved==False:
			x_center=float(raw_input("What central x position should I use?  "))
			y_center=float(raw_input("What central y position should I use?  "))
			z_center=float(raw_input("What central z position should I use?  "))
				
			z_range=float(raw_input("What should the total horizontal (z) travel in the normal plane be?  "))
			y_range=float(raw_input("What should the total vertical (y) travel in the normal plane be?  "))
			z_step=float(raw_input("What should the horizontal (z) stepsize be?  "))
			y_step=float(raw_input("What should the vertical (y) stepsize be?  "))
				
			if(y_range<0 or z_range<0 or y_step<0 or z_step<0):
				print("Oops! Only positive numbers for the range and stepsize please")
				print("\nLet's try again:")
			else:
				y_start=y_center-(y_range/2)
				y_end=y_center+(y_range/2)
				x_start=x_center
				x_end=x_center
				x_step=0
				z_start=z_center-(z_range/2)
				z_end=z_center+(z_range/2)	

				numDatapoints=((z_range/z_step)+1)*((y_range/y_step)+1)
				print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
				response = raw_input("Type y or Y if that's OK, q or Q to quit or anything else to try again:  ")
				if response=='y' or response=='Y':
					parametersApproved=True
				if response=='q' or response=='Q':
					exit()
				else:
					print("\nOK then, let's try again:")		
		
		now = time.time()
		localtime = time.localtime(now) 
	
		threeAxisRaster(
			ax1=manipulatorAxis["PM_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PM_y"],
			ax2_name="y",
			ax2_start=y_start,
			ax2_end=y_end,
			ax2_step=y_step,
			ax3=manipulatorAxis["PM_x"],
			ax3_name="x",
			ax3_start=x_start,
			ax3_end=x_end,
			ax3_step=x_step,
			recordCurrent=False,
			serialPort=0,
			recordPressure=False,
			pressureChannel=0,
			runForever=True,
			saveFile=0)

	#********************************************************
	if int(programMode) ==4:
	#********************************************************
		current_x_position=manipulatorAxis["PM_x"].read_attribute('position',wait=True).value
		print("\n\n--- Raster the sample in front of the Prep Main LEED ---")
		print("\nI'm going to raster the sample in (y,z) in a square pattern")
		print("\n(x) will be left at its current value of {}".format(current_x_position))
		print("\nYou need to tell me the central (y,z) coordinate, the y- and z- total travel range and the step size\n\n")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you.\n\n")

		parametersApproved=False
		while parametersApproved==False:
			try:
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				y_range=float(raw_input("What should the total travel in y be?  "))
				z_range=float(raw_input("What should the total travel in z be?  "))
				y_step=float(raw_input("What should the y stepsize be?  "))
				z_step=float(raw_input("What should the z stepsize be?  "))
				if(y_range<0 or z_range<0 or y_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center-(y_range/2)
					y_end=y_center+(y_range/2)
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)			
					numDatapoints=((z_range/z_step)+1)*((y_range/y_step)+1)
					print("\n OK, so scanning from y,z = ({},{}) to ({},{}), measuring a total of {} positions".format(y_start,z_start,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		threeAxisRaster(
			ax1=manipulatorAxis["PM_y"],
			ax1_name="y",
			ax1_start=y_start,
			ax1_end=y_end,
			ax1_step=y_step,
			ax2=manipulatorAxis["PM_z"],
			ax2_name="z",
			ax2_start=z_start,
			ax2_end=z_end,
			ax2_step=z_step,
			ax3=manipulatorAxis["PM_x"],
			ax3_name="x",
			ax3_start=current_x_position,
			ax3_end=current_x_position,
			ax3_step=0,
			recordCurrent=False,
			serialPort=0,
			recordPressure=False,
			pressureChannel=0,
			runForever=True,
			saveFile=0)


	#********************************************************
	if int(programMode) ==5:
	#********************************************************

		print("\n\n--- Profile the Prep Sec LEED spot ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the LEED beam")
		print("\n\nIf we say that the sample surface facing away from ring corresponds to a polar angle of 0 degrees, then the sample surface is normal to the LEED at a polar angle of 45 degrees")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\n[I am assuming that +x is away from the RDC, and +y is away from the ring]")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing up from the perspective of the LEED) (i.e. if sample goes down, spot position on sample goes up")
		print("\t(x,y,z)=(-sin(45),sin(45),0) \t(pointing left from the perspective of the LEED)")
		print("\n\nIf you just tell me the manipulator (x,y,z) coordinates of the starting point, I'll take care of the linked (x,y) motion.\n\n")
		
		parametersApproved=False
		while parametersApproved==False:
			try:
				x_center=float(raw_input("What central x position should I use?  "))
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				
				horizontal_range=float(raw_input("What should the total horizontal (x,y) travel in the normal plane be?  "))
				z_range=float(raw_input("What should the total vertical(z) travel in the normal plane be?  "))
				horizontal_step=float(raw_input("What should the horizontal stepsize be?  "))
				z_step=float(raw_input("What should the vertical stepsize be?  "))
				
				if(horizontal_range<0 or z_range<0 or horizontal_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_end=y_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_step=horizontal_step*np.sin(np.deg2rad(45))
					x_start=x_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_end=x_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_step=horizontal_step*np.sin(np.deg2rad(45))
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)	

					numDatapoints=((z_range/z_step)+1)*((horizontal_range/horizontal_step)+1)
					print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK:  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK then, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
		saveFileName="PS_LEED_profile({0}-{1:02}-{2:02}_{3:02d}.{4:02d}.{5:02d}).txt".format(localtime.tm_year,localtime.tm_mon,localtime.tm_mday,localtime.tm_hour,localtime.tm_min,localtime.tm_sec)
		saveFile=open(saveFileName,'w')	
		saveFile.write("--- Profile the Prep Sec LEED spot ---\n")
		saveFile.write("Measurement start time="+str(datetime.datetime.now())+"\n")
		saveFile.write("Scan axis 1 name=Z\n")
		saveFile.write("Scan axis 1 start={0}\n".format(z_start))
		saveFile.write("Scan axis 1 stop={0}\n".format(z_end))
		saveFile.write("Scan axis 1 step={0}\n".format(z_step))
		saveFile.write("Scan axis 2 name=X\n")
		saveFile.write("Scan axis 2 start={0}\n".format(x_start))
		saveFile.write("Scan axis 2 stop={0}\n".format(x_end))
		saveFile.write("Scan axis 2 step={0}\n".format(x_step))
		saveFile.write("Scan axis 3 name=Y\n")
		saveFile.write("Scan axis 3 start={0}\n".format(y_start))
		saveFile.write("Scan axis 3 stop={0}\n".format(y_end))
		saveFile.write("Scan axis 3 step={0}\n".format(y_step))	

		threeAxisRaster(
			ax1=manipulatorAxis["PS_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PS_x"],
			ax2_name="x",
			ax2_start=x_start,
			ax2_end=x_end,
			ax2_step=x_step,
			ax3=manipulatorAxis["PS_y"],
			ax3_name="y",
			ax3_start=y_start,
			ax3_end=y_end,
			ax3_step=y_step,
			recordCurrent=True,
			serialPort=serialPort,
			recordPressure=False,
			pressureChannel=0,
			runForever=False,
			saveFile=saveFile)

	#********************************************************
	if int(programMode) ==6:
	#********************************************************
		print("\n\n--- Profile the Prep Sec sputter beam ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the sputter beam")
		print("\n\nIf we say that the sample surface facing away from ring corresponds to a polar angle of 0 degrees, then the sample surface is normal to the sputter gun (port#7) at a polar angle of -140 degrees")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\n[I am assuming that +x is away from the RDC, and +y is away from the ring]")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing up from the perspective of the LEED) (i.e. if sample goes down, spot position on sample goes up")
		print("\t(x,y,z)=(-sin(45),sin(45),0) \t(pointing left from the perspective of the LEED)")
		print("\n\nIf you just tell me the manipulator (x,y,z) coordinates of the starting point, I'll take care of the linked (x,y) motion.\n\n")
		
		parametersApproved=False
		while parametersApproved==False:
			try:
				x_center=float(raw_input("What central x position should I use?  "))
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				
				horizontal_range=float(raw_input("What should the total horizontal (x,y) travel in the normal plane be?  "))
				z_range=float(raw_input("What should the total vertical(z) travel in the normal plane be?  "))
				horizontal_step=float(raw_input("What should the horizontal stepsize be?  "))
				z_step=float(raw_input("What should the vertical stepsize be?  "))
				
				if(horizontal_range<0 or z_range<0 or horizontal_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_end=y_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_step=horizontal_step*np.sin(np.deg2rad(45))
					x_start=x_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_end=x_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_step=horizontal_step*np.sin(np.deg2rad(45))
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)

					numDatapoints=((z_range/z_step)+1)*((horizontal_range/horizontal_step)+1)
					print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK:  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK then, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
		saveFileName="PS_sputter_profile({0}-{1:02}-{2:02}_{3:02d}.{4:02d}.{5:02d}).txt".format(localtime.tm_year,localtime.tm_mon,localtime.tm_mday,localtime.tm_hour,localtime.tm_min,localtime.tm_sec)
		saveFile=open(saveFileName,'w')	
		saveFile.write("--- Profile the Prep Sec sputter spot ---\n")
		saveFile.write("Measurement start time="+str(datetime.datetime.now())+"\n")
		saveFile.write("Scan axis 1 name=Z\n")
		saveFile.write("Scan axis 1 start={0}\n".format(z_start))
		saveFile.write("Scan axis 1 stop={0}\n".format(z_end))
		saveFile.write("Scan axis 1 step={0}\n".format(z_step))
		saveFile.write("Scan axis 2 name=X\n")
		saveFile.write("Scan axis 2 start={0}\n".format(x_start))
		saveFile.write("Scan axis 2 stop={0}\n".format(x_end))
		saveFile.write("Scan axis 2 step={0}\n".format(x_step))
		saveFile.write("Scan axis 3 name=Y\n")
		saveFile.write("Scan axis 3 start={0}\n".format(y_start))
		saveFile.write("Scan axis 3 stop={0}\n".format(y_end))
		saveFile.write("Scan axis 3 step={0}\n".format(y_step))			

		threeAxisRaster(
			ax1=manipulatorAxis["PS_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PS_x"],
			ax2_name="x",
			ax2_start=x_start,
			ax2_end=x_end,
			ax2_step=x_step,
			ax3=manipulatorAxis["PS_y"],
			ax3_name="y",
			ax3_start=y_start,
			ax3_end=y_end,
			ax3_step=y_step,
			recordCurrent=True,
			serialPort=serialPort,
			recordPressure=True,
			pressureChannel=pressure["PS"],
			runForever=False,
			saveFile=saveFile)


	#********************************************************
	if int(programMode) ==7:
	#********************************************************
		print("\n\n--- Raster the sample in front of the Prep Sec sputter beam ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the sputter beam")
		print("\n\nIf we say that the sample surface facing away from ring corresponds to a polar angle of 0 degrees, then the sample surface is normal to the sputter gun (port#7) at a polar angle of -140 degrees")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\n[I am assuming that +x is away from the RDC, and +y is away from the ring]")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing up from the perspective of the LEED) (i.e. if sample goes down, spot position on sample goes up")
		print("\t(x,y,z)=(cos(50),sin(50),0) \t(pointing right from the perspective of the sputter gun)")
		print("\n\nIf you just tell me the manipulator (x,y,z) coordinates of the starting point, I'll take care of the linked (x,y) motion.\n\n")
		
		parametersApproved=False
		while parametersApproved==False:
			try:
				x_center=float(raw_input("What central x position should I use?  "))
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				
				horizontal_range=float(raw_input("What should the total horizontal (x,y) travel in the normal plane be?  "))
				z_range=float(raw_input("What should the total vertical(z) travel in the normal plane be?  "))
				horizontal_step=float(raw_input("What should the horizontal stepsize be?  "))
				z_step=float(raw_input("What should the vertical stepsize be?  "))
				
				if(horizontal_range<0 or z_range<0 or horizontal_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center+(horizontal_range/2)*np.cos(np.deg2rad(50))
					y_end=y_center-(horizontal_range/2)*np.cos(np.deg2rad(50))
					y_step=horizontal_step*np.cos(np.deg2rad(50))
					x_start=x_center+(horizontal_range/2)*np.sin(np.deg2rad(50))
					x_end=x_center-(horizontal_range/2)*np.sin(np.deg2rad(50))
					x_step=horizontal_step*np.sin(np.deg2rad(50))
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)	

					numDatapoints=((z_range/z_step)+1)*((horizontal_range/horizontal_step)+1)
					print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK:  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK then, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
	
		threeAxisRaster(
			ax1=manipulatorAxis["PS_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PS_x"],
			ax2_name="x",
			ax2_start=x_start,
			ax2_end=x_end,
			ax2_step=x_step,
			ax3=manipulatorAxis["PS_y"],
			ax3_name="y",
			ax3_start=y_start,
			ax3_end=y_end,
			ax3_step=y_step,
			recordCurrent=False,
			serialPort=0,
			recordPressure=False,
			pressureChannel=0,
			runForever=True,
			saveFile=0)

	#********************************************************
	if int(programMode) ==8:
	#********************************************************
		print("\n\n--- Raster the sample in front of the Prep Sec LEED ---")
		print("\nI'm going to raster the sample in a square pattern, normal to the LEED beam")
		print("\n\nIf we say that the sample surface facing away from ring corresponds to a polar angle of 0 degrees, then the sample surface is normal to the LEED at a polar angle of 45 degrees")
		print("\n\n--->>> Make sure the polar angle is correct! <<---\nI don't know if the polar axis is homed, so I'm not going to set this for you. ")
		print("\n\n[[[[I am assuming that +y is towards the RDC, and +x is away from the ring]]]]")
		print("\n\nIn this geometry, orthogonal unit vectors in the normal plane are:")
		print("\t(x,y,z)=(0,0,z) \t(pointing up from the perspective of the LEED)")
		print("\t(x,y,z)=(-sin(45),-sin(45),0) \t(pointing right from the perspective of the LEED)")
		print("\n\nIf you just tell me the manipulator (x,y,z) coordinates of the starting point, I'll take care of the linked (x,y) motion.\n\n")
		
		parametersApproved=False
		while parametersApproved==False:
			try:
				x_center=float(raw_input("What central x position should I use?  "))
				y_center=float(raw_input("What central y position should I use?  "))
				z_center=float(raw_input("What central z position should I use?  "))
				
				horizontal_range=float(raw_input("What should the total horizontal (x,y) travel in the normal plane be?  "))
				z_range=float(raw_input("What should the total vertical(z) travel in the normal plane be?  "))
				horizontal_step=float(raw_input("What should the horizontal stepsize be?  "))
				z_step=float(raw_input("What should the vertical stepsize be?  "))
				
				if(horizontal_range<0 or z_range<0 or horizontal_step<0 or z_step<0):
					print("Oops! Only positive numbers for the range and stepsize please")
					print("\nLet's try again:")
				else:
					y_start=y_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_end=y_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					y_step=horizontal_step*np.sin(np.deg2rad(45))
					x_start=x_center+(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_end=x_center-(horizontal_range/2)*np.sin(np.deg2rad(45))
					x_step=horizontal_step*np.sin(np.deg2rad(45))
					z_start=z_center-(z_range/2)
					z_end=z_center+(z_range/2)	

					numDatapoints=((z_range/z_step)+1)*((horizontal_range/horizontal_step)+1)
					print("\n OK, so scanning from x,y,z = ({},{},{}) to ({},{},{}), measuring a total of {} positions".format(x_start,y_start,z_start,x_end,y_end,z_end,numDatapoints))
					response = raw_input("Type y or Y if that's OK:  ")
					if response=='y' or response=='Y':
						parametersApproved=True
					else:
						print("\nOK then, let's try again:")	
			except:
				print("Oops! Please only enter valid numbers\n\nLet's try again:")
		
		now = time.time()
		localtime = time.localtime(now) 
	
		threeAxisRaster(
			ax1=manipulatorAxis["PS_z"],
			ax1_name="z",
			ax1_start=z_start,
			ax1_end=z_end,
			ax1_step=z_step,
			ax2=manipulatorAxis["PS_x"],
			ax2_name="x",
			ax2_start=x_start,
			ax2_end=x_end,
			ax2_step=x_step,
			ax3=manipulatorAxis["PS_y"],
			ax3_name="y",
			ax3_start=y_start,
			ax3_end=y_end,
			ax3_step=y_step,
			recordCurrent=False,
			serialPort=0,
			recordPressure=False,
			pressureChannel=0,
			runForever=True,
			saveFile=False)
