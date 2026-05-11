import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi
import time

def list_apps():
    desktop = Atspi.get_desktop(0)
    print(f"Desktop child count: {desktop.get_child_count()}")
    for i in range(desktop.get_child_count()):
        child = desktop.get_child_at_index(i)
        if child:
            print(f"- {child.get_name()} (Role: {child.get_role_name()})")

if __name__ == "__main__":
    list_apps()
