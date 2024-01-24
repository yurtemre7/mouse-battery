# Import the essential modules
import rivalcfg, pystray, os, time, threading
from PIL import Image, ImageDraw

# Our state variables
last_update = None
battery_level = None
icon = None
stopped = False

# Change the time_delta to your liking
time_delta = 60 * 10  # 60s * 10 = 600s = 10min
time_error = 60 * 0.2  # 60s * 0.2 = 12s

directory = f"{os.path.dirname(os.path.realpath(__file__))}/"
image_directory = f"{directory}images/"


# Fuction to create the menu
def create_menu(name, battery_level, last_update):
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
            "Last update: "
            + time.strftime("%H:%M:%S", time.localtime(last_update))
            + f' (next update in {time_delta if battery_level is not None else 1 / 20}s)',
            lambda: None,
        ),
        pystray.MenuItem("Quit", quit_app),
    )

# Function to load the images
def load_image(image_name):
    return Image.open(f"{image_directory}{image_name}.png")

# Function to get the battery data
def get_battery():
    global stopped, icon, battery_level, last_update
    while not stopped:
        try:
            mouse = rivalcfg.get_first_mouse()
            print(f"Mouse found {mouse}")
            if mouse is None:
                print("No mouse found")
                time.sleep(1 / 20)
                continue

            battery = mouse.battery
            battery = mouse.battery
            
            print(f"Mouse battery {battery}")

            if battery is not None:
                name = mouse.name
                if battery["level"] is not None:
                    battery_level = max(min(battery["level"], 100), 0)
                    last_update = time.time()
                icon.icon = create_battery_icon()
                icon.menu = create_menu(name, battery_level, last_update)
                icon.title = f"Battery: {str(f'{battery_level}%' if battery_level is not None else 'N/A')}"
                icon.update_menu()
                time.sleep(time_delta if battery["level"] is not None else 1 / 20)
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
    image = Image.new("RGB", (100, 100), color="white")
    draw = ImageDraw.Draw(image)

    if battery_level is not None:
        draw.rectangle((0, 0, 100, 100), fill="black")
        draw.rectangle((0, 100 - battery_level, 100, 100), fill="green")
    else:
        draw.rectangle((0, 0, 100, 100), fill="black")
        error = load_image("error")
        image.paste(error, (0, 0), error)

    if battery_level is not None and battery_level < 20:
        error = load_image("error")
    else:
        error = load_image("no_error")
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
