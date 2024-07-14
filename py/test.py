from motors_controller import ScanConf, calc_scan_frame, Vector

conf = ScanConf(
        center = Vector(0, 0, 0),
        horiz_range = 10,
        vert_range = 10,
        horiz_step = 1,
        vert_step = 1)

(x, y, z) = calc_scan_frame(conf)

print(f"conf: {conf}")
print(f"x: {x.start} - {x.end}")
print(f"y: {y.start} - {y.end}")
print(f"z: {z.start} - {z.end}")
