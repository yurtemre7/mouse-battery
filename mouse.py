# Import the essential modules
import rivalcfg, pystray, os, time, threading
from PIL import Image, ImageDraw

# Our state variables
last_update = None
battery_level = None
battery_charging = None
icon = None
stopped = False
event = None

# Change the time_delta to your liking
time_delta = 60 * 1  # 60s * 1 = 60s = 1min
time_error = 60 * 0.2  # 60s * 0.2 = 12s

directory = f"{os.path.dirname(os.path.realpath(__file__))}/"
image_directory = f"{directory}images/"


# Fuction to create the menu
def create_menu(name, battery_level, last_update, battery_charging):
    return pystray.Menu(
        pystray.MenuItem(
            f"Name: {name}",
            lambda: None,
        ),
        pystray.MenuItem(
            f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}",
            lambda: None,
        ),
        pystray.MenuItem(
            (
                "Status: Charging"
                if battery_charging else "Status: Discharging"
            ),
            lambda: None,
        ),
        pystray.MenuItem(
            "Last update: "
            + time.strftime("%H:%M:%S", time.localtime(last_update))
            + f" (click to refresh or wait {time_delta if battery_level is not None else 1 / 20}s)",
            refresh_connection,
        ),
        pystray.MenuItem("Quit", quit_app),
    )


# Function to load the images
def load_image(image_name):
    return Image.open(f"{image_directory}{image_name}.png")


# Function to get the battery data
def get_battery(event: threading.Event):
    global stopped, icon, battery_level, last_update, battery_charging
    while not stopped:
        try:
            mouse = rivalcfg.get_first_mouse()
            print(f"Mouse found {mouse}")
            if mouse is None:
                print("No mouse found")
                time.sleep(1 / 20)
                raise Exception

            battery = mouse.battery
            battery = mouse.battery

            print(f"Mouse battery {battery}")

            if battery is not None:
                name = mouse.name
                if battery["level"] is not None:
                    battery_level = max(min(battery["level"], 100), 0)
                    last_update = time.time()
                    battery_charging = battery["is_charging"]
                icon.icon = create_battery_icon()
                icon.menu = create_menu(
                    name, battery_level, last_update, battery_charging
                )
                icon.title = f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}"
                icon.update_menu()
                sleeptime = time_delta if battery["level"] is not None else 1 / 20
                event.clear()
                event.wait(timeout=sleeptime)
            else:
                print("No battery found")
                time.sleep(1 / 20)
        except Exception as e:
            print(f"Error: {e}\n\nSleeping for {time_error} seconds...")
            time.sleep(time_error)
    mouse.close()
    print("Stopping thread")


# Änderungen in der Funktion create_battery_icon
def create_battery_icon():
    global battery_level
    global battery_charging
    image = Image.new("RGB", (100, 100), color="white")
    draw = ImageDraw.Draw(image)

    draw.rectangle((0, 0, 100, 100), fill="black")
    error = load_image("no_error")

    def draw_battery_indicator(color, level):
        draw.rectangle((0, 0, 100, 100), fill="black")
        draw.rectangle((0, 100 - level, 100, 100), fill=color)

    if battery_level is not None:
        if battery_charging:
            draw_battery_indicator("orange", battery_level)
        else:
            if battery_level < 20:
                draw_battery_indicator("red", battery_level)
            elif battery_level < 50:
                draw_battery_indicator("yellow", battery_level)
            else:
                draw_battery_indicator("green", battery_level)
    else:
        error = load_image("error")

    image.paste(error, (0, 0), error)

    image = image.convert("RGBA")
    data = image.getdata()
    new_data = []
    for item in data:
        if item[0] == 0 and item[1] == 0 and item[2] == 0:
            new_data.append((255, 255, 255, 0))
        else:
            new_data.append(item)
    image.putdata(new_data)

    return image


def refresh_connection():
    global event
    event.set()


# This function is called when you click on the quit button
def quit_app(icon, item):
    global stopped
    icon.stop()
    stopped = True


# This is the main function, where we initialize the system tray icon and start the thread
def main():
    global icon
    global event

    event = threading.Event()
    image = create_battery_icon()
    icon = pystray.Icon("Battery", icon=image, title="Battery: N/A")
    thread = threading.Thread(target=get_battery, args=(event,))
    thread.daemon = True
    thread.start()
    icon.menu = pystray.Menu(
        pystray.MenuItem(
            "Looking for mouse and mouse data...",
            lambda: None,
        ),
        pystray.MenuItem("Quit", quit_app),
    )
    icon.run()


# Python boilerplate
if __name__ == "__main__":
    main()
