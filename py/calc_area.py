from numpy import sin, deg2rad

# from pprint import pprint


class Axis:
    def __init__(self):
        self.start = 0
        self.end = 0
        self.step = 0


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


def get_sub_conf(conf: ScanConf, cell_center_xy, cell_x_range, cell_y_range, new_step):
    # range_x = conf.horiz_range * conf.horiz_step
    # range_y = conf.horiz_range * conf.vert_step
    # min_x = conf.center.x - range_x / 2
    # min_y = conf.center.y - range_y / 2

    # print(f"range_x: {range_x}")
    # print(f"range_y: {range_y}")
    # print(f"min_x: {min_x}")
    # print(f"min_y: {min_y}")

    return None


small_1_conf = ScanConf(
    center=Vector(-0.8, 5.5, 58.25),
    horiz_range=12,
    vert_range=10,
    horiz_step=0.5,
    vert_step=0.5,
)


def real_pos_from_xy(conf, cell_x, cell_y):
    x = Axis()
    y = Axis()
    z = Axis()

    center = conf.center
    y.start = center.y + (conf.horiz_range / 2) * sin(deg2rad(45))
    y.end = center.y - (conf.horiz_range / 2) * sin(deg2rad(45))
    y.step = conf.horiz_step * sin(deg2rad(45))

    x.start = center.x - (conf.horiz_range / 2) * sin(deg2rad(45))
    x.end = center.x + (conf.horiz_range / 2) * sin(deg2rad(45))
    x.step = conf.horiz_step * sin(deg2rad(45))

    z.start = center.z - (conf.vert_range / 2)
    z.end = center.z + (conf.vert_range / 2)
    z.step = conf.vert_step

    z_positions = []
    x_positions = []
    y_positions = []

    for ii in range(int(abs(z.end - z.start) / z.step) + 1):
        if z.start > z.end:
            z_positions.append(z.start - (ii * z.step))
        else:
            z_positions.append(z.start + (ii * z.step))

    for ii in range(int(abs(x.end - x.start) / x.step) + 1):
        if x.start > x.end:
            x_positions.append(x.start - (ii * x.step))
        else:
            x_positions.append(x.start + (ii * x.step))
        if y.start > y.end:
            y_positions.append(y.start - (ii * y.step))
        else:
            y_positions.append(y.start + (ii * y.step))

    return (round(x_positions[cell_x], 3), round(y_positions[cell_x], 3), round(z_positions[cell_y], 3))


print(
    f"real_pos_from_xy(small_1_conf, 15, 11): {real_pos_from_xy(small_1_conf, 15, 11)}"
)

# sub_conf = get_sub_conf(small_1_conf, (15, 11), 2, 2, 0.1)


# def print_conf(conf):
#     v = vars(conf)
#     v['center'] = vars(v['center'])
#     pprint(v)

# print("sub_conf:")
# print_conf(sub_conf)
