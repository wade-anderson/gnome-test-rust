import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi
import sys

def dump_tree(obj, indent=0):
    if not obj: return
    name = obj.get_name()
    role = obj.get_role_name()
    print("  " * indent + f"- {name} (Role: {role})")
    for i in range(obj.get_child_count()):
        dump_tree(obj.get_child_at_index(i), indent + 1)

def dump_app(app_name):
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        child = desktop.get_child_at_index(i)
        if child and app_name in child.get_name():
            dump_tree(child)
            return
    print(f"App {app_name} not found")

if __name__ == "__main__":
    dump_app("gnome-test-rust")
