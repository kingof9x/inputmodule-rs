import os
import platform
import sys
import threading
import webbrowser

import tkinter as tk
from tkinter import ttk, messagebox

from inputmodule import firmware_update
from inputmodule.inputmodule import (
    send_command,
    get_version,
    brightness,
    get_brightness,
    bootloader_jump,
    CommandVals,
    Game,
    GameControlVal
)
from inputmodule.gui.ledmatrix import countdown, random_eq, clock
from inputmodule.gui.gui_threading import stop_thread, is_dev_disconnected
from inputmodule.inputmodule.ledmatrix import (
    percentage,
    pattern,
    animate,
    PATTERNS,
    PWM_FREQUENCIES,
    show_symbols,
    show_string,
    pwm_freq,
    image_bl,
    image_greyscale,
    set_startup_animation,
    get_startup_animation,
    StartupMode,
    STARTUP_ANIMATIONS,
)

def update_brightness_slider(devices):
    average_brightness = None
    for dev in devices:
        if not average_brightness:
            average_brightness = 0

        br = get_brightness(dev)
        average_brightness += br
    if average_brightness:
        brightness_scale.set(average_brightness)

def popup(message, gui=True):
    if gui:
        messagebox.showinfo("Framework Laptop 16 LED Matrix", message)

def run_gui(devices):
    root = tk.Tk()
    root.title("LED Matrix Control")

    ico = "framework_startmenuicon.ico"
    res_path = resource_path()
    if os.name == 'nt':
        root.iconbitmap(f"{res_path}/res/{ico}")

    tabControl = ttk.Notebook(root)
    tab1 = ttk.Frame(tabControl)
    tab_games = ttk.Frame(tabControl)
    tab2 = ttk.Frame(tabControl)
    tab_fw = ttk.Frame(tabControl)
    tab3 = ttk.Frame(tabControl)
    tabControl.add(tab1, text="Home")
    tabControl.add(tab_games, text="Games")
    tabControl.add(tab2, text="Dynamic Controls")
    tabControl.add(tab_fw, text="Firmware Update")
    tabControl.add(tab3, text="Advanced")
    tabControl.pack(expand=1, fill="both")

    # Device Checkboxes
    detected_devices_frame = ttk.LabelFrame(root, text="Detected Devices", style="TLabelframe")
    detected_devices_frame.pack(fill="x", padx=10, pady=5)

    global device_checkboxes
    device_checkboxes = {}
    versions = {}
    startup_settings = {}
    # Devices whose port another program (like a system monitor) has open.
    # On Windows only one program at a time can open a serial port.
    in_use = set()
    for dev in devices:
        check_device(dev, versions, startup_settings, in_use)
        checkbox_var = tk.BooleanVar(value=True)
        checkbox = ttk.Checkbutton(detected_devices_frame, text=device_info(dev, versions), variable=checkbox_var, style="TCheckbutton")
        checkbox.pack(anchor="w")
        device_checkboxes[dev.name] = (checkbox_var, checkbox)

    # Online Info
    info_frame = ttk.LabelFrame(tab1, text="Online Info", style="TLabelframe")
    info_frame.pack(fill="x", padx=10, pady=5)
    infos = {
        "Web Interface": "https://ledmatrix.frame.work",
        "Latest Releases": "https://github.com/FrameworkComputer/inputmodule-rs/releases",
        "Hardware Info": "https://github.com/FrameworkComputer/InputModules",
    }
    for (i, (text, url)) in enumerate(infos.items()):
        # Organize in columns of three
        row = int(i / 3)
        column = i % 3
        btn = ttk.Button(info_frame, text=text, command=lambda url=url: webbrowser.open(url), style="TButton")
        btn.grid(row=row, column=column)

    # Brightness Slider
    brightness_frame = ttk.LabelFrame(tab1, text="Brightness", style="TLabelframe")
    brightness_frame.pack(fill="x", padx=10, pady=5)
    global brightness_scale
    brightness_scale = tk.Scale(brightness_frame, from_=0, to=255, orient='horizontal', command=lambda value: set_brightness(devices, value))
    brightness_scale.set(120)  # Default value
    brightness_scale.pack(fill="x", padx=5, pady=5)

    # Startup Animation
    startup_frame = ttk.LabelFrame(tab1, text="Startup Animation", style="TLabelframe")
    startup_frame.pack(fill="x", padx=10, pady=5)
    startup_widgets(startup_frame, devices, startup_settings, in_use)

    # Animation Control
    animation_frame = ttk.LabelFrame(tab1, text="Animation", style="TLabelframe")
    animation_frame.pack(fill="x", padx=10, pady=5)
    animation_buttons = {
        "Start Animation": "start_animation",
        "Stop Animation": "stop_animation"
    }
    for text, action in animation_buttons.items():
        ttk.Button(animation_frame, text=text, command=lambda a=action: perform_action(devices, a), style="TButton").pack(side="left", padx=5, pady=5)

    # Pattern Combo Box
    pattern_frame = ttk.LabelFrame(tab1, text="Pattern", style="TLabelframe")
    pattern_frame.pack(fill="x", padx=10, pady=5)
    pattern_combo = ttk.Combobox(pattern_frame, values=PATTERNS, style="TCombobox", state="readonly")
    pattern_combo.pack(fill="x", padx=5, pady=5)
    pattern_combo.bind("<<ComboboxSelected>>", lambda event: set_pattern(devices, pattern_combo.get()))

    # Percentage Slider
    percentage_frame = ttk.LabelFrame(tab1, text="Fill screen X% (could be volume indicator)", style="TLabelframe")
    percentage_frame.pack(fill="x", padx=10, pady=5)
    percentage_scale = tk.Scale(percentage_frame, from_=0, to=100, orient='horizontal', command=lambda value: set_percentage(devices, value))
    percentage_scale.pack(fill="x", padx=5, pady=5)

    # Games tab
    games_frame = ttk.LabelFrame(tab_games, text="Interactive", style="TLabelframe")
    games_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(games_frame, text="Snake", command=lambda: perform_action(devices, 'game_snake'), style="TButton").pack(side="left", padx=5, pady=5)
    ttk.Button(games_frame, text="Ledris", command=lambda: perform_action(devices, 'game_ledris'), style="TButton").pack(side="left", padx=5, pady=5)
    gol_frame = ttk.LabelFrame(tab_games, text="Game of Life", style="TLabelframe")
    gol_frame.pack(fill="x", padx=10, pady=5)
    animation_buttons = {
        "Current": "gol_current",
        "Pattern 1": "gol_pattern1",
        "Pattern 2": "gol_pattern2",
        "Pattern 3": "gol_pattern3",
        "Pattern 4": "gol_pattern4",
        "Pattern 5": "gol_pattern5",
        "Blinker": "gol_blinker",
        "Toad": "gol_toad",
        "Beacon": "gol_beacon",
        "Glider": "gol_glider",
        "R-pentomino": "gol_rpentomino",
        "Acorn": "gol_acorn",
        "Diehard": "gol_diehard",
        "Light spaceship": "gol_lwss",
        "Medium spaceship": "gol_mwss",
        "Heavy spaceship": "gol_hwss",
        "Spaceship fleet": "gol_fleet",
        "Octagon": "gol_octagon",
        "Tumbler": "gol_tumbler",
        "Caterer": "gol_caterer",
        "Stop": "game_stop",
    }
    for (i, (text, action)) in enumerate(animation_buttons.items()):
        # Organize in columns of three
        row = int(i / 3)
        column = i % 3
        if action == "game_stop":
            column = 0
            row += 1
        btn = ttk.Button(gol_frame, text=text, command=lambda a=action: perform_action(devices, a), style="TButton")
        btn.grid(row=row, column=column)

    # Countdown Timer
    countdown_frame = ttk.LabelFrame(tab2, text="Countdown Timer", style="TLabelframe")
    countdown_frame.pack(fill="x", padx=10, pady=5)
    countdown_spinbox = tk.Spinbox(countdown_frame, from_=1, to=60, width=5, textvariable=tk.StringVar(value=10))
    countdown_spinbox.pack(side="left", padx=5, pady=5)
    ttk.Label(countdown_frame, text="Seconds", style="TLabel").pack(side="left")
    ttk.Button(countdown_frame, text="Start", command=lambda: start_countdown(devices, countdown_spinbox.get()), style="TButton").pack(side="left", padx=5, pady=5)
    ttk.Button(countdown_frame, text="Stop", command=stop_thread, style="TButton").pack(side="left", padx=5, pady=5)

    # Black & White and Greyscale Images in same row
    image_frame = ttk.LabelFrame(tab1, text="Black&White Images / Greyscale Images", style="TLabelframe")
    image_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(image_frame, text="Send stripe.gif", command=lambda: send_image(devices, "stripe.gif", image_bl), style="TButton").pack(side="left", padx=5, pady=5)
    ttk.Button(image_frame, text="Send greyscale.gif", command=lambda: send_image(devices, "greyscale.gif", image_greyscale), style="TButton").pack(side="left", padx=5, pady=5)

    # Display Current Time
    time_frame = ttk.LabelFrame(tab2, text="Display Current Time", style="TLabelframe")
    time_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(time_frame, text="Start", command=lambda: perform_action(devices, "start_time"), style="TButton").pack(side="left", padx=5, pady=5)
    ttk.Button(time_frame, text="Stop", command=stop_thread, style="TButton").pack(side="left", padx=5, pady=5)

    # Custom Text
    custom_text_frame = ttk.LabelFrame(tab1, text="Custom Text", style="TLabelframe")
    custom_text_frame.pack(fill="x", padx=10, pady=5)
    custom_text_entry = ttk.Entry(custom_text_frame, width=20, style="TEntry")
    custom_text_entry.pack(side="left", padx=5, pady=5)
    ttk.Button(custom_text_frame, text="Show", command=lambda: show_custom_text(devices, custom_text_entry.get()), style="TButton").pack(side="left", padx=5, pady=5)

    # Display Text with Symbols
    symbols_frame = ttk.LabelFrame(tab1, text="Display Text with Symbols", style="TLabelframe")
    symbols_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(symbols_frame, text="Send '2 5 degC thunder'", command=lambda: send_symbols(devices), style="TButton").pack(side="left", padx=5, pady=5)

    # Firmware Update
    installed_fw_frame = ttk.LabelFrame(tab_fw, text="Installed Firmware", style="TLabelframe")
    installed_fw_frame.pack(fill="x", padx=10, pady=5)
    installed_fw_labels = {}
    for dev in devices:
        label = ttk.Label(installed_fw_frame, text=installed_fw_text(dev, versions, startup_settings, in_use), style="TLabel")
        label.pack(anchor="w", padx=5, pady=2)
        installed_fw_labels[dev.name] = label

    def recheck_in_use():
        """Keep checking modules that another program is using, until it lets go of them"""
        if not in_use:
            return
        freed = False
        for dev in devices:
            if dev.name in in_use:
                check_device(dev, versions, startup_settings, in_use)
                if dev.name not in in_use:
                    freed = True
                    device_checkboxes[dev.name][1].configure(text=device_info(dev, versions))
                    installed_fw_labels[dev.name].configure(text=installed_fw_text(dev, versions, startup_settings, in_use))
        if freed:
            for child in startup_frame.winfo_children():
                child.destroy()
            startup_widgets(startup_frame, devices, startup_settings, in_use)
        root.after(3000, recheck_in_use)
    root.after(3000, recheck_in_use)

    bootloader_frame = ttk.LabelFrame(tab_fw, text="Bootloader", style="TLabelframe")
    bootloader_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(bootloader_frame, text="Enter Bootloader", command=lambda: perform_action(devices, "bootloader"), style="TButton").pack(side="left", padx=5, pady=5)

    bundled_fw_frame = ttk.LabelFrame(tab_fw, text="Bundled Updates", style="TLabelframe")
    bundled_fw_frame.pack(fill="x", padx=10, pady=5)
    releases = firmware_update.find_releases(resource_path(), r'(ledmatrix).uf2')
    if not releases:
        tk.Label(bundled_fw_frame, text="Cannot find firmware updates").pack(side="top", padx=5, pady=5)
    else:
        versions = sorted(list(releases.keys()), reverse=True)

        #tk.Label(fw_update_frame, text="Ignore user configured keymap").pack(side="top", padx=5, pady=5)
        fw_ver_combo = ttk.Combobox(bundled_fw_frame, values=versions, style="TCombobox", state="readonly")
        fw_ver_combo.pack(side=tk.LEFT, padx=5, pady=5)
        fw_ver_combo.current(0)
        flash_btn = ttk.Button(bundled_fw_frame, text="Update", command=lambda: tk_flash_firmware(devices, releases, fw_ver_combo.get(), 'ledmatrix'), style="TButton")
        flash_btn.pack(side="left", padx=5, pady=5)
        # The bundled firmware is the official release, downloaded when building the app
        ttk.Label(bundled_fw_frame, text="Official firmware from Framework", style="TLabel").pack(side="left", padx=5, pady=5)
        source_btn = ttk.Button(bundled_fw_frame, text="Release Notes", command=lambda: webbrowser.open(f"https://github.com/FrameworkComputer/inputmodule-rs/releases/tag/v{fw_ver_combo.get()}"), style="TButton")
        source_btn.pack(side="left", padx=5, pady=5)

    # PWM Frequency Combo Box
    pwm_freq_frame = ttk.LabelFrame(tab3, text="PWM Frequency", style="TLabelframe")
    pwm_freq_frame.pack(fill="x", padx=10, pady=5)
    pwm_freq_combo = ttk.Combobox(pwm_freq_frame, values=PWM_FREQUENCIES, style="TCombobox", state="readonly")
    pwm_freq_combo.pack(fill="x", padx=5, pady=5)
    pwm_freq_combo.bind("<<ComboboxSelected>>", lambda: set_pwm_freq(devices, pwm_freq_combo.get()))

    # Equalizer
    equalizer_frame = ttk.LabelFrame(tab2, text="Equalizer", style="TLabelframe")
    equalizer_frame.pack(fill="x", padx=10, pady=5)
    ttk.Button(equalizer_frame, text="Start random equalizer", command=lambda: perform_action(devices, "start_eq"), style="TButton").pack(side="left", padx=5, pady=5)
    ttk.Button(equalizer_frame, text="Stop", command=stop_thread, style="TButton").pack(side="left", padx=5, pady=5)

    # Device Control Buttons
    device_control_frame = ttk.LabelFrame(tab3, text="Device Control", style="TLabelframe")
    device_control_frame.pack(fill="x", padx=10, pady=5)
    control_buttons = {
        "Sleep": "sleep",
        "Wake": "wake"
    }
    for text, action in control_buttons.items():
        ttk.Button(device_control_frame, text=text, command=lambda a=action: perform_action(devices, a), style="TButton").pack(side="left", padx=5, pady=5)

    root.mainloop()

