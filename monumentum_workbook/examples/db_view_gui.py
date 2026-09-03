#!/usr/bin/env python3

import subprocess
import tkinter as tk
from tkinter import filedialog, scrolledtext, messagebox
import os
import sys

class MonumentumViewer:
    def __init__(self, root):
        self.root = root
        self.root.title("Monumentum DB Viewer")
        self.root.geometry("800x600")

        top = tk.Frame(root)
        top.pack(side=tk.TOP, fill=tk.X, padx=5, pady=5)

        self.file_var = tk.StringVar()
        self.file_entry = tk.Entry(top, textvariable=self.file_var, width=60)
        self.file_entry.pack(side=tk.LEFT, expand=True, fill=tk.X, padx=(0,5))

        browse_btn = tk.Button(top, text="Browse", command=self.browse_file)
        browse_btn.pack(side=tk.LEFT, padx=2)

        load_btn = tk.Button(top, text="Load", command=self.load_file)
        load_btn.pack(side=tk.LEFT, padx=2)

        bottom = tk.Frame(root)
        bottom.pack(side=tk.BOTTOM, fill=tk.BOTH, expand=True, padx=5, pady=5)

        self.output = scrolledtext.ScrolledText(bottom, wrap=tk.WORD, font=("Courier", 10))
        self.output.pack(fill=tk.BOTH, expand=True)

    def browse_file(self):
        filename = filedialog.askopenfilename(
            title="Select .monumentum file",
            filetypes=[("Monumentum files", "*.monumentum"), ("All files", "*.*")],
        )
        if filename:
            self.file_var.set(filename)

    def load_file(self):
        path = self.file_var.get().strip()
        if not path:
            messagebox.showerror("Error", "Please select a file.")
            return
        if not os.path.exists(path):
            messagebox.showerror("Error", f"File not found:\n{path}")
            return

        self.output.delete(1.0, tk.END)
        self.output.insert(tk.END, f"Loading {path}...\n")

        script_dir = os.path.dirname(os.path.abspath(__file__))
        candidates = [
            os.path.join("target", "debug", "examples", "db_view"),
            os.path.join(script_dir, "target", "debug", "examples", "db_view"),
        ]
        db_view_bin = None
        for c in candidates:
            if os.path.exists(c):
                db_view_bin = c
                break

        if not db_view_bin:
            cmd = ["cargo", "run", "--quiet", "--example", "db_view", "--", path]
            self.output.insert(tk.END, "Building db_view via cargo...\n")
        else:
            cmd = [db_view_bin, path]

        try:
            result = subprocess.run(
                cmd,
                cwd=os.path.dirname(script_dir) if db_view_bin is None else None,
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode != 0:
                self.output.insert(tk.END, f"Error:\n{result.stderr}")
                return
            self.output.delete(1.0, tk.END)
            self.output.insert(tk.END, result.stdout)
        except subprocess.TimeoutExpired:
            self.output.insert(tk.END, "Error: db_view timed out.")
        except Exception as e:
            self.output.insert(tk.END, f"Error: {e}")

def main():
    root = tk.Tk()
    app = MonumentumViewer(root)
    root.mainloop()

if __name__ == "__main__":
    main()
