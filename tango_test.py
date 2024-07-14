import tango

def test():
    axis = tango.DeviceProxy("B110A-EA01/DIA/MP-01-Z")


if __name__ == "__main__":
    test()
