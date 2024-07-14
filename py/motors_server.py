import serial
import json
import motors_controller as M

port_name = "/dev/ttyUSB0"

pos_x = 0
pos_y = 0


def main():
    port = serial.Serial(port_name, 38400)

    def send_response(payload):
        response = json.dumps(payload)
        print("Out:", response)
        port.write(bytes(response + "\n"))
        port.flush()

    def respond_ok():
        send_response({"status": "ok", "msg": ""})

    def respond_error(msg):
        send_response({"status": "error", "msg": msg})

    def scan_step_callback(x, y):
        # print("Pos: ", x, y)
        send_response({"tag": "ScanStep", "x": x, "y": y})

    def on_scan_start():
        print("Scan started")
        send_response({"tag": "ScanStarted"})

    M.init_scanner(on_scan_start, scan_step_callback)

    def check_stop_message():
        if port.inWaiting() > 0:
            msg = port.readline()
            command = json.loads(msg)
            tag = command["tag"]
            if tag == "stop_scan":
                print("Stopping scan")
                M.stop_scan()

    def loop_step():
        print("Waiting for command.")
        msg = port.readline()
        command = json.loads(msg)
        print("command:", command)

        tag = command["tag"]

        if tag == "set_pos":
            x = command["x"]
            y = command["y"]
            if isinstance(x, int) and isinstance(y, int):
                M.set_pos(x, y)
            else:
                print("Error: Invalid position")
            # if set_pos(x, y):
            #     respond_ok()
            # else:
            #     respond_error("Could not set position")

        elif tag == "set_conf":
            conf = command["scan_conf"]
            center = conf["center"]

            M.set_scan_conf(
                M.ScanConf(
                    center=M.Vector(center[0], center[1], center[2]),
                    horiz_step=conf["step_size"],
                    vert_step=conf["step_size"],
                    horiz_range=conf["horiz_range"],
                    vert_range=conf["vert_range"],
                )
            )

            send_response({"tag": "CurrentConf", "conf": conf})

        elif tag == "get_pos":
            pos = M.get_pos()
            if pos:
                (x, y) = pos
                send_response({"tag": "CurrentPos", "x": x, "y": y})
            # else:
            # respond_error("Could not get position")

        elif tag == "start_scan":
            print("Starting scan")
            M.scan(check_stop_message)

    while True:
        loop_step()


if __name__ == "__main__":
    main()
