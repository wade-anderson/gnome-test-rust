import subprocess
import time
import sys
import os
import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi

def find_child(parent, name=None, role=None, timeout=10):
    """Recursively find a child by name and/or role using BFS."""
    start = time.time()
    while time.time() - start < timeout:
        queue = [parent]
        while queue:
            curr = queue.pop(0)
            count = curr.get_child_count()
            for i in range(count):
                child = curr.get_child_at_index(i)
                if not child:
                    continue
                
                c_name = child.get_name()
                c_role = child.get_role_name()
                
                # print(f"Checking: {c_name} (Role: {c_role})")
                
                match_name = name is None or c_name == name
                match_role = role is None or c_role == role
                
                if match_name and match_role:
                    return child
                
                # Add to queue for further search
                if child.get_child_count() > 0:
                    queue.append(child)
        time.sleep(0.5)
    return None

def find_app_by_binary(binary_name, timeout=15):
    """Find the application in the Atspi tree by its binary name."""
    print(f"Searching for application name containing: '{binary_name}'")
    start = time.time()
    while time.time() - start < timeout:
        desktop = Atspi.get_desktop(0)
        if not desktop:
            print("Warning: Could not get desktop object.")
            time.sleep(1)
            continue
            
        count = desktop.get_child_count()
        found_names = []
        for i in range(count):
            child = desktop.get_child_at_index(i)
            if child:
                name = child.get_name()
                found_names.append(name)
                if binary_name in name:
                    return child
        print(f"Apps currently in tree: {found_names}")
        time.sleep(2)
    return None

def run_test():
    # Use the debug binary for testing as it was built last
    binary = os.path.join(os.getcwd(), "target/debug/gnome-test-rust")
    if not os.path.exists(binary):
        binary = os.path.join(os.getcwd(), "target/release/gnome-test-rust")
    
    if not os.path.exists(binary):
        print(f"Binary not found: {binary}")
        return False

    print(f"Starting application: {binary}")
    # Start with accessibility enabled
    env = os.environ.copy()
    proc = subprocess.Popen([binary], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    
    try:
        # 1. Find the application
        print("Waiting for application to appear in accessibility tree...")
        app = find_app_by_binary("gnome-test-rust")
        if not app:
            print("FAILED: Application not found in accessibility tree.")
            return False
        print(f"SUCCESS: Found application '{app.get_name()}'")

        # 1.1 Find the main window
        print("Locating main window...")
        window = find_child(app, name="Gnome Test Rust", role="frame")
        if not window:
            print("FAILED: Main window 'Gnome Test Rust' not found.")
            return False
        print("SUCCESS: Found main window.")

        # 2. Verify "Hello World" label
        print("Verifying 'Hello World' label...")
        label = find_child(window, name="Hello World", role="label")
        if not label:
            print("FAILED: 'Hello World' label not found.")
            return False
        print("SUCCESS: 'Hello World' label found.")

        # 3. Find and click "Map" button
        print("Locating 'Map' button...")
        map_button = find_child(window, name="Map", role="button")
        if not map_button:
            print("FAILED: 'Map' button not found.")
            return False
        
        print("Clicking 'Map' button...")
        action = map_button.get_action_iface()
        action.do_action(0)
        
        # 4. Verify "Map View" window appears
        print("Waiting for 'Map View' window...")
        map_window = find_app_by_binary("Map View")
        if not map_window:
             map_window = find_child(app, name="Map View", role="frame")

        if not map_window:
            print("FAILED: 'Map View' window did not appear.")
            return False
        print("SUCCESS: 'Map View' window is visible.")

        # 5. Find and click "Close Map" button
        print("Locating 'Close Map' button...")
        close_button = find_child(map_window, name="Close Map", role="button")
        if not close_button:
            print("FAILED: 'Close Map' button not found.")
            return False
            
        print("Clicking 'Close Map' button...")
        close_button.get_action_iface().do_action(0)
        time.sleep(1)

        # 6. Click "OK" button to exit
        print("Locating 'OK' button...")
        ok_button = find_child(window, name="OK", role="button")
        if not ok_button:
            print("FAILED: 'OK' button not found.")
            return False
            
        print("Clicking 'OK' button (Exiting app)...")
        ok_button.get_action_iface().do_action(0)
        
        # 7. Wait for process to exit
        try:
            proc.wait(timeout=5)
            print("SUCCESS: Application exited cleanly.")
            return True
        except subprocess.TimeoutExpired:
            print("FAILED: Application did not exit after clicking OK.")
            return False
        
    except Exception as e:
        print(f"ERROR during test: {e}")
        return False
    finally:
        if proc.poll() is None:
            print("Terminating application...")
            proc.terminate()

if __name__ == "__main__":
    success = run_test()
    if success:
        print("\n=== E2E TEST PASSED ===")
        sys.exit(0)
    else:
        print("\n=== E2E TEST FAILED ===")
        sys.exit(1)
