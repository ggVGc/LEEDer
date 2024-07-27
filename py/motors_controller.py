import numpy as np
import time

# Initially based on prepSampleScanner.py (included in this repository for reference)

with_tango = True
step_time = 0.5

class Logger:
    def __init__(self):
        self.listeners = []

    def write_log_message(self, msg):
        print(msg)


logger = Logger()
log = logger.write_log_message


class ScanConf:
    def __init__(self, center, horiz_range, horiz_step, vert_range, vert_step):
        self.center = center
        self.horiz_range = horiz_range
        self.vert_range = vert_range
        self.horiz_step = horiz_step
        self.vert_step = vert_step


class Vector:
    def __init__(self, x=0, y=0, z=0):
        self.x = x
        self.y = y
        self.z = z


class Attr:
    def __init__(self, value):
        self.value = value


class DummyDevice:
    def __init__(self, name):
        self.name = name

    def read_attribute(self, attr, **kwargs):
        if attr == "StatusMoving":
            return Attr(False)
        else:
            raise ("Unhandled attribute read:" + attr)

    def write_attribute(self, attr, value):
        log("Write attribute: " + self.name + "." + attr + " = " + str(value))


if with_tango:
    import tango


def get_device_proxy(name, path):
    if with_tango:
        return tango.DeviceProxy(path)
    else:
        return DummyDevice(name)


def scan(callback):
    if scanner:
        scanner.scan(callback)
    else:
        log("Error: Scanner is not initialized")


def stop_scan():
    scanner.stop()


def dummy_scan(on_start_callback, step_callback):
    on_start_callback()
    for x in range(0, 20):
        for y in range(0, 20):
            step_callback(x, y)


class Axis:
    def __init__(self, manipulator):
        self.manipulator = manipulator
        self.start = 0
        self.end = 0
        self.step = 0


def build_axes(conf):
    x = Axis(get_device_proxy("X", "B110A-EA06/DIA/MP-01-X"))
    y = Axis(get_device_proxy("Y", "B110A-EA06/DIA/MP-01-Y"))
    z = Axis(get_device_proxy("Z", "B110A-EA06/DIA/MP-01-Z"))

    if (
        conf.horiz_range < 0
        or conf.vert_range < 0
        or conf.horiz_step < 0
        or conf.vert_step < 0
    ):
        raise ("Oops! Only positive numbers for the range and stepsize please")
    else:
        center = conf.center
        horiz_range = conf.horiz_range
        horiz_step = conf.horiz_step

        y.start = center.y + (horiz_range / 2) * np.sin(np.deg2rad(45))
        y.end = center.y - (horiz_range / 2) * np.sin(np.deg2rad(45))
        y.step = horiz_step * np.sin(np.deg2rad(45))

        x.start = center.x - (horiz_range / 2) * np.sin(np.deg2rad(45))
        x.end = center.x + (horiz_range / 2) * np.sin(np.deg2rad(45))
        x.step = horiz_step * np.sin(np.deg2rad(45))

        z.start = center.z - (conf.vert_range / 2)
        z.end = center.z + (conf.vert_range / 2)
        z.step = conf.vert_step

        return (x, y, z)


