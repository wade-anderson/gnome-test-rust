import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi
import subprocess
import time
import os

def dump_tree(node, depth=0):
    if not node: return
    try:
        print("  " * depth + f"- {node.get_name()} [{node.get_role_name()}]")
        for i in range(node.get_child_count()):
            dump_tree(node.get_child_at_index(i), depth + 1)
    except Exception as e:
        print("  " * depth + f"- Error accessing node: {e}")

def list_apps():
    binary = os.path.join(os.getcwd(), "target/debug/gnome-test-rust")
    proc = subprocess.Popen([binary], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    
    time.sleep(3) # Wait for it to register
    
    try:
        desktop = Atspi.get_desktop(0)
        count = desktop.get_child_count()
        for i in range(count):
            child = desktop.get_child_at_index(i)
            if child and "gnome-test-rust" in child.get_name():
                print(f"Found app: {child.get_name()}")
                dump_tree(child)
    finally:
        proc.terminate()

if __name__ == "__main__":
    list_apps()