def check_device(dev, versions, startup_settings, in_use):
    """Read the firmware version and startup animation setting of a module.
    Adds the module to in_use if another program has its port open."""
    try:
        startup_settings[dev.name] = get_startup_animation(dev)
        in_use.discard(dev.name)
        versions[dev.name] = get_version(dev)
    except (IOError, OSError):
        startup_settings[dev.name] = None
        in_use.add(dev.name)
        versions[dev.name] = "In use by another program"

def device_info(dev, versions):
    return f"{dev.name}\nSerial No: {dev.serial_number}\nFW Version:{versions[dev.name]}"

def installed_fw_text(dev, versions, startup_settings, in_use):
    if dev.name in in_use:
        return f"{dev.name}: In use by another program. Close it, this updates by itself."
    startup_support = "supported" if startup_settings[dev.name] else "not supported"
    return f"{dev.name}: Version {versions[dev.name]}, startup animation setting {startup_support}"

def startup_widgets(frame, devices, startup_settings, in_use):
    """Controls to choose which animation is shown when the module powers on"""
    current = None
    for dev in devices:
        current = startup_settings[dev.name]
        if current:
            break

    mode_var = tk.IntVar(value=StartupMode.RandomAll)
    animation_combo = ttk.Combobox(frame, values=STARTUP_ANIMATIONS, style="TCombobox", state="readonly")
    animation_combo.current(0)
    set_vars = [tk.BooleanVar(value=True) for _ in STARTUP_ANIMATIONS]
    if current:
        (mode, animation, animation_set) = current
        mode_var.set(mode)
        if animation < len(STARTUP_ANIMATIONS):
            animation_combo.current(animation)
        for (i, var) in enumerate(set_vars):
            var.set(bool(animation_set & (1 << i)))

    set_frame = ttk.Frame(frame)
    set_checkboxes = []
    for (i, name) in enumerate(STARTUP_ANIMATIONS):
        checkbox = ttk.Checkbutton(set_frame, text=name, variable=set_vars[i], style="TCheckbutton")
        # Organize in columns of two
        checkbox.grid(row=int(i / 2), column=i % 2, sticky="w", padx=5)
        set_checkboxes.append(checkbox)

    def update_enabled():
        animation_combo.configure(state="readonly" if mode_var.get() == StartupMode.Fixed else "disabled")
        for checkbox in set_checkboxes:
            checkbox.configure(state="normal" if mode_var.get() == StartupMode.RandomSet else "disabled")

    modes = {
        "Random": StartupMode.RandomAll,
        "Always this one:": StartupMode.Fixed,
        "Random out of these:": StartupMode.RandomSet,
        "None": StartupMode.NoAnimation,
    }
    for (text, mode) in modes.items():
        ttk.Radiobutton(frame, text=text, variable=mode_var, value=mode, command=update_enabled).pack(anchor="w", padx=5)
        if mode == StartupMode.Fixed:
            animation_combo.pack(fill="x", padx=25, pady=2)
        elif mode == StartupMode.RandomSet:
            set_frame.pack(fill="x", padx=20, pady=2)
    update_enabled()

    status_label = ttk.Label(frame, text="", style="TLabel")

    def save():
        mask = 0
        for (i, var) in enumerate(set_vars):
            if var.get():
                mask |= 1 << i
        if mode_var.get() == StartupMode.RandomSet and mask == 0:
            status_label.configure(text="Pick at least one animation")
            return
        for dev in get_selected_devices(devices):
            set_startup_animation(dev, mode_var.get(), animation_combo.current(), mask)
        status_label.configure(text="Saved. Playing it now.")

    save_btn = ttk.Button(frame, text="Save", command=save, style="TButton")
    save_btn.pack(side="left", padx=5, pady=5)
    status_label.pack(side="left", padx=5, pady=5)

    if not current:
        # None of the devices support it
        for child in frame.winfo_children():
            if child not in (status_label, set_frame):
                child.configure(state="disabled")
        for checkbox in set_checkboxes:
            checkbox.configure(state="disabled")
        if in_use:
            status_label.configure(text="Close other programs using the LED Matrix, this updates by itself")
        else:
            status_label.configure(text="Needs a firmware update")

