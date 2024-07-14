import serial

p = serial.Serial("/dev/ttyUSB0")
p.write(b"Hello!\n")
