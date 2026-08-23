#!/usr/bin/env python3

import os
import sys
import argparse
import signal
import http.server
import socketserver

# Global variables for log file re-opening during rotation
log_file_path = None
log_file_obj = None

def setup_logging(path):
    """Opens the log file and redirects stdout and stderr to it."""
    global log_file_obj, log_file_path
    log_file_path = os.path.abspath(path)
    
    log_dir = os.path.dirname(log_file_path)
    if log_dir and not os.path.exists(log_dir):
        os.makedirs(log_dir, exist_ok=True)
        
    # Line buffering (buffering=1) ensures logs are written immediately
    log_file_obj = open(log_file_path, "a+", buffering=1)
    
    # Redirect standard output and error descriptors
    os.dup2(log_file_obj.fileno(), sys.stdout.fileno())
    os.dup2(log_file_obj.fileno(), sys.stderr.fileno())

def handle_sighup(signum, frame):
    """Handles SIGHUP signal to close and re-open the log file for logrotate compatibility."""
    global log_file_obj, log_file_path
    if log_file_path and log_file_obj:
        try:
            log_file_obj.close()
            log_file_obj = open(log_file_path, "a+", buffering=1)
            os.dup2(log_file_obj.fileno(), sys.stdout.fileno())
            os.dup2(log_file_obj.fileno(), sys.stderr.fileno())
            print("[INFO] Log file successfully re-opened following SIGHUP.")
        except Exception as e:
            sys.__stderr__.write(f"Error re-opening log file: {e}\n")

class SafeStaticHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, directory=None, **kwargs):
        self.base_dir = os.path.abspath(directory)
        super().__init__(*args, directory=self.base_dir, **kwargs)

    def translate_path(self, path):
        target_path = super().translate_path(path)
        real_target_path = os.path.abspath(target_path)
        
        # Security barrier: enforce directory confinement
        if not real_target_path.startswith(self.base_dir):
            print(f"[WARNING] Blocked restricted access attempt to: {path}")
            return self.base_dir
            
        return real_target_path

def daemonize(log_path):
    """Detaches the script process from the current terminal context (Unix fork)."""
    try:
        pid = os.fork()
        if pid > 0:
            sys.exit(0)
    except OSError as e:
        sys.exit(f"Fork error #1: {e}")

    os.setsid()
    os.chdir("/")
    os.umask(0)

    try:
        pid = os.fork()
        if pid > 0:
            sys.exit(0)
    except OSError as e:
        sys.exit(f"Fork error #2: {e}")

    sys.stdin.flush()
    si = open(os.devnull, "r")
    os.dup2(si.fileno(), sys.stdin.fileno())

    setup_logging(log_path)

def run_server(port, directory, public, daemon, log_path):
    if not os.path.isdir(directory):
        print(f"Error: The specified directory does not exist: {directory}")
        sys.exit(1)

    abs_directory = os.path.abspath(directory)
    pid_file = os.path.join(abs_directory, "server.pid")

    handler = lambda *args, **kwargs: SafeStaticHTTPRequestHandler(*args, directory=abs_directory, **kwargs)
    socketserver.TCPServer.allow_reuse_address = True
    
    host = "0.0.0.0" if public else "127.0.0.1"
    accessibility = "all network interfaces (public)" if public else "localhost only (private)"

    if daemon:
        print(f"Starting server in the background...")
        print(f"PID File location: {pid_file}")
        print(f"Log File location: {os.path.abspath(log_path)}")
        
        signal.signal(signal.SIGHUP, handle_sighup)
        daemonize(log_path)
        
        # Save actual background process ID to the PID file
        with open(pid_file, "w") as f:
            f.write(str(os.getpid()))

    print(f"[INFO] Server live at http://{host if public else 'localhost'}:{port} ({accessibility})")
    print(f"[INFO] Serving path: {abs_directory}")

    with socketserver.TCPServer((host, port), handler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n[INFO] Server execution interrupted manually.")
        finally:
            if daemon and os.path.exists(pid_file):
                os.remove(pid_file)
            if log_file_obj:
                log_file_obj.close()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Secure Static HTTP Server with Log Rotation Support")
    
    parser.add_argument("directory", nargs="?", default=".", help="Target directory path to serve")
    parser.add_argument("port", nargs="?", type=int, default=8080, help="Server port selection")
    parser.add_argument("-p", "--public", action="store_true", help="Expose server to external networks")
    parser.add_argument("-d", "--daemon", action="store_true", help="Execute in background daemon mode")
    parser.add_argument("-l", "--log", type=str, help="Log file output path (Required if running as daemon)")

    args = parser.parse_args()

    if args.daemon and not args.log:
        parser.error("Daemon execution (-d) strictly requires providing a log path (-l / --log)!")

    run_server(args.port, args.directory, args.public, args.daemon, args.log)