def perform_action(devices, action):
    if action.startswith("game_"):
        from inputmodule.gui.pygames import snake, ledris
        action_map = {
            "game_snake": snake.main_devices,
            "game_ledris": ledris.main_devices,
        }
        if action in action_map:
            threading.Thread(target=action_map[action], args=(devices,), daemon=True).start(),

    action_map = {
        "bootloader": bootloader_jump,
        "sleep": lambda dev: send_command(dev, CommandVals.Sleep, [True]),
        "wake": lambda dev: send_command(dev, CommandVals.Sleep, [False]),
        "start_animation": lambda dev: animate(dev, True),
        "stop_animation": lambda dev: animate(dev, False),
        "start_time": lambda dev: threading.Thread(target=clock, args=(dev,), daemon=True).start(),
        "start_eq": lambda dev: threading.Thread(target=random_eq, args=(dev,), daemon=True).start(),
        "gol_current": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 0]),
        "gol_pattern1": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 1]),
        "gol_blinker": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 2]),
        "gol_toad": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 3]),
        "gol_beacon": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 4]),
        "gol_glider": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 5]),
        "gol_rpentomino": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 7]),
        "gol_acorn": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 8]),
        "gol_diehard": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 9]),
        "gol_lwss": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 10]),
        "gol_mwss": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 11]),
        "gol_hwss": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 12]),
        "gol_fleet": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 13]),
        "gol_octagon": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 14]),
        "gol_tumbler": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 15]),
        "gol_caterer": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 16]),
        "gol_pattern2": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 17]),
        "gol_pattern3": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 18]),
        "gol_pattern4": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 19]),
        "gol_pattern5": lambda dev: send_command(dev, CommandVals.StartGame, [Game.GameOfLife, 20]),
        "game_stop": lambda dev: send_command(dev, CommandVals.GameControl, [GameControlVal.Quit]),
    }
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        if action in action_map:
            action_map[action](dev)

    # Only after sending the command, because it unselects the devices
    if action == "bootloader":
        disable_devices(devices)
        restart_hint()

