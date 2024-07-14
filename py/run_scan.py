import motors_controller


def on_start():
    pass


def on_step(x, y):
    pass


conf = motors_controller.ScanConf(
    center=motors_controller.Vector(-0.8, 5.5, 58.25),
    horiz_range=2,
    vert_range=2,
    horiz_step=0.5,
    vert_step=0.5,
)
motors_controller.scan(conf, on_start, on_step)
