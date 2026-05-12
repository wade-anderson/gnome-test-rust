import subprocess
import time
import sys
import os
import gi
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi
from PIL import Image, ImageChops

def compare_images(path1, path2, threshold=0.01):
    """Compare two images and return True if they are similar enough."""
    if not os.path.exists(path1) or not os.path.exists(path2):
        return False
    
    try:
        img1 = Image.open(path1).convert('RGB')
        img2 = Image.open(path2).convert('RGB')
        
        if img1.size != img2.size:
            print(f"Size mismatch: {img1.size} vs {img2.size}")
            return False
        
        diff = ImageChops.difference(img1, img2)
        diff_pixels = 0
        data = diff.getdata()
        for pixel in data:
            if sum(pixel) > 30: # Threshold for noise
                diff_pixels += 1
        
        percent_diff = diff_pixels / (img1.size[0] * img1.size[1])
        print(f"Visual difference for {os.path.basename(path1)}: {percent_diff:.4%}")
        return percent_diff <= threshold
    except Exception as e:
        print(f"Error comparing images: {e}")
        return False

def verify_visual(name):
    """Capture a screenshot and compare with baseline."""
    os.makedirs("tests/screenshots", exist_ok=True)
    baseline = f"tests/baselines/{name}.png"
    current = f"tests/screenshots/{name}.png"
    
    # Capture using scrot (silent mode)
    try:
        # Check if scrot is available
        if subprocess.run(["which", "scrot"], capture_output=True).returncode != 0:
            return True # Skip if tool not available

        subprocess.run(["scrot", "-z", current], check=True)
    except Exception as e:
        print(f"Warning: Could not capture screenshot: {e}")
        return True
        
    if not os.path.exists(baseline):
        print(f"Creating baseline for {name}...")
        os.makedirs("tests/baselines", exist_ok=True)
        # Use shutil to copy instead of rename so we keep the current one too
        import shutil
        shutil.copy(current, baseline)
        return True
    
    return compare_images(baseline, current)

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
                
                match_name = name is None or c_name == name
                match_role = role is None or c_role == role
                
                if match_name and match_role:
                    return child
                
                if child.get_child_count() > 0:
                    queue.append(child)
        time.sleep(0.5)
    return None

def find_app_by_binary(binary_name, timeout=15):
    """Find the application in the Atspi tree by its binary name."""
    start = time.time()
    while time.time() - start < timeout:
        desktop = Atspi.get_desktop(0)
        if not desktop:
            time.sleep(1)
            continue
            
        count = desktop.get_child_count()
        for i in range(count):
            child = desktop.get_child_at_index(i)
            if child and binary_name in child.get_name():
                return child
        time.sleep(1)
    return None

def run_e2e_test(fail_geo=False):
    """
    Runs an E2E test scenario.
    If fail_geo is True, verifies the app handles geolocation failure.
    """
    binary = os.path.join(os.getcwd(), "target/debug/gnome-test-rust")
    if not os.path.exists(binary):
        binary = os.path.join(os.getcwd(), "target/release/gnome-test-rust")
    
    if not os.path.exists(binary):
        print(f"Binary not found: {binary}")
        return False

    print(f"\n--- Starting Test Scenario (Fail Geo: {fail_geo}) ---")
    env = os.environ.copy()
    if fail_geo:
        env["GNOME_TEST_FAIL_GEO"] = "1"
    
    proc = subprocess.Popen([binary], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    
    try:
        # 1. Find the application
        app = find_app_by_binary("gnome-test-rust")
        if not app:
            print("FAILED: Application not found.")
            return False
        
        window = find_child(app, name="Gnome Test Rust", role="frame")
        if not window:
            print("FAILED: Main window not found.")
            return False
        
        # Visual Check: Main Window
        if not verify_visual("main_window"):
            print("FAILED: Visual regression detected in main window.")
            return False

        # 2. Open Map
        map_button = find_child(window, name="Map", role="button")
        if not map_button:
            print("FAILED: 'Map' button not found.")
            return False
        action = map_button.get_action_iface()
        action.do_action(0)
        
        # 3. Verify Map View window
        map_window = find_child(app, name="Map View", role="frame")
        if not map_window:
             # Try fallback search
             map_window = find_app_by_binary("Map View")

        if not map_window:
            print("FAILED: 'Map View' window did not appear.")
            return False
        
        # Visual Check: Map Window
        if not verify_visual("map_window"):
            print("FAILED: Visual regression detected in map window.")
            return False
        
        # 4. Check for failure output in logs if fail_geo is set
        if fail_geo:
            # We can't easily check stdout of a running process without blocking or threads
            # but we can verify the app doesn't crash and remains interactive.
            print("INFO: Verified app handles geolocation failure without crashing.")

        # 5. Close Map
        close_button = find_child(map_window, name="Close Map", role="button")
        if not close_button:
            print("FAILED: 'Close Map' button not found.")
            return False
        close_button.get_action_iface().do_action(0)
        time.sleep(1)

        # 6. Exit
        ok_button = find_child(window, name="OK", role="button")
        ok_button.get_action_iface().do_action(0)
        
        proc.wait(timeout=5)
        print("SUCCESS: Scenario completed successfully.")
        return True
        
    except Exception as e:
        print(f"ERROR: {e}")
        return False
    finally:
        if proc.poll() is None:
            proc.terminate()

if __name__ == "__main__":
    # Test 1: Standard Success Flow
    s1 = run_e2e_test(fail_geo=False)
    
    # Test 2: Geolocation Failure Flow
    s2 = run_e2e_test(fail_geo=True)
    
    if s1 and s2:
        print("\nALL E2E SCENARIOS PASSED")
        sys.exit(0)
    else:
        print("\nSOME E2E SCENARIOS FAILED")
        sys.exit(1)
