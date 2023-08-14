# Import the essential modules
import rivalcfg
import pystray
from PIL import Image, ImageDraw
import os, time, threading

# Our state variables
last_update = None
battery_level = None
icon = None
stopped = False

# Change the time_delta to your liking
time_delta = 60 * 10  # 60s * 10 = 600s = 10min

directory = f"{os.path.dirname(os.path.realpath(__file__))}/"
image_directory = f"{directory}images/"

# This function is running in a separate thread to get the battery level of your mouse
def get_battery():
    global stopped
    while not stopped:
        mouse = rivalcfg.get_first_mouse()
        if mouse is None:
            print("No mouse found")
            time.sleep(1 / 20)
            continue
        battery = mouse.battery

        if battery["level"] is not None:
            # print(f"Mouse got {battery['level']}% juice left")
            global battery_level, icon, last_update
            battery_level = max(min(battery["level"], 100), 0)
            last_update = time.time()
            icon.icon = create_battery_icon()
            icon.menu = pystray.Menu(
                pystray.MenuItem(
                    f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}",
                    lambda: None,
                ),
                pystray.MenuItem(
                    "Last update: "
                    + time.strftime("%H:%M:%S", time.localtime(last_update)),
                    lambda: None,
                ),
                pystray.MenuItem("Quit", quit_app),
            )
            icon.title = f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}"
            icon.update_menu()
            time.sleep(time_delta)
        else:
            time.sleep(1 / 20)
    print("Stopping thread")


# This function creates the system tray icon dynamically
def create_battery_icon():
    global battery_level
    image = Image.new("RGB", (100, 100), color="white")
    draw = ImageDraw.Draw(image)

    if battery_level is not None:
        draw.rectangle((0, 0, 100, 100), fill="black")
        draw.rectangle((0, 100 - battery_level, 100, 100), fill="green")
    else:
        draw.rectangle((0, 0, 100, 100), fill="black")
        error = Image.open(f"{image_directory}error.png")
        image.paste(error, (0, 0), error)

    # load image error.png and put on top
    if battery_level is not None and battery_level < 20:
        error = Image.open(f"{image_directory}error.png")
    else:
        error = Image.open(f"{image_directory}no_error.png")
    image.paste(error, (0, 0), error)

    # replace color black with transparent
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


# This function is called when you click on the quit button
def quit_app(icon, item):
    global stopped
    icon.stop()
    stopped = True


# This is the main function, where we initialize the system tray icon and start the thread
def main():
    global icon
    image = create_battery_icon()
    icon = pystray.Icon("Battery", icon=image, title="Battery: N/A")
    thread = threading.Thread(target=get_battery)
    thread.start()
    icon.menu = pystray.Menu(
        pystray.MenuItem(
            f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}",
            lambda: None,
        ),
        pystray.MenuItem("Quit", quit_app),
    )

    icon.run()


# Python boilerplate
if __name__ == "__main__":
    main()