def set_brightness(devices, value):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        brightness(dev, int(value))

def set_pattern(devices, pattern_name):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        pattern(dev, pattern_name)

def set_percentage(devices, value):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        percentage(dev, int(value))

def show_custom_text(devices, text):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        show_string(dev, text.upper())

def send_image(devices, image_name, image_function):
    selected_devices = get_selected_devices(devices)
    path = os.path.join(resource_path(), "res", image_name)
    if not os.path.exists(path):
        popup(f"Image file {image_name} not found.")
        return
    for dev in selected_devices:
        image_function(dev, path)

def send_symbols(devices):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        show_symbols(dev, ["2", "5", "degC", " ", "thunder"])

def start_countdown(devices, countdown_time):
    selected_devices = get_selected_devices(devices)
    if len(selected_devices) == 1:
        dev = selected_devices[0]
        threading.Thread(target=countdown, args=(dev, int(countdown_time)), daemon=True).start()
    else:
        popup("Select exactly 1 device for this action")

def set_pwm_freq(devices, freq):
    selected_devices = get_selected_devices(devices)
    for dev in selected_devices:
        pwm_freq(dev, freq)

def get_selected_devices(devices):
    return [dev for dev in devices if dev.name in device_checkboxes and device_checkboxes[dev.name][0].get()]

def resource_path():
    """Get absolute path to resource, works for dev and for PyInstaller"""
    try:
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath(".")

    return base_path

def info_popup(msg):
    parent = tk.Tk()
    parent.title("Info")
    message = tk.Message(parent, text=msg, width=800)
    message.pack(padx=20, pady=20)
    parent.mainloop()

def tk_flash_firmware(devices, releases, version, fw_type):
    selected_devices = get_selected_devices(devices)
    if len(selected_devices) != 1:
        info_popup('To flash select exactly 1 device.')
        return
    dev = selected_devices[0]
    firmware_update.flash_firmware(dev, releases[version][fw_type])
    # Disable device that we just flashed
    disable_devices(devices)
    restart_hint()

def restart_hint():
    parent = tk.Tk()
    parent.title("Restart Application")
    message = tk.Message(parent, text="After updating a device,\n restart the application to reload the connections.", width=800)
    message.pack(padx=20, pady=20)
    parent.mainloop()

def disable_devices(devices):
    # Disable checkbox of selected devices
    for dev in devices:
        for name, (checkbox_var, checkbox) in device_checkboxes.items():
            if name == dev.name:
                checkbox_var.set(False)
                checkbox.config(state=tk.DISABLED)