class Scanner:
    def __init__(self, start_callback, step_callback):
        self.step_callback = step_callback
        self.start_callback = start_callback
        self.reset()

    def set_axes(self, ax1, ax2, ax3):
        log("Setting scanner axes")
        self.ax1 = ax1
        self.ax2 = ax2
        self.ax3 = ax3

        self.ax1_positions = []
        self.ax2_positions = []
        self.ax3_positions = []

        for ii in range(int(np.abs(ax1.end - ax1.start) / ax1.step) + 1):
            if ax1.start > ax1.end:
                self.ax1_positions.append(ax1.start - (ii * ax1.step))
            else:
                self.ax1_positions.append(ax1.start + (ii * ax1.step))

        for ii in range(int(np.abs(ax2.end - ax2.start) / ax2.step) + 1):
            if ax2.start > ax2.end:
                self.ax2_positions.append(ax2.start - (ii * ax2.step))
            else:
                self.ax2_positions.append(ax2.start + (ii * ax2.step))
            if ax3.start > ax3.end:
                self.ax3_positions.append(ax3.start - (ii * ax3.step))
            else:
                self.ax3_positions.append(ax3.start + (ii * ax3.step))

    def reset(self):
        self.should_stop = False
        self.cur_index_x = 0
        self.cur_index_y = 0
        self.dir_x = 1

    def move_to_bottom_left(self):
        log("Moving to initial position...")
        self.move_to_location(0, 0)

    def set_location(self, x, y):
        if x < 0 or x >= len(self.ax1_positions):
            log("Error: x position out of range. x=" + str(x))
            return False
        elif y < 0 or y >= len(self.ax1_positions):
            log("Error: y position out of range. y=" + str(y))
            return False

        ax1_position = self.ax1_positions[y]
        initiateManipulatorMovement(self.ax1, ax1_position)
        waitForMotionToFinish(self.ax1)

        ax2_position = self.ax2_positions[x]
        ax3_position = self.ax3_positions[x]
        initiateManipulatorMovement(self.ax2, ax2_position)
        initiateManipulatorMovement(self.ax3, ax3_position)
        waitForMotionToFinish(self.ax2)
        waitForMotionToFinish(self.ax3)

        log(
            "{:.2f}\t{:.2f}\t{:.2f}\t\t(ctrl+c to abort)".format(
                ax1_position, ax2_position, ax3_position
            )
        )
        log("")
        self.cur_index_x = x
        self.cur_index_y = y
        time.sleep(step_time)
        self.step_callback(self.cur_index_x, self.cur_index_y)
        return True

    def move_to_location(self, x, y):
        def dir_from_diff(val):
            if val > 0:
                return 1

            elif val < 0:
                return -1
            else:
                return 0

        while self.cur_index_x != x or self.cur_index_y != y:
            while x != self.cur_index_x or y != self.cur_index_y:
                dir_x = dir_from_diff(x - self.cur_index_x)
                dir_y = dir_from_diff(y - self.cur_index_y)
                print(dir_x, dir_y)

                if not self.set_location(
                    self.cur_index_x + dir_x, self.cur_index_y + dir_y
                ):
                    break

    def step(self):
        log("Scan step")
        log("")

        if self.cur_index_x < len(self.ax2_positions) and self.cur_index_y < len(
            self.ax1_positions
        ):
            self.set_location(self.cur_index_x, self.cur_index_y)

            if self.dir_x > 0:
                self.cur_index_x += 1
                if self.cur_index_x >= len(self.ax2_positions):
                    self.cur_index_y += 1
                    self.cur_index_x = len(self.ax2_positions) - 1
                    self.dir_x = -1
            else:
                self.cur_index_x -= 1
                if self.cur_index_x <= 0:
                    self.cur_index_y += 1
                    self.cur_index_x = 0
                    self.dir_x = 1

            return True
        else:
            log("Scan done!")
            return False

    def scan(self, callback):
        self.reset()
        self.should_stop = False
        self.move_to_bottom_left()
        self.start_callback()
        while scanner.step():
            callback()
            if self.should_stop:
                log("Scan stopped!")
                return

        log("Scan Finished!")

    def stop(self):
        self.should_stop = True


scanner = None


def init_scanner(on_start_callback, step_callback):
    global scanner
    scanner = Scanner(
        start_callback=on_start_callback,
        step_callback=step_callback,
    )


def set_scan_conf(conf):
    (x, y, z) = build_axes(conf)

    numDatapoints = ((conf.vert_range / z.step) + 1) * (
        (conf.horiz_range / conf.horiz_step) + 1
    )

    log("Start:")
    log("  x: %.04f" % (x.start))
    log("  y: %.04f" % (y.start))
    log("  z: %.04f" % (z.start))
    log("")

    log("End:")
    log("  x: %.04f" % (x.end))
    log("  y: %.04f" % (y.end))
    log("  z: %.04f" % (z.end))

    log("")
    log("Total data points: %i" % (numDatapoints))

    scanner.set_axes(
        ax1=z,
        ax2=x,
        ax3=y,
    )


# Tango seems to lie to me sometimes about whether the motors have stopped moving.
# So don't accept a 'yes' until you've had three in a row.
def isMovementFinished(axis):
    numberOfConfirmations = 0
    while True:
        manipulatorMoving = axis.manipulator.read_attribute(
            "StatusMoving", wait=True
        ).value
        if manipulatorMoving == True:
            return 0
        else:
            numberOfConfirmations = numberOfConfirmations + 1
            if numberOfConfirmations == 3:
                return 1
            time.sleep(0.05)


def initiateManipulatorMovement(axis, destination):
    axis.manipulator.write_attribute("position", destination)


def waitForMotionToFinish(axis):
    delay_time = 0.3
    timeout = 100
    counter = 0
    while isMovementFinished(axis) == False:
        time.sleep(delay_time)
        counter += 1
        if counter >= timeout:
            log("Timed out waiting for motion to finish")
            exit()


def set_pos(x, y):
    print("Setting pos", x, y)
    if scanner:
        scanner.move_to_location(x, y)
    else:
        log("Error: Scanner is not initialized")


# TODO
def get_pos():
    pass
